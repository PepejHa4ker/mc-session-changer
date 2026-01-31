use crate::{
    core::state::GlobalState,
    graphics::context::{AppTab, PayloadContext},
    graphics::svg_icons::SvgIconManager,
    input::clipboard::ClipboardManager,
    input::window_proc::window_proc,
    ui::notification_manager::NotificationManager,
    utils::SafeGLContext,
};
use egui::Context;
use glow::Context as GlowContext;
use std::time::Instant;
use winapi::{
    shared::windef::HDC,
    um::libloaderapi::{GetModuleHandleA, GetProcAddress},
    um::wingdi::{wglCreateContext, wglGetCurrentContext, wglMakeCurrent, wglShareLists},
    um::winuser::{GWLP_WNDPROC, GetClientRect, SetWindowLongPtrW, WindowFromDC},
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
    GlobalState::instance().initialize_packet_store();

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

    let painter = egui_glow::Painter::new(glow_context.clone(), "", None, false).map_err(|e| {
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
        manual_password: String::new(),
        show_edit_dialog: false,
        edit_account_name: String::new(),
        edit_username: String::new(),
        edit_player_id: String::new(),
        edit_access_token: String::new(),
        edit_session_type: String::new(),
        edit_password: String::new(),
        edit_original_name: String::new(),
        auth_in_progress: false,

        packet_filter: String::new(),
        packet_show_inbound: true,
        packet_show_outbound: true,
        packet_autoscroll: true,
        packet_paused: false,
        packets_detached: false,
        packets_window_open: false,
        selected_packet_id: None,
        packet_only_pinned: false,
        packet_limit_count: 500,
        packet_autoclear_oldest: true,
        packet_filter_profiles: Vec::new(),
        packet_profile_new_name: String::new(),
        packet_profile_new_query: String::new(),
        packet_show_only_new: false,
        packet_last_seen_id: 0,
        packet_secondary_selected_id: None,
        packet_triggers: Vec::new(),
        packet_trigger_input: String::new(),
        packet_tag_editor: String::new(),
        packet_color_hex: String::from("#ffaa00"),
        packet_export_limit: 500,
        packet_import_buffer: String::new(),

        search_query: String::new(),
        selected_class: None,
        is_loading: false,
        error_message: None,
        detached: false,
        window_open: false,
        expand_fields: false,
        expand_methods: false,
        member_search_query: String::new(),

        auth_tab_username: String::new(),
        auth_tab_password: String::new(),
        auth_tab_result_token: String::new(),
        auth_tab_result_profile: String::new(),
        auth_tab_in_progress: false,
        auth_tab_error: None,
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
