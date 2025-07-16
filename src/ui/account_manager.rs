use egui::{Color32, ScrollArea, Order};
use crate::graphics::renderer::UiState;
use crate::core::state::GlobalState;
use crate::jvm::{get_minecraft_session, SessionInfo};

pub fn render_account_manager_tab(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.horizontal(|ui| {
        ui.heading("Account Manager");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(manager) = GlobalState::get_account_manager().get() {
                let count = manager.lock().get_account_count();
                ui.label(format!("📊 {} accounts", count));
            }
        });
    });


    render_current_account_section(ui, ui_state);
    render_add_account_section(ui, ui_state);
    render_accounts_list(ui, ui_state);
    render_account_status(ui, ui_state);

    render_manual_input_dialog(ui, ui_state);
    render_edit_account_dialog(ui, ui_state);
}

fn render_current_account_section(ui: &mut egui::Ui, _ui_state: &mut UiState) {
    ui.heading("Current Account");

    let session_manager = get_minecraft_session();
    let current_session = session_manager.get_current_session();

    ui.horizontal(|ui| {
        ui.label("👤");
        ui.colored_label(Color32::LIGHT_BLUE, &current_session.username);
        ui.separator();
        ui.label("🆔");
        ui.colored_label(Color32::LIGHT_GREEN, &current_session.player_id[..8]);
        ui.label("...");
        ui.separator();
        ui.label("🔑");
        ui.colored_label(Color32::YELLOW, &current_session.session_type);
    });
}

fn render_add_account_section(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.heading("Add Account");

    ui.horizontal(|ui| {
        ui.label("Account Name:");
        ui.text_edit_singleline(ui_state.account_name_input);
    });

    ui.horizontal(|ui| {
        if ui.button("📥 Add from Current Session").clicked() {
            add_account_from_current_session(ui_state);
        }

        if ui.button("✏️ Add Manually").clicked() {
            *ui_state.show_manual_input_dialog = true;
            ui_state.manual_account_name.clear();
            ui_state.manual_username.clear();
            ui_state.manual_player_id.clear();
            ui_state.manual_access_token.clear();
            *ui_state.manual_session_type = "mojang".to_string();
        }
    });
}

fn render_manual_input_dialog(ui: &mut egui::Ui, ui_state: &mut UiState) {
    if !*ui_state.show_manual_input_dialog {
        return;
    }

    egui::Window::new("✏️ Add Account Manually")
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .order(Order::Foreground)
        .current_pos(egui::pos2(
            ui.ctx().screen_rect().center().x - 200.0,
            ui.ctx().screen_rect().center().y - 150.0
        ))
        .default_size([400.0, 300.0])
        .show(ui.ctx(), |ui| {
            ui.ctx().layer_painter(egui::LayerId::background())
                .rect_filled(
                    ui.ctx().screen_rect(),
                    0.0,
                    Color32::from_black_alpha(128)
                );

            ui.vertical_centered(|ui| {
                ui.heading("Manual Account Entry");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Account Name:");
                    ui.text_edit_singleline(ui_state.manual_account_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(ui_state.manual_username);
                });

                ui.horizontal(|ui| {
                    ui.label("Player ID:");
                    ui.text_edit_singleline(ui_state.manual_player_id);
                });

                ui.horizontal(|ui| {
                    ui.label("Access Token:");
                    ui.text_edit_singleline(ui_state.manual_access_token);
                });

                ui.horizontal(|ui| {
                    ui.label("Session Type:");
                    egui::ComboBox::from_label("")
                        .selected_text(ui_state.manual_session_type.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(ui_state.manual_session_type, "mojang".to_string(), "Mojang");
                            ui.selectable_value(ui_state.manual_session_type, "legacy".to_string(), "Legacy");
                        });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("💾 Save Account").clicked() {
                        if save_manual_account(ui_state) {
                            *ui_state.show_manual_input_dialog = false;
                        }
                    }

                    if ui.button("❌ Cancel").clicked() {
                        *ui_state.show_manual_input_dialog = false;
                    }
                });
            });
        });
}

fn render_edit_account_dialog(ui: &mut egui::Ui, ui_state: &mut UiState) {
    if !*ui_state.show_edit_dialog {
        return;
    }

    egui::Window::new("✏️ Edit Account")
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .order(Order::Foreground)
        .current_pos(egui::pos2(
            ui.ctx().screen_rect().center().x - 200.0,
            ui.ctx().screen_rect().center().y - 150.0
        ))
        .default_size([400.0, 300.0])
        .show(ui.ctx(), |ui| {
            ui.ctx().layer_painter(egui::LayerId::background())
                .rect_filled(
                    ui.ctx().screen_rect(),
                    0.0,
                    Color32::from_black_alpha(128)
                );

            ui.vertical_centered(|ui| {
                ui.heading("Edit Account");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Account Name:");
                    ui.text_edit_singleline(ui_state.edit_account_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(ui_state.edit_username);
                });

                ui.horizontal(|ui| {
                    ui.label("Player ID:");
                    ui.text_edit_singleline(ui_state.edit_player_id);
                });

                ui.horizontal(|ui| {
                    ui.label("Access Token:");
                    ui.text_edit_singleline(ui_state.edit_access_token);
                });

                ui.horizontal(|ui| {
                    ui.label("Session Type:");
                    egui::ComboBox::from_label("")
                        .selected_text(ui_state.edit_session_type.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(ui_state.edit_session_type, "mojang".to_string(), "Mojang");
                            ui.selectable_value(ui_state.edit_session_type, "legacy".to_string(), "Legacy");
                        });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("💾 Save Changes").clicked() {
                        if save_edited_account(ui_state) {
                            *ui_state.show_edit_dialog = false;
                        }
                    }

                    if ui.button("❌ Cancel").clicked() {
                        *ui_state.show_edit_dialog = false;
                    }
                });

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    *ui_state.show_edit_dialog = false;
                }
            });
        });
}

fn save_manual_account(ui_state: &mut UiState) -> bool {
    if ui_state.manual_account_name.is_empty() {
        *ui_state.account_status_message = "❌ Please enter an account name".to_string();
        return false;
    }

    if ui_state.manual_username.is_empty() ||
        ui_state.manual_player_id.is_empty() ||
        ui_state.manual_access_token.is_empty() {
        *ui_state.account_status_message = "❌ All fields are required".to_string();
        return false;
    }

    let session_info = SessionInfo {
        username: ui_state.manual_username.clone(),
        player_id: ui_state.manual_player_id.clone(),
        access_token: ui_state.manual_access_token.clone(),
        session_type: ui_state.manual_session_type.clone(),
    };

    if let Some(manager_mutex) = GlobalState::get_account_manager().get() {
        let mut manager = manager_mutex.lock();

        match manager.add_account(ui_state.manual_account_name.clone(), session_info) {
            Ok(_) => {
                *ui_state.account_status_message = format!("✅ Account '{}' added successfully", ui_state.manual_account_name);
                true
            }
            Err(e) => {
                *ui_state.account_status_message = format!("❌ Failed to add account: {}", e);
                false
            }
        }
    } else {
        *ui_state.account_status_message = "❌ Account manager not available".to_string();
        false
    }
}

fn save_edited_account(ui_state: &mut UiState) -> bool {
    if ui_state.edit_account_name.is_empty() {
        *ui_state.account_status_message = "❌ Please enter an account name".to_string();
        return false;
    }

    if ui_state.edit_username.is_empty() ||
        ui_state.edit_player_id.is_empty() ||
        ui_state.edit_access_token.is_empty() {
        *ui_state.account_status_message = "❌ All fields are required".to_string();
        return false;
    }

    let session_info = SessionInfo {
        username: ui_state.edit_username.clone(),
        player_id: ui_state.edit_player_id.clone(),
        access_token: ui_state.edit_access_token.clone(),
        session_type: ui_state.edit_session_type.clone(),
    };

    if let Some(manager_mutex) = GlobalState::get_account_manager().get() {
        let mut manager = manager_mutex.lock();

        if ui_state.edit_account_name != ui_state.edit_original_name {
            if let Err(e) = manager.rename_account(&ui_state.edit_original_name, ui_state.edit_account_name.clone()) {
                *ui_state.account_status_message = format!("❌ Failed to rename account: {}", e);
                return false;
            }
        }

        match manager.update_account(&ui_state.edit_account_name, session_info) {
            Ok(_) => {
                *ui_state.account_status_message = format!("✅ Account '{}' updated successfully", ui_state.edit_account_name);
                true
            }
            Err(e) => {
                *ui_state.account_status_message = format!("❌ Failed to update account: {}", e);
                false
            }
        }
    } else {
        *ui_state.account_status_message = "❌ Account manager not available".to_string();
        false
    }
}

fn render_accounts_list(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.heading("Saved Accounts");

    if let Some(manager_mutex) = GlobalState::get_account_manager().get() {
        ui.separator();
        let accounts = {
            let manager = manager_mutex.lock();
            manager.get_all_accounts()
        };

        if accounts.is_empty() {
            ui.label("No accounts saved yet.");
            return;
        }

        let mut account_to_delete: Option<String> = None;
        let mut account_to_use: Option<String> = None;
        let mut account_to_copy: Option<String> = None;
        let mut account_to_edit: Option<String> = None;

        ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for account in &accounts {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("👤");
                                ui.colored_label(Color32::LIGHT_BLUE, &account.username);
                                ui.separator();
                                ui.label("📛");
                                ui.colored_label(Color32::WHITE, &account.name);
                            });

                            ui.horizontal(|ui| {
                                ui.label("🕒 Created:");
                                ui.colored_label(Color32::GRAY, account.format_created_date());
                                ui.separator();
                                ui.label("🕐 Last used:");
                                ui.colored_label(Color32::GRAY, account.format_last_used());
                            });
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑️").on_hover_text("Delete account").clicked() {
                                account_to_delete = Some(account.name.clone());
                            }

                            if ui.button("✏️").on_hover_text("Edit account").clicked() {
                                account_to_edit = Some(account.name.clone());
                            }

                            if ui.button("🔄").on_hover_text("Use this account").clicked() {
                                account_to_use = Some(account.name.clone());
                            }

                            if ui.button("📋").on_hover_text("Copy to clipboard").clicked() {
                                account_to_copy = Some(account.name.clone());
                            }
                        });
                    });
                }
            });

        if let Some(account_name) = account_to_delete {
            delete_account(&account_name, ui_state);
        }
        if let Some(account_name) = account_to_use {
            use_account(&account_name, ui_state);
        }
        if let Some(account_name) = account_to_copy {
            copy_account_to_clipboard(&account_name, ui_state);
        }
        if let Some(account_name) = account_to_edit {
            edit_account(&account_name, ui_state);
        }
    }
}

fn edit_account(account_name: &str, ui_state: &mut UiState) {
    if let Some(manager_mutex) = GlobalState::get_account_manager().get() {
        let manager = manager_mutex.lock();

        if let Some(account) = manager.get_account(account_name) {
            *ui_state.edit_account_name = account.name.clone();
            *ui_state.edit_username = account.username.clone();
            *ui_state.edit_player_id = account.player_id.clone();
            *ui_state.edit_access_token = account.access_token.clone();
            *ui_state.edit_session_type = account.session_type.clone();
            *ui_state.edit_original_name = account.name.clone();

            *ui_state.show_edit_dialog = true;
        } else {
            *ui_state.account_status_message = format!("❌ Account '{}' not found", account_name);
        }
    }
}

fn render_account_status(ui: &mut egui::Ui, ui_state: &mut UiState) {
    if !ui_state.account_status_message.is_empty() {
        ui.separator();
        let color = if ui_state.account_status_message.starts_with("✅") {
            Color32::GREEN
        } else if ui_state.account_status_message.starts_with("❌") {
            Color32::RED
        } else if ui_state.account_status_message.starts_with("🔄") {
            Color32::YELLOW
        } else {
            Color32::WHITE
        };

        ui.colored_label(color, ui_state.account_status_message.as_str());
    }
}

fn add_account_from_current_session(ui_state: &mut UiState) {
    if ui_state.account_name_input.is_empty() {
        *ui_state.account_status_message = "❌ Please enter an account name".to_string();
        return;
    }

    let session_manager = get_minecraft_session();
    let current_session = session_manager.get_current_session();

    if let Some(manager_mutex) = GlobalState::get_account_manager().get() {
        let mut manager = manager_mutex.lock();

        match manager.add_account(ui_state.account_name_input.clone(), current_session) {
            Ok(_) => {
                *ui_state.account_status_message = format!("✅ Account '{}' added successfully", ui_state.account_name_input);
                ui_state.account_name_input.clear();
            }
            Err(e) => {
                *ui_state.account_status_message = format!("❌ Failed to add account: {}", e);
            }
        }
    }
}

fn use_account(account_name: &str, ui_state: &mut UiState) {
    if let Some(manager_mutex) = GlobalState::get_account_manager().get() {
        let session_info = {
            let mut manager = manager_mutex.lock();
            manager.use_account(account_name)
        };

        match session_info {
            Ok(session_info) => {
                let session_manager = get_minecraft_session();
                match session_manager.change_session(session_info) {
                    Ok(_) => {
                        *ui_state.account_status_message = format!("✅ Switched to account '{}'", account_name);
                        *ui_state.selected_account = Some(account_name.to_string());
                    }
                    Err(e) => {
                        *ui_state.account_status_message = format!("❌ Failed to switch to account: {}", e);
                    }
                }
            }
            Err(e) => {
                *ui_state.account_status_message = format!("❌ Failed to load account: {}", e);
            }
        }
    }
}

fn delete_account(account_name: &str, ui_state: &mut UiState) {
    if let Some(manager_mutex) = GlobalState::get_account_manager().get() {
        let result = {
            let mut manager = manager_mutex.lock();
            manager.remove_account(account_name)
        };

        match result {
            Ok(_) => {
                *ui_state.account_status_message = format!("✅ Account '{}' deleted successfully", account_name);
                if ui_state.selected_account.as_deref() == Some(account_name) {
                    *ui_state.selected_account = None;
                }
            }
            Err(e) => {
                *ui_state.account_status_message = format!("❌ Failed to delete account: {}", e);
            }
        }
    }
}

fn copy_account_to_clipboard(account_name: &str, ui_state: &mut UiState) {
    if let Some(manager_mutex) = GlobalState::get_account_manager().get() {
        let json_result = {
            let manager = manager_mutex.lock();
            manager.export_to_clipboard(account_name)
        };

        match json_result {
            Ok(json) => {
                if ui_state.clipboard.set_text(&json) {
                    *ui_state.account_status_message = format!("✅ Account '{}' copied to clipboard", account_name);
                } else {
                    *ui_state.account_status_message = "❌ Failed to copy to clipboard".to_string();
                }
            }
            Err(e) => {
                *ui_state.account_status_message = format!("❌ Failed to export account: {}", e);
            }
        }
    }
}