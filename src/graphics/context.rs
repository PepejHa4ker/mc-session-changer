use crate::input::clipboard::ClipboardManager;
use crate::utils::SafeGLContext;
use egui::{Context, Event};
use egui_glow::Painter;
use glow::Context as GlowContext;
use std::time::Instant;
use winapi::shared::windef::HDC;
use crate::graphics::svg_icons::SvgIconManager;
use crate::ui::notification_manager::NotificationManager;

pub struct PayloadContext {
    pub painter: Painter,
    pub egui_ctx: Context,
    pub dimensions: [u32; 2],
    pub _glow_context: std::sync::Arc<GlowContext>,
    pub our_gl_context: SafeGLContext,
    pub game_gl_context: SafeGLContext,
    pub start_time: Instant,
    pub last_frame_time: Option<Instant>,
    pub input_events: Vec<Event>,
    pub clipboard: ClipboardManager,
    pub last_notification_update: Option<Instant>,
    pub icon_manager: SvgIconManager,
    pub notification_manager: NotificationManager,

    pub new_username: String,
    pub new_player_id: String,
    pub new_access_token: String,
    pub new_session_type: String,

    pub selected_tab: AppTab,
    pub account_name_input: String,
    pub selected_account: Option<String>,

    pub show_manual_input_dialog: bool,
    pub manual_account_name: String,
    pub manual_username: String,
    pub manual_player_id: String,
    pub manual_access_token: String,
    pub manual_session_type: String,

    pub show_edit_dialog: bool,
    pub edit_account_name: String,
    pub edit_username: String,
    pub edit_player_id: String,
    pub edit_access_token: String,
    pub edit_session_type: String,
    pub edit_original_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppTab {
    SessionChanger,
    AccountManager,
}

impl Default for AppTab {
    fn default() -> Self {
        AppTab::SessionChanger
    }
}

impl PayloadContext {
    pub fn add_event(&mut self, event: Event) {
        self.input_events.push(event);
    }

    pub fn render(&mut self, hdc: HDC) -> Result<(), String> {
        crate::graphics::renderer::render_frame(self, hdc)
    }
}