use crate::{
    graphics::context::{PayloadContext},
    ui::main_window::render_main_window,
};
use crate::ui::state::UiState;
use egui::{Pos2, RawInput, Vec2};
use std::time::{Duration, Instant};
use winapi::{
    shared::windef::{HDC, RECT},
    um::wingdi::wglMakeCurrent,
    um::winuser::{GetClientRect, WindowFromDC},
};

pub fn render_frame(context: &mut PayloadContext, hdc: HDC) -> Result<(), String> {
    let window = unsafe { WindowFromDC(hdc) };
    if window.is_null() {
        return Err("Failed to get window from DC".to_string());
    }

    let mut rect = RECT::default();
    unsafe { GetClientRect(window, &mut rect) };

    let width = (rect.right - rect.left) as u32;
    let height = (rect.bottom - rect.top) as u32;

    if width == 0 || height == 0 {
        return Err("Invalid window dimensions".to_string());
    }

    if width != context.dimensions[0] || height != context.dimensions[1] {
        context.dimensions = [width, height];
    }

    unsafe {
        if wglMakeCurrent(hdc, context.our_gl_context.get()) == 0 {
            return Err("Failed to make our OpenGL context current".to_string());
        }
    }

    let now = Instant::now();
    let dt = if let Some(last_update) = context.last_notification_update {
        now - last_update
    } else {
        Duration::from_millis(16)
    };
    context.last_notification_update = Some(now);

    context.notification_manager.update(dt);

    let egui_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        render_egui_frame(context, width, height)
    }));

    unsafe {
        if wglMakeCurrent(hdc, context.game_gl_context.get()) == 0 {
            return Err("Failed to restore game OpenGL context".to_string());
        }
    }

    match egui_result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Panic in egui rendering".to_string()),
    }
}

fn render_egui_frame(context: &mut PayloadContext, width: u32, height: u32) -> Result<(), String> {
    let current_time = Instant::now();

    let time_since_start = if let Some(last_time) = context.last_frame_time {
        let elapsed = current_time.duration_since(last_time);
        if elapsed > Duration::from_millis(100) {
            Duration::from_millis(16)
        } else {
            elapsed
        }
    } else {
        Duration::from_millis(16)
    };

    let time_since_start_total = current_time.duration_since(context.start_time);
    context.last_frame_time = Some(current_time);

    let mut raw_input = RawInput::default();
    raw_input.time = Some(time_since_start_total.as_secs_f64());
    raw_input.predicted_dt = time_since_start.as_secs_f32();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(width as f32, height as f32),
    ));

    let events = context.input_events.drain(..).collect();
    raw_input.events = events;

    let mut ui_state = UiState {
        new_username: &mut context.new_username,
        new_player_id: &mut context.new_player_id,
        new_access_token: &mut context.new_access_token,
        new_session_type: &mut context.new_session_type,
        notification_manager: &context.notification_manager,
        clipboard: &mut context.clipboard,
        selected_tab: &mut context.selected_tab,
        account_name_input: &mut context.account_name_input,
        selected_account: &mut context.selected_account,
        show_manual_input_dialog: &mut context.show_manual_input_dialog,
        manual_account_name: &mut context.manual_account_name,
        manual_username: &mut context.manual_username,
        manual_player_id: &mut context.manual_player_id,
        manual_access_token: &mut context.manual_access_token,
        manual_session_type: &mut context.manual_session_type,
        manual_password: &mut context.manual_password,
        show_edit_dialog: &mut context.show_edit_dialog,
        edit_account_name: &mut context.edit_account_name,
        edit_username: &mut context.edit_username,
        edit_player_id: &mut context.edit_player_id,
        edit_access_token: &mut context.edit_access_token,
        edit_session_type: &mut context.edit_session_type,
        edit_password: &mut context.edit_password,
        edit_original_name: &mut context.edit_original_name,
        auth_in_progress: &mut context.auth_in_progress,
        packet_filter: &mut context.packet_filter,
        packet_show_inbound: &mut context.packet_show_inbound,
        packet_show_outbound: &mut context.packet_show_outbound,
        packet_autoscroll: &mut context.packet_autoscroll,
        packet_paused: &mut context.packet_paused,
        packets_detached: &mut context.packets_detached,
        packets_window_open: &mut context.packets_window_open,
        selected_packet_id: &mut context.selected_packet_id,
        packet_only_pinned: &mut context.packet_only_pinned,
        packet_limit_count: &mut context.packet_limit_count,
        packet_autoclear_oldest: &mut context.packet_autoclear_oldest,
        packet_filter_profiles: &mut context.packet_filter_profiles,
        packet_profile_new_name: &mut context.packet_profile_new_name,
        packet_profile_new_query: &mut context.packet_profile_new_query,
        packet_show_only_new: &mut context.packet_show_only_new,
        packet_last_seen_id: &mut context.packet_last_seen_id,
        packet_secondary_selected_id: &mut context.packet_secondary_selected_id,
        packet_triggers: &mut context.packet_triggers,
        packet_trigger_input: &mut context.packet_trigger_input,
        packet_tag_editor: &mut context.packet_tag_editor,
        packet_color_hex: &mut context.packet_color_hex,
        packet_export_limit: &mut context.packet_export_limit,
        packet_import_buffer: &mut context.packet_import_buffer,

        search_query: &mut context.search_query,
        selected_class: &mut context.selected_class,
        is_loading: &mut context.is_loading,
        error_message: &mut context.error_message,
        detached: &mut context.detached,
        window_open: &mut context.window_open,
        expand_fields: &mut context.expand_fields,
        expand_methods: &mut context.expand_methods,
        member_search_query: &mut context.member_search_query,

        auth_tab_username: &mut context.auth_tab_username,
        auth_tab_password: &mut context.auth_tab_password,
        auth_tab_result_token: &mut context.auth_tab_result_token,
        auth_tab_result_profile: &mut context.auth_tab_result_profile,
        auth_tab_in_progress: &mut context.auth_tab_in_progress,
        auth_tab_error: &mut context.auth_tab_error,
    };

    let mut icon_manager = std::mem::take(&mut context.icon_manager);

    let egui::FullOutput {
        platform_output: _,
        textures_delta,
        pixels_per_point,
        viewport_output: _,
        shapes,
    } = context.egui_ctx.run(raw_input, |ctx| {
        render_main_window(&mut ui_state, &mut icon_manager, ctx);
    });

    context.icon_manager = icon_manager;

    let screen_size = Vec2::new(context.dimensions[0] as f32, context.dimensions[1] as f32);
    context.notification_manager.render_in_context(
        &context.egui_ctx,
        &mut context.icon_manager,
        screen_size,
    );

    let clipped_primitives = context.egui_ctx.tessellate(shapes, pixels_per_point);

    context.painter.paint_and_update_textures(
        context.dimensions,
        pixels_per_point,
        &clipped_primitives,
        &textures_delta,
    );

    Ok(())
}
