use crate::graphics::renderer::UiState;
use crate::jvm::get_minecraft_session;
use egui::Color32;

pub fn render_session_tab(ui: &mut egui::Ui, ui_state: &mut UiState) {
    render_current_session_section(ui, ui_state);
    render_session_actions(ui, ui_state);
    ui.separator();
    render_change_session_section(ui, ui_state);
    render_status_and_tips(ui, ui_state);
}

fn render_current_session_section(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.heading("Current Session");
    ui.separator();

    let session_manager = get_minecraft_session();
    let current_session = session_manager.get_current_session();

    ui.horizontal(|ui| {
        ui.label("Username:");
        ui.colored_label(Color32::LIGHT_BLUE, &current_session.username);
        if ui.button("📋").on_hover_text("Copy username").clicked() {
            if ui_state.clipboard.set_text(&current_session.username) {
                *ui_state.status_message = "✅ Username copied to clipboard".to_string();
            } else {
                *ui_state.status_message = "❌ Failed to copy username".to_string();
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Player ID:");
        ui.colored_label(Color32::LIGHT_GREEN, &current_session.player_id);
        if ui.button("📋").on_hover_text("Copy player ID").clicked() {
            if ui_state.clipboard.set_text(&current_session.player_id) {
                *ui_state.status_message = "✅ Player ID copied to clipboard".to_string();
            } else {
                *ui_state.status_message = "❌ Failed to copy player ID".to_string();
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
        if ui.button("📋").on_hover_text("Copy access token").clicked() {
            if ui_state.clipboard.set_text(&current_session.access_token) {
                *ui_state.status_message = "✅ Access token copied to clipboard".to_string();
            } else {
                *ui_state.status_message = "❌ Failed to copy access token".to_string();
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Session Type:");
        ui.colored_label(Color32::YELLOW, &current_session.session_type);
    });
}

fn render_session_actions(ui: &mut egui::Ui, ui_state: &mut UiState) {
    let session_manager = get_minecraft_session();

    if ui.button("🔄 Refresh Session").clicked() {
        match session_manager.refresh_session() {
            Ok(_) => *ui_state.status_message = "✅ Session refreshed successfully".to_string(),
            Err(e) => *ui_state.status_message = format!("❌ Failed to refresh: {}", e),
        }
    }
}

fn render_change_session_section(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.heading("Change Session");

    ui.horizontal(|ui| {
        ui.label("New Username:");
        ui.text_edit_singleline(ui_state.new_username);
    });

    ui.horizontal(|ui| {
        ui.label("New Player ID:");
        ui.text_edit_singleline(ui_state.new_player_id);
    });

    ui.horizontal(|ui| {
        ui.label("Access Token:");
        ui.text_edit_singleline(ui_state.new_access_token);
    });

    ui.horizontal(|ui| {
        ui.label("Session Type:");
        egui::ComboBox::from_label("")
            .selected_text(ui_state.new_session_type.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(ui_state.new_session_type, "mojang".to_string(), "Mojang");
                ui.selectable_value(ui_state.new_session_type, "legacy".to_string(), "Legacy");
            });
    });

    render_session_buttons(ui, ui_state);
}

fn render_session_buttons(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.horizontal(|ui| {
        let session_manager = get_minecraft_session();
        let current_session = session_manager.get_current_session();

        if ui.button("📋 Copy Current").clicked() {
            *ui_state.new_username = current_session.username.clone();
            *ui_state.new_player_id = current_session.player_id.clone();
            *ui_state.new_access_token = current_session.access_token.clone();
            *ui_state.new_session_type = current_session.session_type.clone();
        }

        if ui.button("🔄 Apply Changes").clicked() {
            use crate::jvm::SessionInfo;
            let new_session = SessionInfo {
                username: ui_state.new_username.clone(),
                player_id: ui_state.new_player_id.clone(),
                access_token: ui_state.new_access_token.clone(),
                session_type: ui_state.new_session_type.clone(),
            };

            match session_manager.change_session(new_session) {
                Ok(_) => *ui_state.status_message = "✅ Session changed successfully".to_string(),
                Err(e) => *ui_state.status_message = format!("❌ Failed to change session: {}", e),
            }
        }

        if ui.button("🗑️ Clear Fields").clicked() {
            ui_state.new_username.clear();
            ui_state.new_player_id.clear();
            ui_state.new_access_token.clear();
            *ui_state.new_session_type = "legacy".to_string();
        }
    });
}

fn render_status_and_tips(ui: &mut egui::Ui, ui_state: &mut UiState) {
    if !ui_state.status_message.is_empty() {
        ui.label(ui_state.status_message.as_str());
    }

    ui.separator();
    ui.label("💡 Tips:");
    ui.label("• Use 'Refresh Session' to load current game session");
    ui.label("• Player ID should be in UUID format");
    ui.label("• Press INSERT to toggle this menu");
    ui.label("• Changes apply immediately to the game");
    ui.label("• Use Ctrl+C/Ctrl+V to copy/paste text");
    ui.label("• Click 📋 buttons to copy values to clipboard");
}