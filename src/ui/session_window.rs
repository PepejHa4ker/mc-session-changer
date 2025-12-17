use crate::{
    graphics::icon_renderer::{render_clickable_icon_with_text, render_decorative_icon},
    graphics::svg_icons::SvgIconManager,
    jvm::{get_jvm, SessionInfo},
    ui::UiState,
};
use egui::{Color32, RichText, TextEdit, Ui};

pub fn render_session_tab(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    ui.vertical(|ui| {
        render_current_session_section(ui_state, icon_manager, ui);
        render_change_session_section(ui_state, icon_manager, ui);
    });
}

fn render_current_session_section(
    ui_state: &mut UiState,
    icon_manager: &mut SvgIconManager,
    ui: &mut Ui,
) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {

                render_decorative_icon(icon_manager, ui, "session_active", Color32::LIGHT_BLUE, Some(16));
                ui.label(
                    RichText::new("Current Session")
                        .size(16.0)
                        .color(Color32::LIGHT_BLUE),
                );
            });

            ui.separator();

            let session_manager = get_jvm();
            let current_session = session_manager.get_current_session();

            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.colored_label(Color32::LIGHT_BLUE, &current_session.username);

                if render_clickable_icon_with_text(
                    icon_manager,
                    ui,
                    "copy",
                    "Copy",
                    Color32::LIGHT_GREEN,
                    Some(16),
                    "Copy username to clipboard"
                ).clicked() {
                    if ui_state.clipboard.set_text(&current_session.username) {
                        ui_state.notification_manager.show_success("Copied", "Username copied to clipboard");
                    } else {
                        ui_state.notification_manager.show_error("Copy Failed", "Failed to copy username");
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Player ID:");
                ui.colored_label(Color32::LIGHT_GREEN, &current_session.player_id);

                if render_clickable_icon_with_text(
                    icon_manager,
                    ui,
                    "copy",
                    "Copy",
                    Color32::LIGHT_GREEN,
                    Some(16),
                    "Copy player ID to clipboard"
                ).clicked() {
                    if ui_state.clipboard.set_text(&current_session.player_id) {
                        ui_state.notification_manager.show_success("Copied", "Player ID copied to clipboard");
                    } else {
                        ui_state.notification_manager.show_error("Copy Failed", "Failed to copy player ID");
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Access Token:");
                let token_display = if current_session.access_token.len() > 16 {
                    format!("{}...", &current_session.access_token[..16])
                } else {
                    current_session.access_token.clone()
                };
                ui.colored_label(Color32::GRAY, token_display);

                if render_clickable_icon_with_text(
                    icon_manager,
                    ui,
                    "copy",
                    "Copy",
                    Color32::LIGHT_GREEN,
                    Some(16),
                    "Copy access token to clipboard"
                ).clicked() {
                    if ui_state.clipboard.set_text(&current_session.access_token) {
                        ui_state.notification_manager.show_success("Copied", "Access token copied to clipboard");
                    } else {
                        ui_state.notification_manager.show_error("Copy Failed", "Failed to copy access token");
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Session Type:");
                ui.colored_label(Color32::YELLOW, &current_session.session_type);
            });

            render_session_actions(ui_state, icon_manager, ui);
        });
    });
}

fn render_session_actions(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    ui.horizontal(|ui| {

        if render_clickable_icon_with_text(
            icon_manager,
            ui,
            "refresh",
            "Refresh Session",
            Color32::YELLOW,
            Some(16),
            "Refresh current session from game"
        ).clicked() {
            let session_manager = get_jvm();
            match session_manager.refresh_session() {
                Ok(_) => {
                    ui_state.notification_manager.show_success("Session Refreshed", "Session refreshed successfully");
                }
                Err(e) => {
                    ui_state.notification_manager.show_error("Refresh Failed", &format!("Failed to refresh: {}", e));
                }
            }
        }
    });
}

fn render_change_session_section(
    ui_state: &mut UiState,
    icon_manager: &mut SvgIconManager,
    ui: &mut Ui,
) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {

                render_decorative_icon(icon_manager, ui, "session_changer", Color32::LIGHT_GREEN, Some(16));
                ui.label(
                    RichText::new("Change Session")
                        .size(16.0)
                        .color(Color32::LIGHT_GREEN),
                );
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("New Username:");
                ui.add(TextEdit::singleline(ui_state.new_username).desired_width(200.0));
            });

            ui.horizontal(|ui| {
                ui.label("New Player ID:");
                ui.add(TextEdit::singleline(ui_state.new_player_id).desired_width(200.0));
            });

            ui.horizontal(|ui| {
                ui.label("New Access Token:");
                ui.add(TextEdit::singleline(ui_state.new_access_token).desired_width(200.0));
            });

            ui.horizontal(|ui| {
                ui.label("New Session Type:");
                egui::ComboBox::from_label("")
                    .selected_text(ui_state.new_session_type.as_str())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(ui_state.new_session_type, "mojang".to_string(), "Mojang");
                        ui.selectable_value(ui_state.new_session_type, "legacy".to_string(), "Legacy");
                    });
            });

            render_session_buttons(ui_state, icon_manager, ui);
        });
    });
}

fn render_session_buttons(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let session_manager = get_jvm();
        let current_session = session_manager.get_current_session();

        if render_clickable_icon_with_text(
            icon_manager,
            ui,
            "copy",
            "Copy Current",
            Color32::LIGHT_BLUE,
            Some(16),
            "Copy current session data to input fields"
        ).clicked() {
            *ui_state.new_username = current_session.username.clone();
            *ui_state.new_player_id = current_session.player_id.clone();
            *ui_state.new_access_token = current_session.access_token.clone();
            *ui_state.new_session_type = current_session.session_type.clone();
            ui_state.notification_manager.show_info("Fields Filled", "Current session data copied to input fields");
        }

        ui.separator();

        if render_clickable_icon_with_text(
            icon_manager,
            ui,
            "apply",
            "Apply Changes",
            Color32::LIGHT_GREEN,
            Some(16),
            "Apply new session data to game"
        ).clicked() {
            let new_session = SessionInfo {
                username: ui_state.new_username.clone(),
                player_id: ui_state.new_player_id.clone(),
                access_token: ui_state.new_access_token.clone(),
                session_type: ui_state.new_session_type.clone(),
            };

            match session_manager.change_session(new_session) {
                Ok(_) => {
                    ui_state.notification_manager.show_success("Session Changed", "Session changed successfully");
                }
                Err(e) => {
                    ui_state.notification_manager.show_error("Change Failed", &format!("Failed to change session: {}", e));
                }
            }
        }

        ui.separator();

        if render_clickable_icon_with_text(
            icon_manager,
            ui,
            "clear",
            "Clear Fields",
            Color32::YELLOW,
            Some(16),
            "Clear all input fields"
        ).clicked() {
            ui_state.new_username.clear();
            ui_state.new_player_id.clear();
            ui_state.new_access_token.clear();
            *ui_state.new_session_type = "legacy".to_string();
            ui_state.notification_manager.show_info("Fields Cleared", "All input fields cleared");
        }
    });
}
