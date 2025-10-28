use crate::{
    graphics::renderer::UiState,
    graphics::icon_renderer::{render_clickable_icon_with_text, render_decorative_icon},
    core::state::GlobalState,
    account::StoredAccount,
    graphics::svg_icons::SvgIconManager,
    jvm::{get_jvm, SessionInfo}
};
use egui::{Color32, Order, RichText, ScrollArea, TextEdit, Ui, Vec2};

pub fn render_account_manager_tab(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    ui.vertical(|ui| {
        render_header_section(icon_manager, ui);
        render_current_account_section(icon_manager, ui);
        render_add_account_section(ui_state, icon_manager, ui);
        render_accounts_list(ui_state, icon_manager, ui);
    });

    render_manual_input_dialog(ui_state, icon_manager, ui);
    render_edit_account_dialog(ui_state, icon_manager, ui);
}

fn render_header_section(icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    ui.horizontal(|ui| {

        render_decorative_icon(icon_manager, ui, "account", Color32::LIGHT_GREEN, Some(16));
        ui.label(RichText::new("Account Manager").size(18.0).color(Color32::LIGHT_GREEN));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(manager) = GlobalState::instance().get_account_manager().get() {
                let count = manager.lock().get_account_count();
                ui.horizontal(|ui| {

                    render_decorative_icon(icon_manager, ui, "counter", Color32::LIGHT_BLUE, Some(16));
                    ui.label(format!("{} accounts", count));
                });
            }
        });
    });
}

fn render_current_account_section(icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {

                render_decorative_icon(icon_manager, ui, "session_active", Color32::LIGHT_BLUE, Some(16));
                ui.label(RichText::new("Current Account").size(16.0).color(Color32::LIGHT_BLUE));
            });

            ui.separator();

            let session_manager = get_jvm();
            let current_session = session_manager.get_current_session();

            ui.horizontal(|ui| {

                render_decorative_icon(icon_manager, ui, "user", Color32::LIGHT_BLUE, Some(16));
                ui.colored_label(Color32::LIGHT_BLUE, &current_session.username);
                ui.separator();
                render_decorative_icon(icon_manager, ui, "id", Color32::LIGHT_GREEN, Some(16));
                ui.colored_label(Color32::LIGHT_GREEN, &current_session.player_id[..8.min(current_session.player_id.len())]);
                ui.label("...");
                ui.separator();
                render_decorative_icon(icon_manager, ui, "key", Color32::YELLOW, Some(16));
                ui.colored_label(Color32::YELLOW, &current_session.session_type);
            });
        });
    });
}

fn render_add_account_section(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {

                render_decorative_icon(icon_manager, ui, "account_add", Color32::LIGHT_GREEN, Some(16));
                ui.label(RichText::new("Add Account").size(16.0).color(Color32::LIGHT_GREEN));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Account Name:");
                ui.add(TextEdit::singleline(ui_state.account_name_input).desired_width(200.0));
            });

            ui.horizontal(|ui| {

                if render_clickable_icon_with_text(
                    icon_manager,
                    ui,
                    "add",
                    "Add from Current Session",
                    Color32::LIGHT_GREEN,
                    Some(16),
                    "Add account using current session data"
                ).clicked() {
                    add_account_from_current_session(ui_state);
                }

                ui.separator();

                if render_clickable_icon_with_text(
                    icon_manager,
                    ui,
                    "edit",
                    "Add Manually",
                    Color32::YELLOW,
                    Some(16),
                    "Add account with manual input"
                ).clicked() {
                    *ui_state.show_manual_input_dialog = true;
                    ui_state.manual_account_name.clear();
                    ui_state.manual_username.clear();
                    ui_state.manual_player_id.clear();
                    ui_state.manual_access_token.clear();
                    *ui_state.manual_session_type = "mojang".to_string();
                }
            });
        });
    });
}

fn render_manual_input_dialog(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    if !*ui_state.show_manual_input_dialog {
        return;
    }

    egui::Window::new("Add Account Manually")
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
                ui.horizontal(|ui| {

                    render_decorative_icon(icon_manager, ui, "edit", Color32::YELLOW, Some(16));
                    ui.heading("Manual Account Entry");
                });
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

                    if render_clickable_icon_with_text(
                        icon_manager,
                        ui,
                        "save",
                        "Save Account",
                        Color32::LIGHT_GREEN,
                        Some(16),
                        "Save the account with entered data"
                    ).clicked() {
                        if save_manual_account(ui_state) {
                            *ui_state.show_manual_input_dialog = false;
                        }
                    }

                    ui.separator();

                    if render_clickable_icon_with_text(
                        icon_manager,
                        ui,
                        "cancel",
                        "Cancel",
                        Color32::RED,
                        Some(16),
                        "Cancel and close dialog"
                    ).clicked() {
                        *ui_state.show_manual_input_dialog = false;
                    }
                });
            });
        });
}

fn render_edit_account_dialog(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    if !*ui_state.show_edit_dialog {
        return;
    }

    egui::Window::new("Edit Account")
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
                ui.horizontal(|ui| {

                    render_decorative_icon(icon_manager, ui, "edit", Color32::YELLOW, Some(16));
                    ui.heading("Edit Account");
                });
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

                    if render_clickable_icon_with_text(
                        icon_manager,
                        ui,
                        "save",
                        "Save Changes",
                        Color32::LIGHT_GREEN,
                        Some(16),
                        "Save changes to account"
                    ).clicked() {
                        if save_edited_account(ui_state) {
                            *ui_state.show_edit_dialog = false;
                        }
                    }

                    ui.separator();

                    if render_clickable_icon_with_text(
                        icon_manager,
                        ui,
                        "cancel",
                        "Cancel",
                        Color32::RED,
                        Some(16),
                        "Cancel editing and close dialog"
                    ).clicked() {
                        *ui_state.show_edit_dialog = false;
                    }
                });

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    *ui_state.show_edit_dialog = false;
                }
            });
        });
}

fn render_account_info_aligned(
    icon_manager: &mut SvgIconManager,
    ui: &mut Ui,
    account: &StoredAccount,
) {

    let username_col_width = 180.0;
    let account_name_col_width = 120.0;
    let player_id_col_width = 100.0;
    let session_type_col_width = 80.0;
    let date_col_width = 140.0;
    let row_height = 20.0;

    ui.vertical(|ui| {

        ui.horizontal(|ui| {

            ui.allocate_ui_with_layout(
                Vec2::new(username_col_width, row_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    render_decorative_icon(icon_manager, ui, "user", Color32::LIGHT_BLUE, Some(16));
                    ui.add_space(4.0);

                    let display_username = if account.username.len() > 20 {
                        format!("{}...", &account.username[..17])
                    } else {
                        account.username.clone()
                    };

                    ui.colored_label(Color32::LIGHT_BLUE, display_username)
                        .on_hover_text(&account.username);
                }
            );

            ui.allocate_ui_with_layout(
                Vec2::new(account_name_col_width, row_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    render_decorative_icon(icon_manager, ui, "tag", Color32::WHITE, Some(16));
                    ui.add_space(4.0);

                    let display_name = if account.name.len() > 14 {
                        format!("{}...", &account.name[..11])
                    } else {
                        account.name.clone()
                    };

                    ui.colored_label(Color32::WHITE, display_name)
                        .on_hover_text(&account.name);
                }
            );

            ui.allocate_ui_with_layout(
                Vec2::new(player_id_col_width, row_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    render_decorative_icon(icon_manager, ui, "id", Color32::LIGHT_GREEN, Some(16));
                    ui.add_space(4.0);

                    let player_id_short = if account.player_id.len() > 8 {
                        format!("{}...", &account.player_id[..8])
                    } else {
                        account.player_id.clone()
                    };

                    ui.colored_label(Color32::LIGHT_GREEN, player_id_short)
                        .on_hover_text(&account.player_id);
                }
            );

            ui.allocate_ui_with_layout(
                Vec2::new(session_type_col_width, row_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    render_decorative_icon(icon_manager, ui, "key", Color32::YELLOW, Some(16));
                    ui.add_space(4.0);
                    ui.colored_label(Color32::YELLOW, &account.session_type);
                }
            );
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {

            ui.allocate_ui_with_layout(
                Vec2::new(date_col_width, row_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    render_decorative_icon(icon_manager, ui, "time", Color32::GRAY, Some(14));
                    ui.add_space(4.0);
                    ui.colored_label(Color32::GRAY, &format!("Added: {}", account.format_created_date()));
                }
            );

            ui.add_space(20.0);

            ui.allocate_ui_with_layout(
                Vec2::new(date_col_width, row_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    render_decorative_icon(icon_manager, ui, "time", Color32::GRAY, Some(14));
                    ui.add_space(4.0);
                    ui.colored_label(Color32::GRAY, &format!("Used: {}", account.format_last_used()));
                }
            );
        });
    });
}

fn render_accounts_list(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {

                render_decorative_icon(icon_manager, ui, "account", Color32::LIGHT_BLUE, Some(16));
                ui.label(RichText::new("Saved Accounts").size(16.0).color(Color32::LIGHT_BLUE));
            });

            ui.separator();

            if let Some(manager_mutex) = GlobalState::instance().get_account_manager().get() {
                let accounts = {
                    let manager = manager_mutex.lock();
                    manager.get_all_accounts()
                };

                if accounts.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        render_decorative_icon(icon_manager, ui, "empty", Color32::GRAY, Some(24));
                        ui.add_space(8.0);
                        ui.colored_label(Color32::GRAY, "No accounts saved yet.");
                        ui.add_space(20.0);
                    });
                    return;
                }

                let mut account_to_delete: Option<String> = None;
                let mut account_to_use: Option<String> = None;
                let mut account_to_copy: Option<String> = None;
                let mut account_to_edit: Option<String> = None;

                ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for (index, account) in accounts.iter().enumerate() {

                            if index > 0 {
                                ui.add_space(8.0);
                            }

                            ui.group(|ui| {
                                ui.horizontal(|ui| {

                                    render_account_info_aligned(icon_manager, ui, account);

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                                        if render_clickable_icon_with_text(
                                            icon_manager,
                                            ui,
                                            "delete",
                                            "Delete",
                                            Color32::RED,
                                            Some(16),
                                            "Delete this account"
                                        ).clicked() {
                                            account_to_delete = Some(account.name.clone());
                                        }

                                        if render_clickable_icon_with_text(
                                            icon_manager,
                                            ui,
                                            "edit",
                                            "Edit",
                                            Color32::YELLOW,
                                            Some(16),
                                            "Edit this account"
                                        ).clicked() {
                                            account_to_edit = Some(account.name.clone());
                                        }

                                        if render_clickable_icon_with_text(
                                            icon_manager,
                                            ui,
                                            "refresh",
                                            "Use",
                                            Color32::LIGHT_GREEN,
                                            Some(16),
                                            "Switch to this account"
                                        ).clicked() {
                                            account_to_use = Some(account.name.clone());
                                        }

                                        if render_clickable_icon_with_text(
                                            icon_manager,
                                            ui,
                                            "copy",
                                            "Copy",
                                            Color32::LIGHT_BLUE,
                                            Some(16),
                                            "Copy account data to clipboard"
                                        ).clicked() {
                                            account_to_copy = Some(account.name.clone());
                                        }
                                    });
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
        });
    });
}

fn save_manual_account(ui_state: &mut UiState) -> bool {
    if ui_state.manual_account_name.is_empty() {
        ui_state.notification_manager.show_error("Validation Error", "Please enter an account name");
        return false;
    }

    if ui_state.manual_username.is_empty() ||
        ui_state.manual_player_id.is_empty() ||
        ui_state.manual_access_token.is_empty() {
        ui_state.notification_manager.show_error("Validation Error", "All fields are required");
        return false;
    }

    let session_info = SessionInfo {
        username: ui_state.manual_username.clone(),
        player_id: ui_state.manual_player_id.clone(),
        access_token: ui_state.manual_access_token.clone(),
        session_type: ui_state.manual_session_type.clone(),
    };

    if let Some(manager_mutex) = GlobalState::instance().get_account_manager().get() {
        let mut manager = manager_mutex.lock();

        match manager.add_account(ui_state.manual_account_name.clone(), session_info) {
            Ok(_) => {
                ui_state.notification_manager.show_success(
                    "Account Added",
                    &format!("Account '{}' added successfully", ui_state.manual_account_name)
                );
                true
            }
            Err(e) => {
                ui_state.notification_manager.show_error("Add Failed", &format!("Failed to add account: {}", e));
                false
            }
        }
    } else {
        ui_state.notification_manager.show_error("System Error", "Account manager not available");
        false
    }
}

fn save_edited_account(ui_state: &mut UiState) -> bool {
    if ui_state.edit_account_name.is_empty() {
        ui_state.notification_manager.show_error("Validation Error", "Please enter an account name");
        return false;
    }

    if ui_state.edit_username.is_empty() ||
        ui_state.edit_player_id.is_empty() ||
        ui_state.edit_access_token.is_empty() {
        ui_state.notification_manager.show_error("Validation Error", "All fields are required");
        return false;
    }

    let session_info = SessionInfo {
        username: ui_state.edit_username.clone(),
        player_id: ui_state.edit_player_id.clone(),
        access_token: ui_state.edit_access_token.clone(),
        session_type: ui_state.edit_session_type.clone(),
    };

    if let Some(manager_mutex) = GlobalState::instance().get_account_manager().get() {
        let mut manager = manager_mutex.lock();

        if ui_state.edit_account_name != ui_state.edit_original_name {
            if let Err(e) = manager.rename_account(&ui_state.edit_original_name, ui_state.edit_account_name.clone()) {
                ui_state.notification_manager.show_error("Rename Failed", &format!("Failed to rename account: {}", e));
                return false;
            }
        }

        match manager.update_account(&ui_state.edit_account_name, session_info) {
            Ok(_) => {
                ui_state.notification_manager.show_success(
                    "Account Updated",
                    &format!("Account '{}' updated successfully", ui_state.edit_account_name)
                );
                true
            }
            Err(e) => {
                ui_state.notification_manager.show_error("Update Failed", &format!("Failed to update account: {}", e));
                false
            }
        }
    } else {
        ui_state.notification_manager.show_error("System Error", "Account manager not available");
        false
    }
}

fn edit_account(account_name: &str, ui_state: &mut UiState) {
    if let Some(manager_mutex) = GlobalState::instance().get_account_manager().get() {
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
            ui_state.notification_manager.show_error("Account Not Found", &format!("Account '{}' not found", account_name));
        }
    }
}

fn add_account_from_current_session(ui_state: &mut UiState) {
    if ui_state.account_name_input.is_empty() {
        ui_state.notification_manager.show_error("Validation Error", "Please enter an account name");
        return;
    }

    let session_manager = get_jvm();
    let current_session = session_manager.get_current_session();

    if let Some(manager_mutex) = GlobalState::instance().get_account_manager().get() {
        let mut manager = manager_mutex.lock();

        match manager.add_account(ui_state.account_name_input.clone(), current_session) {
            Ok(_) => {
                ui_state.notification_manager.show_success(
                    "Account Added",
                    &format!("Account '{}' added successfully", ui_state.account_name_input)
                );
                ui_state.account_name_input.clear();
            }
            Err(e) => {
                ui_state.notification_manager.show_error("Add Failed", &format!("Failed to add account: {}", e));
            }
        }
    }
}

fn use_account(account_name: &str, ui_state: &mut UiState) {
    if let Some(manager_mutex) = GlobalState::instance().get_account_manager().get() {
        let session_info = {
            let mut manager = manager_mutex.lock();
            manager.use_account(account_name)
        };

        match session_info {
            Ok(session_info) => {
                let session_manager = get_jvm();
                match session_manager.change_session(session_info) {
                    Ok(_) => {
                        ui_state.notification_manager.show_success(
                            "Account Switched",
                            &format!("Switched to account '{}'", account_name)
                        );
                        *ui_state.selected_account = Some(account_name.to_string());
                    }
                    Err(e) => {
                        ui_state.notification_manager.show_error("Switch Failed", &format!("Failed to switch to account: {}", e));
                    }
                }
            }
            Err(e) => {
                ui_state.notification_manager.show_error("Load Failed", &format!("Failed to load account: {}", e));
            }
        }
    }
}

fn delete_account(account_name: &str, ui_state: &mut UiState) {
    if let Some(manager_mutex) = GlobalState::instance().get_account_manager().get() {
        let result = {
            let mut manager = manager_mutex.lock();
            manager.remove_account(account_name)
        };

        match result {
            Ok(_) => {
                ui_state.notification_manager.show_success(
                    "Account Deleted",
                    &format!("Account '{}' deleted successfully", account_name)
                );
                if ui_state.selected_account.as_deref() == Some(account_name) {
                    *ui_state.selected_account = None;
                }
            }
            Err(e) => {
                ui_state.notification_manager.show_success("Delete Failed", &format!("Failed to delete account: {}", e));
            }
        }
    }
}

fn copy_account_to_clipboard(account_name: &str, ui_state: &mut UiState) {
    if let Some(manager_mutex) = GlobalState::instance().get_account_manager().get() {
        let json_result = {
            let manager = manager_mutex.lock();
            manager.export_to_clipboard(account_name)
        };

        match json_result {
            Ok(json) => {
                if ui_state.clipboard.set_text(&json) {
                    ui_state.notification_manager.show_success(
                        "Copied to Clipboard",
                        &format!("Account '{}' copied to clipboard", account_name)
                    );
                } else {
                    ui_state.notification_manager.show_error("Copy Failed", "Failed to copy to clipboard");
                }
            }
            Err(e) => {
                ui_state.notification_manager.show_error("Export Failed", &format!("Failed to export account: {}", e));
            }
        }
    }
}