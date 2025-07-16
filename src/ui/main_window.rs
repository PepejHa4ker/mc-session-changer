use egui::{Context, Color32};
use crate::graphics::renderer::UiState;
use crate::graphics::context::AppTab;
use crate::ui::session_window::render_session_tab;
use crate::ui::account_manager::render_account_manager_tab;
use crate::initiate_unload;

pub fn render_main_window(ctx: &Context, ui_state: &mut UiState) {
    egui::Window::new("🎮 Minecraft Session Changer")
        .default_size([600.0, 500.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(ui_state.selected_tab, AppTab::SessionChanger, "🔄 Session Changer");
                ui.selectable_value(ui_state.selected_tab, AppTab::AccountManager, "👤 Account Manager");
            });

            ui.separator();

            match ui_state.selected_tab {
                AppTab::SessionChanger => {
                    render_session_tab(ui, ui_state);
                }
                AppTab::AccountManager => {
                    render_account_manager_tab(ui, ui_state);
                }
            }

            ui.separator();
            render_unload_section(ui, ui_state);
        });
}

fn render_unload_section(ui: &mut egui::Ui, ui_state: &mut UiState) {
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        if crate::SHOULD_UNLOAD.load(std::sync::atomic::Ordering::Relaxed) {
            ui.colored_label(Color32::YELLOW, "🔄 Unloading...");
        } else if ui.button("🚪 Unload DLL").clicked() {
            *ui_state.status_message = "🔄 Initiating safe unload...".to_string();
            initiate_unload();
        }
    });
}