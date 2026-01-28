use crate::input::clipboard::ClipboardManager;
use crate::utils::SafeGLContext;
use egui::{Context, Event};
use egui_glow::Painter;
use glow::Context as GlowContext;
use std::time::Instant;
use winapi::shared::windef::HDC;
use crate::graphics::svg_icons::SvgIconManager;
use crate::ui::notification_manager::NotificationManager;
use crate::graphics::netlog::PacketDirection;

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
    pub manual_password: String,

    pub show_edit_dialog: bool,
    pub edit_account_name: String,
    pub edit_username: String,
    pub edit_player_id: String,
    pub edit_access_token: String,
    pub edit_session_type: String,
    pub edit_password: String,
    pub edit_original_name: String,
    pub auth_in_progress: bool,

    pub packet_filter: String,
    pub packet_show_inbound: bool,
    pub packet_show_outbound: bool,
    pub packet_autoscroll: bool,
    pub packet_paused: bool,
    pub packets_detached: bool,
    pub packets_window_open: bool,
    pub selected_packet_id: Option<u64>,
    pub packet_only_pinned: bool,
    pub packet_limit_count: u32,
    pub packet_autoclear_oldest: bool,
    pub packet_filter_profiles: Vec<PacketFilterProfile>,
    pub packet_profile_new_name: String,
    pub packet_profile_new_query: String,
    pub packet_show_only_new: bool,
    pub packet_last_seen_id: u64,
    pub packet_secondary_selected_id: Option<u64>,
    pub packet_triggers: Vec<PacketTrigger>,
    pub packet_trigger_input: String,
    pub packet_tag_editor: String,
    pub packet_color_hex: String,
    pub packet_export_limit: u32,
    pub packet_import_buffer: String,

    pub search_query: String,
    pub selected_class: Option<String>,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub detached: bool,
    pub window_open: bool,
    pub expand_fields: bool,
    pub expand_methods: bool,
    pub member_search_query: String,

}

#[derive(Debug, Clone, PartialEq)]
pub enum AppTab {
    SessionChanger,
    AccountManager,
    PacketAnalyzer,
    JvmAnalyzer,
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

#[derive(Clone, Debug)]
pub struct PacketFilterProfile {
    pub name: String,
    pub query: String,
    pub show_inbound: bool,
    pub show_outbound: bool,
    pub only_pinned: bool,
}

#[derive(Clone, Debug)]
pub struct PacketTrigger {
    pub name: String,
    pub needle: String,
    pub dir: Option<PacketDirection>,
    pub highlight: [u8; 3],
    pub pin: bool,
}
