use std::time::{Duration, Instant};
use winapi::{
    um::winuser::{WindowFromDC, GetClientRect},
    shared::windef::{HDC, RECT},
    um::wingdi::{wglMakeCurrent}
};
use egui::{RawInput, Pos2, Vec2};
use crate::{
    graphics::context::{PayloadContext, AppTab},
    ui::main_window::render_main_window,
    ui::notification_manager::NotificationManager
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
        notification_manager: &mut context.notification_manager,
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
        show_edit_dialog: &mut context.show_edit_dialog,
        edit_account_name: &mut context.edit_account_name,
        edit_username: &mut context.edit_username,
        edit_player_id: &mut context.edit_player_id,
        edit_access_token: &mut context.edit_access_token,
        edit_session_type: &mut context.edit_session_type,
        edit_original_name: &mut context.edit_original_name,
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
    context.notification_manager.render_in_context(&context.egui_ctx, &mut context.icon_manager, screen_size);

    let clipped_primitives = context.egui_ctx.tessellate(shapes, pixels_per_point);

    context.painter.paint_and_update_textures(
        context.dimensions,
        pixels_per_point,
        &clipped_primitives,
        &textures_delta,
    );

    Ok(())
}

pub struct UiState<'a> {
    pub new_username: &'a mut String,
    pub new_player_id: &'a mut String,
    pub new_access_token: &'a mut String,
    pub new_session_type: &'a mut String,
    pub notification_manager: &'a NotificationManager,
    pub clipboard: &'a mut crate::input::clipboard::ClipboardManager,
    pub selected_tab: &'a mut AppTab,
    pub account_name_input: &'a mut String,
    pub selected_account: &'a mut Option<String>,
    pub show_manual_input_dialog: &'a mut bool,
    pub manual_account_name: &'a mut String,
    pub manual_username: &'a mut String,
    pub manual_player_id: &'a mut String,
    pub manual_access_token: &'a mut String,
    pub manual_session_type: &'a mut String,
    pub show_edit_dialog: &'a mut bool,
    pub edit_account_name: &'a mut String,
    pub edit_username: &'a mut String,
    pub edit_player_id: &'a mut String,
    pub edit_access_token: &'a mut String,
    pub edit_session_type: &'a mut String,
    pub edit_original_name: &'a mut String,
}