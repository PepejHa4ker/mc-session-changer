use crate::core::state::GlobalState;
use crate::graphics::opengl::create_render_context;
use crate::input::clipboard::ClipboardManager;
use crate::input::window_proc::window_proc;
use crate::SHOULD_UNLOAD;
use parking_lot::Mutex;
use std::ffi::c_void;
use winapi::{
    shared::windef::HDC,
    um::libloaderapi::{GetModuleHandleA, GetProcAddress},
    um::wingdi::wglGetCurrentContext,
    um::winuser::{GetAsyncKeyState, SetWindowLongPtrW, WindowFromDC, GWLP_WNDPROC},
};
use winapi::um::winuser::VK_INSERT;

retour::static_detour! {
    static SwapBuffersDetour: unsafe extern "system" fn(HDC) -> i32;
    static GLReadPixelsDetour: unsafe extern "C" fn(i32, i32, i32, i32, u32, u32, *mut c_void);
    static GLGetTexImageDetour: unsafe extern "C" fn(u32, i32, u32, u32, *mut c_void);
    static GLPixelStoreiDetour: unsafe extern "C" fn(u32, i32);
    static GLBindTextureDetour: unsafe extern "C" fn(u32, u32);
}

#[derive(Debug)]
pub enum HookError {
    FunctionNotFound(&'static str),
    DetourInitializationFailed(retour::Error),
    DetourEnableFailed(retour::Error),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::FunctionNotFound(func) => write!(f, "Function not found: {}", func),
            HookError::DetourInitializationFailed(e) => {
                write!(f, "Detour initialization failed: {:?}", e)
            }
            HookError::DetourEnableFailed(e) => write!(f, "Detour enable failed: {:?}", e),
        }
    }
}

impl std::error::Error for HookError {}

unsafe fn get_opengl_proc_address(name: &str) -> Option<*const c_void> {
    let c_name = std::ffi::CString::new(name).ok()?;

    let try_get_proc_from_module = |module_name: &[u8]| -> Option<*const c_void> {
        let module_handle = GetModuleHandleA(module_name.as_ptr() as *const i8);
        if !module_handle.is_null() {
            let proc_addr = GetProcAddress(module_handle, c_name.as_ptr());
            if !proc_addr.is_null() {
                return Some(proc_addr as *const c_void);
            }
        }
        None
    };

    let proc_addr = winapi::um::wingdi::wglGetProcAddress(c_name.as_ptr());
    if !proc_addr.is_null() {
        return Some(proc_addr as *const c_void);
    }

    if let Some(addr) = try_get_proc_from_module(b"gdi32.dll\0") {
        return Some(addr);
    }

    try_get_proc_from_module(b"opengl32.dll\0")
}


pub fn initialize_opengl_hooks() -> Result<(), HookError> {
    unsafe {
        let swap_buffers_addr = get_opengl_proc_address("SwapBuffers").ok_or(HookError::FunctionNotFound("SwapBuffers"))?;
        let gl_read_pixels_addr = get_opengl_proc_address("glReadPixels").ok_or(HookError::FunctionNotFound("glReadPixels"))?;
        let gl_get_tex_image_addr = get_opengl_proc_address("glGetTexImage").ok_or(HookError::FunctionNotFound("glGetTexImage"))?;
        let gl_pixel_storei_addr = get_opengl_proc_address("glPixelStorei").ok_or(HookError::FunctionNotFound("glPixelStorei"))?;
        let gl_bind_texture_addr = get_opengl_proc_address("glBindTexture").ok_or(HookError::FunctionNotFound("glBindTexture"))?;

        SwapBuffersDetour
            .initialize(std::mem::transmute(swap_buffers_addr), |hdc| {
                hk_swap_buffers(hdc)
            })
            .map_err(HookError::DetourInitializationFailed)?;
        GLReadPixelsDetour
            .initialize(
                std::mem::transmute(gl_read_pixels_addr),
                |x, y, width, height, format, r#type, pixels| {
                    hooked_gl_read_pixels(x, y, width, height, format, r#type, pixels)
                },
            )
            .map_err(HookError::DetourInitializationFailed)?;

        GLGetTexImageDetour
            .initialize(
                std::mem::transmute(gl_get_tex_image_addr),
                |target, level, format, r#type, pixels| {
                    hooked_gl_get_tex_image(target, level, format, r#type, pixels)
                },
            )
            .map_err(HookError::DetourInitializationFailed)?;
        GLPixelStoreiDetour
            .initialize(std::mem::transmute(gl_pixel_storei_addr), |pname, param| {
                hooked_gl_pixel_storei(pname, param)
            })
            .map_err(HookError::DetourInitializationFailed)?;
        GLBindTextureDetour
            .initialize(
                std::mem::transmute(gl_bind_texture_addr),
                |target, texture| hooked_gl_bind_texture(target, texture),
            )
            .map_err(HookError::DetourInitializationFailed)?;
        SwapBuffersDetour
            .enable()
            .map_err(HookError::DetourEnableFailed)?;
        GLReadPixelsDetour
            .enable()
            .map_err(HookError::DetourEnableFailed)?;
        GLGetTexImageDetour
            .enable()
            .map_err(HookError::DetourEnableFailed)?;
        GLPixelStoreiDetour
            .enable()
            .map_err(HookError::DetourEnableFailed)?;
        GLBindTextureDetour
            .enable()
            .map_err(HookError::DetourEnableFailed)?;

        tracing::info!("OpenGL hooks initialized successfully");
        Ok(())
    }
}

unsafe extern "system" fn hk_swap_buffers(hdc: HDC) -> i32 {
    let state = GlobalState::instance();

    if SHOULD_UNLOAD.load(std::sync::atomic::Ordering::Acquire) && !state.is_unload_initiated() {
        state.set_unload_initiated(true);
        let result = SwapBuffersDetour.call(hdc);
        crate::core::cleanup::initiate_cleanup();
        return result;
    }

    if state.is_unload_initiated() {
        return SwapBuffersDetour.call(hdc);
    }

    ensure_window_hook(hdc);
    handle_input();

    let current_context = wglGetCurrentContext();
    if current_context.is_null() {
        return SwapBuffersDetour.call(hdc);
    }

    if state.is_menu_visible() {
        if let Err(e) = render_overlay(hdc) {
            tracing::warn!("Failed to render overlay: {}", e);
        }
    }

    SwapBuffersDetour.call(hdc)
}

fn handle_input() {
    let state = GlobalState::instance();
    let frame_count = state.increment_frame_count();

    if frame_count % 30 == 0 {
        unsafe {
            let current_key_state = GetAsyncKeyState(VK_INSERT) as u32;
            let last_key_state = state.get_last_key_state();

            if current_key_state != 0 && last_key_state == 0 {
                let new_visibility = !state.is_menu_visible();
                state.set_menu_visible(new_visibility);
                tracing::debug!("Menu visibility toggled: {}", new_visibility);
            }

            state.set_last_key_state(current_key_state);
        }
    }
}

fn ensure_window_hook(hdc: HDC) {
    let state = GlobalState::instance();

    // Skip until the render context is initialized so we do not overwrite the original wndproc.
    if state.get_context().get().is_none() {
        return;
    }

    let window = unsafe { WindowFromDC(hdc) };
    if window.is_null() {
        return;
    }

    if state.get_current_window() == window as isize {
        return;
    }

    unsafe {
        let original_proc = SetWindowLongPtrW(window, GWLP_WNDPROC, window_proc as _);
        if original_proc == 0 {
            tracing::warn!("Failed to hook window procedure after window change");
            return;
        }

        state.set_original_wndproc(original_proc as usize);
        state.set_current_window(window as isize);
    }

    if let Some(context_mutex) = state.get_context().get() {
        if let Some(mut context_guard) = context_mutex.try_lock() {
            if let Some(context) = context_guard.as_mut() {
                context.clipboard = ClipboardManager::new(window);
            }
        }
    }

    tracing::debug!("Window procedure reattached for new window handle");
}

fn render_overlay(hdc: HDC) -> Result<(), Box<dyn std::error::Error>> {
    let state = GlobalState::instance();
    let context_mutex = state
        .get_context()
        .get_or_init(|| unsafe { Mutex::new(create_render_context(hdc).ok()) });

    match context_mutex.try_lock() {
        Some(mut context_guard) => {
            if let Some(context) = context_guard.as_mut() {
                context.render(hdc)?;
            }
            Ok(())
        }
        None => {
            tracing::warn!("Failed to acquire context lock for rendering");
            Ok(())
        }
    }
}

unsafe extern "C" fn hooked_gl_read_pixels(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    format: u32,
    r#type: u32,
    pixels: *mut c_void,
) {
    tracing::debug!(
        "glReadPixels called: {}x{} at ({}, {})",
        width,
        height,
        x,
        y
    );
    if !GlobalState::instance().is_menu_visible() {
        GLReadPixelsDetour.call(x, y, width, height, format, r#type, pixels);
        return;
    }
    GlobalState::instance().set_menu_visible(false);
    GLReadPixelsDetour.call(x, y, width, height, format, r#type, pixels);
    GlobalState::instance().set_menu_visible(true);
}

unsafe extern "C" fn hooked_gl_get_tex_image(
    target: u32,
    level: i32,
    format: u32,
    r#type: u32,
    pixels: *mut c_void,
) {
    tracing::debug!("glGetTexImage called: target={}, level={}", target, level);
    if !GlobalState::instance().is_menu_visible() {
        GLGetTexImageDetour.call(target, level, format, r#type, pixels);
        return;
    }
    GlobalState::instance().set_menu_visible(false);
    GLGetTexImageDetour.call(target, level, format, r#type, pixels);
    GlobalState::instance().set_menu_visible(true);
}

unsafe extern "C" fn hooked_gl_pixel_storei(pname: u32, param: i32) {
    tracing::debug!("glPixelStorei called: pname={}, param={}", pname, param);
    if !GlobalState::instance().is_menu_visible() {
        GLPixelStoreiDetour.call(pname, param);
        return;
    }
    GlobalState::instance().set_menu_visible(false);
    GLPixelStoreiDetour.call(pname, param);
    GlobalState::instance().set_menu_visible(true);
}

unsafe extern "C" fn hooked_gl_bind_texture(target: u32, texture: u32) {
    tracing::debug!(
        "glBindTexture called: target={}, texture={}",
        target,
        texture
    );
    if !GlobalState::instance().is_menu_visible() {
        GLBindTextureDetour.call(target, texture);
        return;
    }
    GlobalState::instance().set_menu_visible(false);
    GLBindTextureDetour.call(target, texture);
    GlobalState::instance().set_menu_visible(true);
}

pub fn cleanup_opengl_hooks() -> Result<(), retour::Error> {
    unsafe {
        SwapBuffersDetour.disable()?;
        GLReadPixelsDetour.disable()?;
        GLGetTexImageDetour.disable()?;
        GLPixelStoreiDetour.disable()?;
        GLBindTextureDetour.disable()?;
        tracing::info!("OpenGL hooks cleaned up successfully");
        Ok(())
    }
}
