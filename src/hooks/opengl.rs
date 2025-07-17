use crate::SHOULD_UNLOAD;
use crate::core::state::GlobalState;
use crate::graphics::opengl::create_render_context;
use parking_lot::Mutex;
use winapi::shared::windef::HDC;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::wingdi::wglGetCurrentContext;
use winapi::um::winuser::GetAsyncKeyState;
use winapi::um::winuser::VK_INSERT;

retour::static_detour! {
    static SwapBuffersDetour: unsafe extern "system" fn(HDC) -> i32;
}

#[derive(Debug)]
pub enum HookError {
    ModuleNotFound(&'static str),
    FunctionNotFound(&'static str),
    DetourInitializationFailed(retour::Error),
    DetourEnableFailed(retour::Error),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::ModuleNotFound(module) => write!(f, "Module not found: {}", module),
            HookError::FunctionNotFound(func) => write!(f, "Function not found: {}", func),
            HookError::DetourInitializationFailed(e) => {
                write!(f, "Detour initialization failed: {:?}", e)
            }
            HookError::DetourEnableFailed(e) => write!(f, "Detour enable failed: {:?}", e),
        }
    }
}

impl std::error::Error for HookError {}

pub fn initialize_opengl_hooks() -> Result<(), HookError> {
    unsafe {
        let gdi32 = GetModuleHandleA(b"gdi32.dll\0".as_ptr() as *const i8);
        if gdi32.is_null() {
            return Err(HookError::ModuleNotFound("gdi32.dll"));
        }

        let swap_buffers_addr = GetProcAddress(gdi32, b"SwapBuffers\0".as_ptr() as *const i8);
        if swap_buffers_addr.is_null() {
            return Err(HookError::FunctionNotFound("SwapBuffers"));
        }

        SwapBuffersDetour
            .initialize(
                std::mem::transmute(swap_buffers_addr),
                |hdc| { hk_swap_buffers(hdc) }
            )
            .map_err(HookError::DetourInitializationFailed)?;

        SwapBuffersDetour
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

pub fn cleanup_opengl_hooks() -> Result<(), retour::Error> {
    unsafe {
        SwapBuffersDetour.disable()?;
        tracing::info!("OpenGL hooks cleaned up successfully");
        Ok(())
    }
}
