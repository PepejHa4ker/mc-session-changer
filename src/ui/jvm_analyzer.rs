
use crate::core::jvm_analyzer::{ClassInfo, JvmAnalyzer};
use crate::jvm::{get_jvm};
use egui::{pos2, Color32, RichText, ScrollArea, Sense, TextStyle, Ui, vec2};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::ui::UiState;

#[derive(Default)]
struct Caches {
    class_info_cache:                            HashMap<String, ClassInfo>,
}

static CACHES: Lazy<Mutex<Caches>> = Lazy::new(|| Mutex::new(Caches::default()));

/// Проверяет, совпадает ли строка поиска с текстом (case-insensitive)
fn matches_search(search: &str, text: &str) -> bool {
    if search.is_empty() {
        return true;
    }
    text.to_lowercase().contains(&search.to_lowercase())
}

pub fn render_jvm_analyzer_tab(ui_state: &mut UiState, ui: &mut Ui) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("JVM Class Analyzer")
                    .size(16.0)
                    .color(Color32::LIGHT_BLUE),
            );
            ui.add_space(12.0);

            ui.label("Class name:");
            ui.text_edit_singleline(ui_state.search_query);

            if ui.button("Analyze").clicked() {
                analyze_class(ui_state);
            }

            ui.add_space(12.0);

            ui.checkbox(ui_state.expand_fields, "Expand fields");
            ui.checkbox(ui_state.expand_methods, "Expand methods");

            if ui.button("Clear cache").clicked() {
                if let Ok(mut c) = CACHES.lock() {
                    c.class_info_cache.clear();
                }
                *ui_state.selected_class = None;
                *ui_state.error_message = None;
            }

            ui.add_space(12.0);
            let detach_label = if *ui_state.detached {
                "Dock"
            } else {
                "Detach"
            };
            if ui.button(detach_label).clicked() {
                *ui_state.detached = !*ui_state.detached;
                *ui_state.window_open = true;
            }
        });
    });

    ui.add_space(8.0);

    if *ui_state.detached {
        let mut win_open = *ui_state.window_open;
        egui::Window::new("JVM Analyzer")
            .open(&mut win_open)
            .collapsible(true)
            .resizable(true)
            .default_size(vec2(1000.0, 700.0))
            .show(ui.ctx(), |win_ui| {
                render_analyzer_panel(win_ui, ui_state, 600.0);
            });
        *ui_state.window_open = win_open;
    } else {
        egui::containers::Resize::default()
            .id_salt("jvm_analyzer_resize")
            .default_size(vec2(ui.available_width(), 600.0))
            .show(ui, |ui| {
                render_analyzer_panel(ui, ui_state, 600.0);
            });
    }

    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(100));
}

fn render_analyzer_panel(ui: &mut Ui, ui_state: &mut UiState, _panel_h: f32) {
    if let Some(err) = &*ui_state.error_message {
        ui.label(
            RichText::new(format!("❌ Error: {}", err))
                .color(Color32::RED)
                .monospace(),
        );
        ui.add_space(6.0);
    }

    if *ui_state.is_loading {
        ui.label(RichText::new("⏳ Loading...").color(Color32::YELLOW));
        return;
    }

    // Отображение информации о классе
    let cached_info = {
        if let Some(selected) = &*ui_state.selected_class {
            let c = CACHES.lock().unwrap();
            c.class_info_cache.get(selected).cloned()
        } else {
            None
        }
    };

    if let Some(class_info) = cached_info {
        render_class_details(ui, &class_info, ui_state);
    }
}

fn render_class_details(
    ui: &mut Ui,
    class_info: &ClassInfo,
    ui_state: &mut UiState,
) {
    ui.separator();
    ui.add_space(6.0);

    // Информация о классе
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(&class_info.class_name)
                    .size(14.0)
                    .strong()
                    .color(Color32::LIGHT_BLUE),
            );

            ui.monospace(format!("Simple name: {}", class_info.simple_name));
            ui.monospace(format!("Modifiers: {}", class_info.modifiers_str));

            if let Some(superclass) = &class_info.superclass {
                ui.monospace(format!("Superclass: {}", superclass));
            }

            if !class_info.interfaces.is_empty() {
                ui.monospace(format!("Interfaces ({})", class_info.interfaces.len()));
                for iface in &class_info.interfaces {
                    ui.monospace(format!("  • {}", iface));
                }
            }
        });
    });

    ui.add_space(8.0);

    // Поиск по методам и полям
    ui.horizontal(|ui| {
        ui.label("🔍 Search members:");
        ui.text_edit_singleline(ui_state.member_search_query);
        if ui.button("Clear").clicked() {
            ui_state.member_search_query.clear();
        }
    });

    ui.add_space(6.0);

    // Поля
    if !class_info.fields.is_empty() {
        let filtered_fields: Vec<_> = class_info.fields.iter()
            .filter(|f| {
                let search = ui_state.member_search_query.to_lowercase();
                matches_search(&search, &f.name)
                    || matches_search(&search, &f.field_type)
                    || matches_search(&search, &f.modifiers_str)
            })
            .collect();

        egui::CollapsingHeader::new(format!(
            "Fields ({}/{})",
            filtered_fields.len(),
            class_info.fields.len()
        ))
            .id_salt(egui::Id::new("fields_header"))
            .default_open(*ui_state.expand_fields)
            .show(ui, |ui| {
                let row_h = ui.text_style_height(&TextStyle::Monospace) + 6.0;

                ScrollArea::vertical()
                    .max_height(200.0)
                    .show_rows(ui, row_h, filtered_fields.len(), |ui, row_range| {
                        for idx in row_range {
                            let field = filtered_fields[idx];
                            render_field_row(ui, field, row_h);
                        }
                    });
            });

        ui.add_space(6.0);
    }

    // Методы
    if !class_info.methods.is_empty() {
        let filtered_methods: Vec<_> = class_info.methods.iter()
            .filter(|m| {
                let search = ui_state.member_search_query.to_lowercase();
                matches_search(&search, &m.name)
                    || matches_search(&search, &m.signature)
                    || matches_search(&search, &m.modifiers_str)
            })
            .collect();

        egui::CollapsingHeader::new(format!(
            "Methods ({}/{})",
            filtered_methods.len(),
            class_info.methods.len()
        ))
            .id_salt(egui::Id::new("methods_header"))
            .default_open(*ui_state.expand_methods)
            .show(ui, |ui| {
                let row_h = ui.text_style_height(&TextStyle::Monospace) + 8.0;

                ScrollArea::vertical()
                    .max_height(300.0)
                    .show_rows(
                        ui,
                        row_h,
                        filtered_methods.len(),
                        |ui, row_range| {
                            for idx in row_range {
                                let method = filtered_methods[idx];
                                render_method_row(ui, method, row_h);
                            }
                        },
                    );
            });
    }
}

fn render_field_row(ui: &mut Ui, field: &crate::core::jvm_analyzer::FieldInfo, row_h: f32) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::hover());

    ui.painter()
        .rect_filled(rect, 2.0, Color32::from_rgb(30, 35, 40).linear_multiply(0.5));

    let left = rect.left() + 6.0;
    let mid = rect.left() + 180.0;
    let right = mid + 200.0;

    ui.painter().text(
        pos2(left, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &field.modifiers_str,
        TextStyle::Monospace.resolve(ui.style()),
        Color32::LIGHT_BLUE,
    );

    // Тип
    ui.painter().text(
        pos2(mid, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &field.field_type,
        TextStyle::Monospace.resolve(ui.style()),
        Color32::LIGHT_GREEN,
    );

    // Имя
    ui.painter().text(
        pos2(right, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &field.name,
        TextStyle::Monospace.resolve(ui.style()),
        Color32::WHITE,
    );
}

fn render_method_row(ui: &mut Ui, method: &crate::core::jvm_analyzer::MethodInfo, row_h: f32) {
    ui.vertical(|ui| {
        let (rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), row_h * 0.6), Sense::hover());

        ui.painter()
            .rect_filled(rect.expand(1.0), 2.0, Color32::from_rgb(30, 35, 40).linear_multiply(0.5));

        let left = rect.left() + 6.0;
        let mid = rect.left() + 150.0;

        // Модификаторы + имя
        ui.painter().text(
            pos2(left, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &method.modifiers_str,
            TextStyle::Monospace.resolve(ui.style()),
            Color32::LIGHT_BLUE,
        );

        ui.painter().text(
            pos2(mid, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &method.name,
            TextStyle::Monospace.resolve(ui.style()),
            Color32::YELLOW,
        );

        // Сигнатура на следующей строке
        ui.monospace(
            RichText::new(&method.signature)
                .color(Color32::GRAY)
                .size(11.0),
        );
    });
}

fn analyze_class(ui_state: &mut UiState) {
    let class_name = ui_state.search_query.trim().to_string();

    if class_name.is_empty() {
        *ui_state.error_message = Some("Please enter a class name".to_string());
        return;
    }

    // Проверяем кэш
    {
        let c = CACHES.lock().unwrap();
        if c.class_info_cache.contains_key(&class_name) {
            *ui_state.selected_class = Some(class_name);
            *ui_state.error_message = None;
            return;
        }
    }

    *ui_state.is_loading = true;
    *ui_state.error_message = None;

    let env = get_jvm();
    match env.get_env() {
        Ok(mut env) => {
            match unsafe { JvmAnalyzer::analyze_class(&mut env, &class_name) } {
                Ok(info) => {
                    if let Ok(mut c) = CACHES.lock() {
                        c.class_info_cache.insert(class_name.clone(), info);
                    }
                    *ui_state.selected_class = Some(class_name);
                    *ui_state.is_loading = false;
                }
                Err(_e) => {
                    *ui_state.is_loading = false;
                }
            }
        }
        Err(_e) => {
            *ui_state.is_loading = false;
        }
    }
}
