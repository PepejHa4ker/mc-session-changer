use crate::{
    graphics::context::AppTab,
    graphics::icon_renderer::{render_clickable_icon_with_text, render_decorative_icon},
    graphics::svg_icons::SvgIconManager,
    initiate_unload,
    ui::account_manager::render_account_manager_tab,
    ui::session_window::render_session_tab,
    ui::UiState,
};
use egui::{Color32, Context};
use crate::ui::jvm_analyzer::render_jvm_analyzer_tab;
use crate::ui::packet_analyzer::{render_packet_analyzer_tab, render_packet_analyzer_detached_windows};

pub fn render_main_window(
    ui_state: &mut UiState,
    icon_manager: &mut SvgIconManager,
    ctx: &Context,
) {
    egui::Window::new("Truncator Tools")
        .default_size([600.0, 500.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if render_clickable_icon_with_text(
                    icon_manager,
                    ui,
                    "session_changer",
                    if *ui_state.selected_tab == AppTab::SessionChanger {
                        "Session Changer"
                    } else {
                        "Session Changer"
                    },
                    if *ui_state.selected_tab == AppTab::SessionChanger {
                        Color32::LIGHT_BLUE
                    } else {
                        Color32::GRAY
                    },
                    Some(16),
                    "Switch to Session Changer tab",
                )
                .clicked()
                {
                    *ui_state.selected_tab = AppTab::SessionChanger;
                }

                ui.separator();

                if render_clickable_icon_with_text(
                    icon_manager,
                    ui,
                    "account",
                    if *ui_state.selected_tab == AppTab::AccountManager {
                        "Account Manager"
                    } else {
                        "Account Manager"
                    },
                    if *ui_state.selected_tab == AppTab::AccountManager {
                        Color32::LIGHT_GREEN
                    } else {
                        Color32::GRAY
                    },
                    Some(16),
                    "Switch to Account Manager tab",
                )
                .clicked()
                {
                    *ui_state.selected_tab = AppTab::AccountManager;
                }

                ui.separator();
                if render_clickable_icon_with_text(
                    icon_manager,
                    ui,
                    "info",
                    "Packet Analyzer",
                    if *ui_state.selected_tab == AppTab::PacketAnalyzer {
                        Color32::LIGHT_BLUE
                    } else {
                        Color32::GRAY
                    },
                    Some(16),
                    "Switch to Packet Analyzer tab",
                )

                .clicked()
                {
                    *ui_state.selected_tab = AppTab::PacketAnalyzer;
                }
                ui.separator();
                if render_clickable_icon_with_text(
                    icon_manager,
                    ui,
                    "info",
                    "JVM Analyzer",
                    if *ui_state.selected_tab == AppTab::JvmAnalyzer {
                        Color32::LIGHT_BLUE
                    } else {
                        Color32::GRAY
                    },
                    Some(16),
                    "Switch to Jvm Analyzer tab",
                )
                    .clicked()
                {
                    *ui_state.selected_tab = AppTab::JvmAnalyzer;
                }
            });
            match *ui_state.selected_tab {
                AppTab::SessionChanger => render_session_tab(ui_state, icon_manager, ui),
                AppTab::AccountManager => render_account_manager_tab(ui_state, icon_manager, ui),
                AppTab::PacketAnalyzer => render_packet_analyzer_tab(ui_state, icon_manager, ui),
                AppTab::JvmAnalyzer => render_jvm_analyzer_tab(ui_state, ui),
            }

            render_unload_section(ui_state, icon_manager, ui);
        });

    // Render floating/detached packet analyzer windows regardless of the active tab
    render_packet_analyzer_detached_windows(ctx, ui_state);
}

fn render_unload_section(
    ui_state: &mut UiState,
    icon_manager: &mut SvgIconManager,
    ui: &mut egui::Ui,
) {
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        if crate::SHOULD_UNLOAD.load(std::sync::atomic::Ordering::Relaxed) {
            ui.horizontal(|ui| {
                render_decorative_icon(icon_manager, ui, "refresh", Color32::YELLOW, Some(20));
                ui.add_space(8.0);
                ui.colored_label(
                    Color32::YELLOW,
                    egui::RichText::new("Unloading DLL...").size(16.0).strong(),
                );
            });
        } else {
            ui.style_mut().spacing.button_padding = egui::Vec2::new(20.0, 12.0);
            ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(140, 30, 30);
            ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(180, 40, 40);
            ui.style_mut().visuals.widgets.active.bg_fill = Color32::from_rgb(120, 25, 25);

            let button_response = render_clickable_icon_with_text(
                icon_manager,
                ui,
                "unload",
                "Unload DLL",
                Color32::RED,
                Some(24),
                "Unload DLL and clean up resources",
            );

            if button_response.clicked() {
                ui_state
                    .notification_manager
                    .show_info("Unloading", "Initiating safe unload...");
                initiate_unload();
            }

            button_response.on_hover_text("Safely unload the DLL from memory");
        }
    });

    ui.add_space(8.0);
}
