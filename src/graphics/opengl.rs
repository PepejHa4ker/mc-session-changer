use crate::{
    input::clipboard::ClipboardManager,
    graphics::context::{AppTab, PayloadContext},
    core::state::GlobalState,
    input::window_proc::window_proc,
    utils::SafeGLContext,
    graphics::svg_icons::SvgIconManager,
    ui::notification_manager::NotificationManager
};
use egui::Context;
use glow::Context as GlowContext;
use std::time::Instant;
use winapi::{
    shared::windef::HDC,
    um::libloaderapi::{GetModuleHandleA, GetProcAddress},
    um::wingdi::{wglCreateContext, wglGetCurrentContext, wglMakeCurrent, wglShareLists},
    um::winuser::{GetClientRect, SetWindowLongPtrW, WindowFromDC, GWLP_WNDPROC}
};

pub unsafe fn create_render_context(hdc: HDC) -> Result<PayloadContext, String> {
    if hdc.is_null() {
        return Err("HDC is null".to_string());
    }

    let window = WindowFromDC(hdc);
    if window.is_null() {
        return Err("Failed to get window from DC".to_string());
    }

    GlobalState::instance().set_current_window(window as isize);

    let mut dimensions = winapi::shared::windef::RECT::default();
    GetClientRect(window, &mut dimensions);

    let width = (dimensions.right - dimensions.left) as u32;
    let height = (dimensions.bottom - dimensions.top) as u32;

    if width == 0 || height == 0 {
        return Err("Invalid window dimensions".to_string());
    }

    let original_proc = SetWindowLongPtrW(window, GWLP_WNDPROC, window_proc as _);
    if original_proc != 0 {
        GlobalState::instance().set_original_wndproc(original_proc as usize);
    }

    let game_context = wglGetCurrentContext();
    if game_context.is_null() {
        return Err("No OpenGL context available".to_string());
    }

    let our_context = wglCreateContext(hdc);
    if our_context.is_null() {
        return Err("Failed to create OpenGL context".to_string());
    }

    if wglShareLists(game_context, our_context) == 0 {
        winapi::um::wingdi::wglDeleteContext(our_context);
        return Err("Failed to share OpenGL contexts".to_string());
    }

    if wglMakeCurrent(hdc, our_context) == 0 {
        winapi::um::wingdi::wglDeleteContext(our_context);
        return Err("Failed to make our context current".to_string());
    }

    let glow_context = create_glow_context()?;
    let glow_context = std::sync::Arc::new(glow_context);
    let egui_ctx = Context::default();

    let painter = egui_glow::Painter::new(glow_context.clone(), "", None, false)
        .map_err(|e| {
            winapi::um::wingdi::wglDeleteContext(our_context);
            format!("Failed to create painter: {}", e)
        })?;

    if wglMakeCurrent(hdc, game_context) == 0 {
        winapi::um::wingdi::wglDeleteContext(our_context);
        return Err("Failed to restore game context".to_string());
    }

    GlobalState::instance().initialize_account_manager();

    Ok(PayloadContext {
        painter,
        egui_ctx,
        dimensions: [width, height],
        _glow_context: glow_context,
        our_gl_context: SafeGLContext::new(our_context),
        game_gl_context: SafeGLContext::new(game_context),
        start_time: Instant::now(),
        last_frame_time: None,
        input_events: Vec::new(),
        clipboard: ClipboardManager::new(window),
        last_notification_update: None,
        icon_manager: SvgIconManager::new(),
        notification_manager: NotificationManager::new(),
        new_username: String::new(),
        new_player_id: String::new(),
        new_access_token: String::new(),
        new_session_type: "mojang".to_string(),
        selected_tab: AppTab::SessionChanger,
        account_name_input: String::new(),
        selected_account: None,
        show_manual_input_dialog: false,
        manual_account_name: String::new(),
        manual_username: String::new(),
        manual_player_id: String::new(),
        manual_access_token: String::new(),
        manual_session_type: "mojang".to_string(),
        show_edit_dialog: false,
        edit_account_name: String::new(),
        edit_username: String::new(),
        edit_player_id: String::new(),
        edit_access_token: String::new(),
        edit_session_type: String::new(),
        edit_original_name: String::new(),
    })
}

unsafe fn create_glow_context() -> Result<GlowContext, String> {
    let glow_context = GlowContext::from_loader_function(|name| {
        let c_str = match std::ffi::CString::new(name) {
            Ok(c) => c,
            Err(_) => return std::ptr::null(),
        };

        let proc_addr = winapi::um::wingdi::wglGetProcAddress(c_str.as_ptr());
        if !proc_addr.is_null() {
            return proc_addr as *const std::ffi::c_void;
        }

        let opengl32 = GetModuleHandleA(b"opengl32.dll\0".as_ptr() as *const i8);
        if !opengl32.is_null() {
            let proc_addr = GetProcAddress(opengl32, c_str.as_ptr());
            if !proc_addr.is_null() {
                return proc_addr as *const std::ffi::c_void;
            }
        }

        std::ptr::null()
    });

    Ok(glow_context)
}