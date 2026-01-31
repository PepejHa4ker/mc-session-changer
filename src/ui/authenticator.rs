use crate::{
    async_runtime::ASYNC_RUNTIME,
    auth,
    graphics::icon_renderer::{render_clickable_icon_with_text, render_decorative_icon},
    graphics::svg_icons::SvgIconManager,
    ui::UiState,
};
use egui::{Color32, RichText, TextEdit, Ui};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::sync::mpsc;

enum AuthTabResult {
    Success { token: String, profile: String },
    Error(String),
}

static AUTH_TAB_CHANNEL: Lazy<Mutex<(
    mpsc::UnboundedSender<AuthTabResult>,
    mpsc::UnboundedReceiver<AuthTabResult>,
)>> = Lazy::new(|| {
    let (tx, rx) = mpsc::unbounded_channel();
    Mutex::new((tx, rx))
});

pub fn render_authenticator_tab(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    poll_auth_result(ui_state);

    ui.vertical(|ui| {
        render_header(icon_manager, ui);
        ui.add_space(8.0);
        render_input_section(ui_state, icon_manager, ui);
        ui.add_space(8.0);
        render_result_section(ui_state, icon_manager, ui);
    });
}

fn poll_auth_result(ui_state: &mut UiState) {
    if let Ok(mut guard) = AUTH_TAB_CHANNEL.try_lock() {
        if let Ok(result) = guard.1.try_recv() {
            *ui_state.auth_tab_in_progress = false;
            match result {
                AuthTabResult::Success { token, profile } => {
                    *ui_state.auth_tab_error = None;
                    *ui_state.auth_tab_result_token = token;
                    *ui_state.auth_tab_result_profile = profile;
                }
                AuthTabResult::Error(msg) => {
                    *ui_state.auth_tab_error = Some(msg);
                }
            }
        }
    }
}

fn render_header(icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    ui.horizontal(|ui| {
        render_decorative_icon(icon_manager, ui, "key", Color32::from_rgb(255, 180, 50), Some(16));
        ui.label(RichText::new("Аутентификатор").size(18.0).color(Color32::from_rgb(255, 180, 50)));
    });
}

fn render_input_section(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                render_decorative_icon(icon_manager, ui, "user", Color32::LIGHT_BLUE, Some(14));
                ui.label(RichText::new("Никнейм:").color(Color32::LIGHT_BLUE));
            });
            ui.add(
                TextEdit::singleline(ui_state.auth_tab_username)
                    .desired_width(ui.available_width())
                    .hint_text("Введите никнейм"),
            );

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                render_decorative_icon(icon_manager, ui, "key", Color32::LIGHT_BLUE, Some(14));
                ui.label(RichText::new("Пароль:").color(Color32::LIGHT_BLUE));
            });
            ui.add(
                TextEdit::singleline(ui_state.auth_tab_password)
                    .desired_width(ui.available_width())
                    .password(true)
                    .hint_text("Введите пароль"),
            );

            ui.add_space(8.0);

            let can_auth = !ui_state.auth_tab_username.is_empty()
                && !ui_state.auth_tab_password.is_empty()
                && !*ui_state.auth_tab_in_progress;

            ui.horizontal(|ui| {
                if *ui_state.auth_tab_in_progress {
                    ui.spinner();
                    ui.label(RichText::new("Авторизация...").color(Color32::YELLOW));
                } else {
                    ui.add_enabled_ui(can_auth, |ui| {
                        if render_clickable_icon_with_text(
                            icon_manager,
                            ui,
                            "apply",
                            "Авторизоваться",
                            Color32::LIGHT_GREEN,
                            Some(16),
                            "Выполнить авторизацию",
                        )
                        .clicked()
                        {
                            start_auth(ui_state);
                        }
                    });
                }
            });
        });
    });
}

fn render_result_section(ui_state: &mut UiState, icon_manager: &mut SvgIconManager, ui: &mut Ui) {
    if let Some(err) = ui_state.auth_tab_error.clone() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                render_decorative_icon(icon_manager, ui, "error", Color32::RED, Some(14));
                ui.label(RichText::new(format!("Ошибка: {}", err)).color(Color32::RED));
            });
        });
    }

    if !ui_state.auth_tab_result_token.is_empty() {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    render_decorative_icon(icon_manager, ui, "success", Color32::LIGHT_GREEN, Some(14));
                    ui.label(RichText::new("Результат").size(16.0).color(Color32::LIGHT_GREEN));
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    render_decorative_icon(icon_manager, ui, "key", Color32::GRAY, Some(14));
                    ui.label(RichText::new("Token:").color(Color32::GRAY));
                });
                let token = ui_state.auth_tab_result_token.clone();
                ui.add(
                    TextEdit::singleline(ui_state.auth_tab_result_token)
                        .desired_width(ui.available_width()),
                );
                ui.horizontal(|ui| {
                    if render_clickable_icon_with_text(
                        icon_manager, ui, "copy", "Копировать",
                        Color32::LIGHT_BLUE, Some(14), "Скопировать токен",
                    ).clicked() {
                        ui_state.clipboard.set_text(&token);
                        ui_state.notification_manager.show_success("Скопировано", "Токен скопирован");
                    }
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    render_decorative_icon(icon_manager, ui, "id", Color32::GRAY, Some(14));
                    ui.label(RichText::new("Player ID:").color(Color32::GRAY));
                });
                let profile = ui_state.auth_tab_result_profile.clone();
                ui.add(
                    TextEdit::singleline(ui_state.auth_tab_result_profile)
                        .desired_width(ui.available_width()),
                );
                ui.horizontal(|ui| {
                    if render_clickable_icon_with_text(
                        icon_manager, ui, "copy", "Копировать",
                        Color32::LIGHT_BLUE, Some(14), "Скопировать Player ID",
                    ).clicked() {
                        ui_state.clipboard.set_text(&profile);
                        ui_state.notification_manager.show_success("Скопировано", "Player ID скопирован");
                    }
                });
            });
        });
    }
}

fn start_auth(ui_state: &mut UiState) {
    *ui_state.auth_tab_in_progress = true;
    *ui_state.auth_tab_error = None;
    ui_state.auth_tab_result_token.clear();
    ui_state.auth_tab_result_profile.clear();

    let username = ui_state.auth_tab_username.clone();
    let password = ui_state.auth_tab_password.clone();

    let sender = AUTH_TAB_CHANNEL.lock().unwrap().0.clone();

    ASYNC_RUNTIME.spawn(async move {
        match auth::auth(&username, &password).await {
            Ok(data) => {
                let _ = sender.send(AuthTabResult::Success {
                    token: data.access_token,
                    profile: data.profile,
                });
            }
            Err(e) => {
                let _ = sender.send(AuthTabResult::Error(format!("{}", e)));
            }
        }
    });
}
