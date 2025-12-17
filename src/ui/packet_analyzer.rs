use crate::core::custom_payload::{decode_custom_payload, DecodedStruct, DecodedValue};
use crate::core::packets::Bound;
use crate::core::state::GlobalState;
use crate::graphics::context::{PacketFilterProfile, PacketTrigger};
use crate::graphics::netlog::{PacketDetails, PacketDirection, PacketRecord, make_record};
use crate::graphics::svg_icons::SvgIconManager;
use crate::ui::UiState;
use base64::Engine;
use egui::{pos2, ScrollArea, Sense};
use egui::{Color32, RichText, StrokeKind, TextStyle, Ui, vec2};
use once_cell::sync::Lazy;
use std::cmp::min;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct Caches {
    decoded_cache: HashMap<u64, DecodedStruct>,
    short_hex_cache: HashMap<u64, String>,
    details_hex_cache: HashMap<(u64, usize), String>,
}

static CACHES: Lazy<Mutex<Caches>> = Lazy::new(|| Mutex::new(Caches::default()));

pub fn render_packet_analyzer_tab(
    ui_state: &mut UiState,
    _icon_manager: &mut SvgIconManager,
    ui: &mut Ui,
) {
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("Packet Analyzer")
                    .size(16.0)
                    .color(Color32::LIGHT_BLUE),
            );
            ui.add_space(12.0);

            ui.checkbox(ui_state.packet_show_inbound, "Inbound");
            ui.checkbox(ui_state.packet_show_outbound, "Outbound");
            ui.checkbox(ui_state.packet_only_pinned, "Only pinned");
            ui.checkbox(ui_state.packet_show_only_new, "Only new");

            ui.add_space(12.0);
            ui.label("Filter:");
            ui.text_edit_singleline(ui_state.packet_filter);

            ui.add_space(12.0);
            ui.checkbox(ui_state.packet_autoscroll, "Autoscroll");
            ui.checkbox(ui_state.packet_paused, "Paused");
            if ui.input(|i| i.modifiers.is_none()) {
                GlobalState::instance().set_packet_paused(*ui_state.packet_paused);
            }

            ui.add_space(12.0);
            ui.label("Max N:");
            ui.add(
                egui::widgets::DragValue::new(ui_state.packet_limit_count).range(0..=500000),
            );
            ui.checkbox(ui_state.packet_autoclear_oldest, "Auto-clear oldest");
            if ui.button("Apply limits").clicked() {
                if let Some(store) = GlobalState::instance().get_packet_store().get() {
                    let mut s = store.lock();
                    let max_n = if *ui_state.packet_limit_count == 0 {
                        None
                    } else {
                        Some(*ui_state.packet_limit_count as usize)
                    };
                    s.set_max_count(max_n);
                    s.set_autoclear_oldest(*ui_state.packet_autoclear_oldest);
                }
            }

            ui.add_space(12.0);
            let detach_label = if *ui_state.packets_detached {
                "Dock"
            } else {
                "Detach"
            };
            if ui.button(detach_label).clicked() {
                *ui_state.packets_detached = !*ui_state.packets_detached;
                *ui_state.packets_window_open = true;
            }

            if ui.button("Clear").clicked() {
                if let Some(store) = GlobalState::instance().get_packet_store().get() {
                    store.lock().clear();
                }
                *ui_state.selected_packet_id = None;

                if let Ok(mut c) = CACHES.lock() {
                    c.decoded_cache.clear();
                    c.short_hex_cache.clear();
                    c.details_hex_cache.clear();
                }
            }
            if ui.button("Mark seen").clicked() {
                if let Some(store) = GlobalState::instance().get_packet_store().get() {
                    let max_seen = store.lock().snapshot().iter().map(|r| r.id).max().unwrap_or(0);
                    *ui_state.packet_last_seen_id = max_seen;
                }
            }
        });

        ui.add_space(6.0);
        render_filter_profiles_bar(ui, ui_state);
        ui.add_space(4.0);
        render_trigger_bar(ui, ui_state);
        ui.add_space(4.0);
        render_import_export_bar(ui, ui_state);
    });

    ui.add_space(8.0);

    if *ui_state.packets_detached {
        ui.colored_label(Color32::GRAY, "Packets are detached to a floating window.");
    } else {
        egui::containers::Resize::default()
            .id_salt("packet_panel_resize")
            .default_size(vec2(ui.available_width(), 800.0))
            .show(ui, |ui| {
                render_packets_panel_with_height(ui, ui_state, 800.0);
            });
    }

    if *ui_state.packet_paused {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(200));
    } else {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
    }
}

pub fn render_packet_analyzer_detached_windows(
    ctx: &egui::Context,
    ui_state: &mut UiState,
) {
    if !*ui_state.packets_detached {
        return;
    }

    let mut win_open: bool = *ui_state.packets_window_open;
    egui::Window::new("Packets")
        .open(&mut win_open)
        .collapsible(true)
        .resizable(true)
        .default_size(vec2(900.0, 520.0))
        .show(ctx, |win_ui| {
            render_packets_panel_with_height(win_ui, ui_state, 800.0);
        });
    *ui_state.packets_window_open = win_open;
}

fn record_matches_query(rec: &PacketRecord, needle_lc: &str) -> bool {
    let name = rec.name.as_str();
    let ch = match &rec.details {
        Some(PacketDetails::CustomPayload { channel, .. }) => channel.as_str(),
        _ => "",
    };
    let group = rec.group.as_deref().unwrap_or("");
    let tags_match = rec.tags.iter().any(|t| contains_ascii_ci(t, needle_lc));

    let hex = {
        let mut caches = CACHES.lock().unwrap();
        caches
            .short_hex_cache
            .entry(rec.id)
            .or_insert_with(|| first_bytes_hex(&rec.data, 64))
            .clone()
    };

    contains_ascii_ci(name, needle_lc)
        || contains_ascii_ci(ch, needle_lc)
        || contains_ascii_ci(group, needle_lc)
        || contains_ascii_ci(&hex, needle_lc)
        || tags_match
}

fn row_highlight_color(rec: &PacketRecord, ui_state: &mut UiState) -> Option<Color32> {
    if let Some(rgb) = rec.color {
        return Some(Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
    }

    for trig in ui_state.packet_triggers.iter() {
        if trigger_matches(rec, trig) {
            if trig.pin {
                if let Some(store) = GlobalState::instance().get_packet_store().get() {
                    store.lock().pin(rec.id, true);
                }
            }
            return Some(Color32::from_rgb(trig.highlight[0], trig.highlight[1], trig.highlight[2]).linear_multiply(0.7));
        }
    }
    None
}

fn trigger_matches(rec: &PacketRecord, trig: &PacketTrigger) -> bool {
    if let Some(dir) = trig.dir {
        if dir != rec.dir {
            return false;
        }
    }
    let needle = trig.needle.as_str();
    record_matches_query(rec, needle)
}

fn render_filter_profiles_bar(ui: &mut Ui, ui_state: &mut UiState) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Profiles:");
        for idx in 0..ui_state.packet_filter_profiles.len() {
            let (apply, remove) = {
                let prof = &ui_state.packet_filter_profiles[idx];
                let apply = ui.button(format!("Use {}", prof.name)).clicked();
                let remove = ui.small_button("x").clicked();
                (apply, remove)
            };

            if apply {
                if let Some(prof) = ui_state.packet_filter_profiles.get(idx).cloned() {
                    *ui_state.packet_filter = prof.query;
                    *ui_state.packet_show_inbound = prof.show_inbound;
                    *ui_state.packet_show_outbound = prof.show_outbound;
                    *ui_state.packet_only_pinned = prof.only_pinned;
                }
            }
            if remove {
                ui_state.packet_filter_profiles.remove(idx);
                break;
            }
        }

        ui.separator();
        ui.label("Save profile as:");
        ui.text_edit_singleline(ui_state.packet_profile_new_name);
        ui.text_edit_singleline(ui_state.packet_profile_new_query)
            .on_hover_text("Custom query; empty -> current filter");
        if ui.button("Save").clicked() {
            let name = if ui_state.packet_profile_new_name.trim().is_empty() {
                format!("Profile {}", ui_state.packet_filter_profiles.len() + 1)
            } else {
                ui_state.packet_profile_new_name.trim().to_string()
            };
            let query = if ui_state.packet_profile_new_query.trim().is_empty() {
                ui_state.packet_filter.trim().to_string()
            } else {
                ui_state.packet_profile_new_query.trim().to_string()
            };
            ui_state.packet_filter_profiles.push(PacketFilterProfile {
                name,
                query,
                show_inbound: *ui_state.packet_show_inbound,
                show_outbound: *ui_state.packet_show_outbound,
                only_pinned: *ui_state.packet_only_pinned,
            });
        }
    });
}

fn render_trigger_bar(ui: &mut Ui, ui_state: &mut UiState) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Triggers:");
        ui.add(
            egui::TextEdit::singleline(ui_state.packet_trigger_input)
                .hint_text("substring or channel"),
        );
        ui.label("Highlight:");
        ui.text_edit_singleline(ui_state.packet_color_hex);
        if ui.button("Add trigger").clicked() {
            if !ui_state.packet_trigger_input.trim().is_empty() {
                let color = parse_hex_color(ui_state.packet_color_hex.trim())
                    .unwrap_or([255, 200, 64]);
                ui_state.packet_triggers.push(PacketTrigger {
                    name: ui_state.packet_trigger_input.trim().to_string(),
                    needle: ui_state.packet_trigger_input.trim().to_ascii_lowercase(),
                    dir: None,
                    highlight: color,
                    pin: false,
                });
                ui_state.packet_trigger_input.clear();
            }
        }
        for idx in 0..ui_state.packet_triggers.len() {
            let name = ui_state.packet_triggers[idx].name.clone();
            if ui.button(format!("x {}", name)).clicked() {
                ui_state.packet_triggers.remove(idx);
                break;
            }
        }
    });
}

fn render_import_export_bar(ui: &mut Ui, ui_state: &mut UiState) {
    ui.collapsing("Import/Replay buffer", |ui| {
        ui.label("Paste JSON (export format) to replay into store:");
        ui.add(
            egui::TextEdit::multiline(ui_state.packet_import_buffer).desired_rows(3),
        );
        ui.horizontal(|ui| {
            if ui.button("Replay buffer").clicked() {
                replay_from_json(ui_state.packet_import_buffer.as_str());
            }
            if ui.button("Clear buffer").clicked() {
                ui_state.packet_import_buffer.clear();
            }
        });
    });
}

fn render_packets_panel_with_height(ui: &mut Ui, ui_state: &mut UiState, list_h: f32) {
    let mut records = if let Some(store) = GlobalState::instance().get_packet_store().get() {
        store.lock().snapshot()
    } else {
        Vec::new()
    };

    records.retain(|r| {
        (*ui_state.packet_show_inbound && r.dir == PacketDirection::Inbound)
            || (*ui_state.packet_show_outbound && r.dir == PacketDirection::Outbound)
    });

    let max_seen = records.iter().map(|r| r.id).max().unwrap_or(0);

    let f = ui_state.packet_filter.trim();
    let needle = if f.is_empty() { None } else { Some(f.to_ascii_lowercase()) };

    if let Some(needle_lc) = needle.as_ref() {
        records.retain(|r| record_matches_query(r, needle_lc));
    }

    if *ui_state.packet_only_pinned {
        records.retain(|r| r.pinned);
    }

    if *ui_state.packet_show_only_new {
        records.retain(|r| r.id > *ui_state.packet_last_seen_id);
    } else if max_seen > *ui_state.packet_last_seen_id {
        *ui_state.packet_last_seen_id = max_seen;
    }

    let total = records.len();

    let row_h = {
        let base = ui.text_style_height(&TextStyle::Body);
        base + 8.0
    };

    ui.separator();

    let top_anchor = ui.min_rect();

    egui::Frame::default().show(ui, |ui| {
        ui.set_max_height(list_h);

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(list_h)
            .show_rows(ui, row_h, total, |ui, row_range| {
                for idx in row_range {
                    let i = total - 1 - idx;
                    let rec = &records[i];

                    let (rect, _) = ui.allocate_exact_size(
                        vec2(ui.available_width(), row_h),
                        Sense::hover(),
                    );

                    // делим пополам для inbound/outbound
                    let mid = rect.center().x;
                    let (left_rect, right_rect) = (
                        egui::Rect::from_min_max(rect.min, pos2(mid - 4.0, rect.max.y)),
                        egui::Rect::from_min_max(pos2(mid + 4.0, rect.min.y), rect.max),
                    );

                    let highlight = row_highlight_color(rec, ui_state);
                    match rec.dir {
                        PacketDirection::Inbound => render_row_cell_in(ui, rec, left_rect, row_h, ui_state, highlight),
                        PacketDirection::Outbound => render_row_cell_out(ui, rec, right_rect, row_h, ui_state, highlight),
                    }
                }
            });

        if *ui_state.packet_autoscroll && !*ui_state.packet_paused {
            ui.scroll_to_rect(top_anchor, Some(egui::Align::TOP));
        }
    });
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.label("Export limit:");
        ui.add(egui::widgets::DragValue::new(ui_state.packet_export_limit).range(1..=10_000));
        let limited = min(total, *ui_state.packet_export_limit as usize);
        if ui.button("Copy JSON (filtered)").clicked() {
            let slice = records.iter().rev().take(limited).cloned().collect::<Vec<_>>();
            if let Ok(text) = serde_json::to_string_pretty(&slice) {
                let _ = ui_state.clipboard.set_text(&text);
                ui_state
                    .notification_manager
                    .show_success("Exported", &format!("{} packets copied as JSON", slice.len()));
            }
        }
        if ui.button("Copy CSV (filtered)").clicked() {
            let slice = records.iter().rev().take(limited).cloned().collect::<Vec<_>>();
            let csv = packets_to_csv(&slice);
            let _ = ui_state.clipboard.set_text(&csv);
            ui_state
                .notification_manager
                .show_success("Exported", &format!("{} packets copied as CSV", slice.len()));
        }
        if ui.button("Replay JSON from clipboard").clicked() {
            if let Some(text) = ui_state.clipboard.get_text() {
                replay_from_json(&text);
            }
        }
    });

    if let Some(sel_id) = *ui_state.selected_packet_id {
        if let Some(rec) = records.iter().find(|r| r.id == sel_id) {
            render_details_panel(ui, rec, ui_state);
        }
    }
}

fn render_row_cell_in(ui: &mut Ui, rec: &PacketRecord, rect: egui::Rect, row_h: f32, ui_state: &mut UiState, highlight: Option<Color32>) {
    render_row_cell_common(ui, rec, rect, row_h, ui_state, Color32::from_rgb(18, 24, 32), "<-", highlight);
}

fn render_row_cell_out(ui: &mut Ui, rec: &PacketRecord, rect: egui::Rect, row_h: f32, ui_state: &mut UiState, highlight: Option<Color32>) {
    render_row_cell_common(ui, rec, rect, row_h, ui_state, Color32::from_rgb(24, 18, 18), "->", highlight);
}

fn render_row_cell_common(
    ui: &mut Ui,
    rec: &PacketRecord,
    rect: egui::Rect,
    row_h: f32,
    ui_state: &mut UiState,
    bg: Color32,
    arrow: &str,
    highlight: Option<Color32>,
) {
    let id = egui::Id::new(("row", rec.id));
    let resp = ui.interact(rect, id, Sense::click());

    let tint = highlight.unwrap_or_else(|| bg.linear_multiply(0.35));
    ui.painter().rect_filled(rect, 4.0, tint);

    let short = rec.name.rsplit('.').next().unwrap_or(&rec.name);
    let t = format_time(rec.ts_millis);
    let title = format!("{arrow} [{t}] {short}");
    let meta = format!("{} bytes", rec.len);

    let left = rect.left() + 8.0;
    let top = rect.center().y - ui.text_style_height(&TextStyle::Body) * 0.5;
    let pin_rect = egui::Rect::from_min_size(
        pos2(left, rect.center().y - 7.0),
        vec2(22.0, 14.0),
    );
    let pin_resp = ui.interact(
        pin_rect,
        egui::Id::new(("pin", rec.id)),
        Sense::click(),
    );

    ui.painter().text(
        pin_rect.center(),
        egui::Align2::CENTER_CENTER,
        if rec.pinned { "PIN" } else { "pin" },
        TextStyle::Body.resolve(ui.style()),
        if rec.pinned { Color32::YELLOW } else { Color32::GRAY },
    );

    let text_left = left + 28.0;
    if pin_resp.clicked() {
        if let Some(store) = GlobalState::instance().get_packet_store().get() {
            store.lock().pin(rec.id, !rec.pinned);
        }
    }
    ui.painter().text(
        pos2(text_left, top),
        egui::Align2::LEFT_TOP,
        title,
        TextStyle::Body.resolve(ui.style()),
        Color32::WHITE,
    );

    ui.painter().text(
        pos2(rect.right() - 8.0, top),
        egui::Align2::RIGHT_TOP,
        meta,
        TextStyle::Monospace.resolve(ui.style()),
        Color32::GRAY,
    );

    let is_selected = matches!(ui_state.selected_packet_id.as_ref(), Some(x) if *x == rec.id);
    if is_selected {
        ui.painter().rect_stroke(
            rect.expand(1.0),
            4.0,
            egui::Stroke::new(1.0, Color32::LIGHT_BLUE),
            StrokeKind::Middle,
        );
    }

    if resp.clicked() {
        *ui_state.selected_packet_id = Some(rec.id);
    }

    if resp.hovered() {
        resp.on_hover_text(format!("{} bytes", rec.data.len()));
    }

    let _ = row_h;
}

fn render_details_panel(ui: &mut Ui, rec: &PacketRecord, ui_state: &mut UiState) {
    let arrow = match rec.dir {
        PacketDirection::Outbound => "->",
        PacketDirection::Inbound => "<-",
    };
    let short = rec.name.rsplit('.').next().unwrap_or(&rec.name);
    let t = format_time(rec.ts_millis);
    let header = format!("{arrow} [{t}] {short}  ({} bytes)", rec.len);

    egui::CollapsingHeader::new(header)
        .id_salt(egui::Id::new(("details", rec.id)))
        .default_open(true)
        .show(ui, |ui| {
            let mut decoded: Option<DecodedStruct> = None;

            if let Some(PacketDetails::CustomPayload { channel, channel_len, preview }) = &rec.details {
                ui.monospace(format!("channel: {channel} ({} bytes)", channel_len));
                if let Some(p) = preview { ui.monospace(format!("preview: \"{p}\"")); }

                let bound = match rec.dir {
                    PacketDirection::Inbound  => Bound::Server,
                    PacketDirection::Outbound => Bound::Client,
                };

                let cached = {
                    let caches = CACHES.lock().unwrap();
                    caches.decoded_cache.get(&rec.id).cloned()
                };
                decoded = if let Some(d) = cached {
                    Some(d)
                } else {
                    let d = decode_custom_payload(channel, &rec.data, bound);
                    if let Some(ref dd) = d {
                        if let Ok(mut caches) = CACHES.lock() {
                            caches.decoded_cache.insert(rec.id, dd.clone());
                        }
                    }
                    d
                };
                ui.add_space(4.0);
            }

            ui.horizontal(|ui| {
                if ui.button(if rec.pinned { "Unpin" } else { "Pin" }).clicked() {
                    if let Some(store) = GlobalState::instance().get_packet_store().get() {
                        store.lock().pin(rec.id, !rec.pinned);
                    }
                }
                if ui.button("Copy hex").clicked() {
                    let hex = {
                        let width = 16usize;
                        let dump = {
                            let mut caches = CACHES.lock().unwrap();
                            caches
                                .details_hex_cache
                                .entry((rec.id, width))
                                .or_insert_with(|| hex_dump(&rec.data, width))
                                .clone()
                        };
                        dump
                    };
                    let _ = ui_state.clipboard.set_text(&hex);
                }
                if ui.button("Copy raw (base64)").clicked() {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&rec.data);
                    let _ = ui_state.clipboard.set_text(&b64);
                }
                if ui.button("Unselect").clicked() {
                    *ui_state.selected_packet_id = None;
                }
            });
            if ui_state.packet_tag_editor.is_empty() && !rec.tags.is_empty() {
                *ui_state.packet_tag_editor = rec.tags.join(", ");
            }
            ui.horizontal_wrapped(|ui| {
                ui.label("Tags:");
                ui.add(
                    egui::TextEdit::singleline(ui_state.packet_tag_editor)
                        .hint_text("comma separated"),
                );
                if ui.button("Apply tags").clicked() {
                    let tags: Vec<String> = ui_state
                        .packet_tag_editor
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if let Some(store) = GlobalState::instance().get_packet_store().get() {
                        store.lock().set_tags(rec.id, tags.clone());
                    }
                    *ui_state.packet_tag_editor = tags.join(", ");
                }
                if !rec.tags.is_empty() {
                    ui.monospace(format!("Current: {}", rec.tags.join(", ")));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Row color (#RRGGBB):");
                ui.text_edit_singleline(ui_state.packet_color_hex);
                if ui.button("Apply color").clicked() {
                    let color = parse_hex_color(ui_state.packet_color_hex.trim());
                    if let Some(store) = GlobalState::instance().get_packet_store().get() {
                        store.lock().set_color(rec.id, color);
                    }
                }
                if ui.button("Clear color").clicked() {
                    if let Some(store) = GlobalState::instance().get_packet_store().get() {
                        store.lock().set_color(rec.id, None);
                    }
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Set as compare target").clicked() {
                    *ui_state.packet_secondary_selected_id = Some(rec.id);
                }
                if let Some(other_id) = *ui_state.packet_secondary_selected_id {
                    if other_id != rec.id {
                        if let Some(other) = find_packet(other_id) {
                            render_diff_section(ui, rec, &other);
                        } else {
                            ui.label("Compare target missing");
                        }
                    }
                }
            });

            if let Some(ds) = decoded.as_ref() {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(RichText::new("Decoded").strong().color(Color32::LIGHT_BLUE));
                render_decoded_struct(ui, ds, ui_state);
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);
            ui.label(RichText::new("Hex dump").strong());

            let width = 16usize;
            let dump = {
                let mut caches = CACHES.lock().unwrap();
                caches
                    .details_hex_cache
                    .entry((rec.id, width))
                    .or_insert_with(|| hex_dump(&rec.data, width))
                    .clone()
            };
            ui.monospace(dump);
        });
}

fn render_decoded_struct(ui: &mut Ui, s: &DecodedStruct, ui_state: &mut UiState) {
    egui::CollapsingHeader::new(&s.name)
        .default_open(true)
        .show(ui, |ui| {
            for f in &s.fields {
                render_field_row(ui, &f.name, &f.value, ui_state);
            }
        });
}

fn render_field_row(ui: &mut Ui, name: &str, value: &DecodedValue, ui_state: &mut UiState) {
    match value {
        DecodedValue::Struct(st) => {
            egui::CollapsingHeader::new(format!("{name}:"))
                .default_open(true)
                .show(ui, |ui| render_decoded_struct(ui, st, ui_state));
        }
        DecodedValue::List(items) => {
            egui::CollapsingHeader::new(format!("{name}: [{}]", items.len()))
                .default_open(false)
                .show(ui, |ui| {
                    for (i, it) in items.iter().enumerate() {
                        render_field_row(ui, &format!("#{i}"), it, ui_state);
                    }
                });
        }
        DecodedValue::Bytes(bytes) => {
            ui.horizontal_wrapped(|ui| {
                ui.monospace(format!(
                    "{name}: <{} bytes>  {}",
                    bytes.len(),
                    first_bytes_hex(bytes, 32)
                ));
                if ui.button("Copy field hex").clicked() {
                    let _ = ui_state.clipboard.set_text(&hex_dump(bytes, 16));
                }
                if ui.button("Copy field b64").clicked() {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                    let _ = ui_state.clipboard.set_text(&b64);
                }
            });
        }
        DecodedValue::Text(t) => {
            ui.monospace(format!("{name}: {t}"));
        }
        DecodedValue::Null => {
            ui.monospace(format!("{name}: null"));
        }
    }
}

fn find_packet(id: u64) -> Option<PacketRecord> {
    let store = GlobalState::instance().get_packet_store().get()?;
    let snap = store.lock().snapshot();
    snap.into_iter().find(|r| r.id == id)
}

fn render_diff_section(ui: &mut Ui, a: &PacketRecord, b: &PacketRecord) {
    let len_a = a.data.len();
    let len_b = b.data.len();
    let min_len = len_a.min(len_b);
    let first_diff = (0..min_len).find(|i| a.data[*i] != b.data[*i]);
    let diff_count = a
        .data
        .iter()
        .take(min_len)
        .zip(b.data.iter())
        .filter(|(x, y)| x != y)
        .count()
        + len_a.saturating_sub(min_len)
        + len_b.saturating_sub(min_len);

    ui.vertical(|ui| {
        ui.label(format!(
            "Diff vs {}: len {} vs {}, mismatches {}",
            b.id, len_a, len_b, diff_count
        ));
        if let Some(idx) = first_diff {
            ui.monospace(format!("First mismatch at byte {idx}: {:02X} vs {:02X}", a.data[idx], b.data[idx]));
        } else {
            ui.monospace("Bodies match up to common length");
        }
        let prev_a = first_bytes_hex(&a.data, 32);
        let prev_b = first_bytes_hex(&b.data, 32);
        ui.monospace(format!("A head: {prev_a}"));
        ui.monospace(format!("B head: {prev_b}"));
    });
}

fn packets_to_csv(records: &[PacketRecord]) -> String {
    let mut out = String::from("id,ts,dir,name,len,tags\n");
    for r in records {
        let dir = match r.dir {
            PacketDirection::Inbound => "in",
            PacketDirection::Outbound => "out",
        };
        let tags = r.tags.join("|").replace(',', ";");
        out.push_str(&format!("{},{},{},{},{},{}\n", r.id, r.ts_millis, dir, r.name, r.len, tags));
    }
    out
}

fn parse_hex_color(s: &str) -> Option<[u8; 3]> {
    let clean = s.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some([r, g, b])
    } else {
        None
    }
}

fn replay_from_json(text: &str) {
    if text.trim().is_empty() {
        return;
    }
    let Ok(parsed) = serde_json::from_str::<Vec<PacketRecord>>(text) else { return; };
    if let Some(store) = GlobalState::instance().get_packet_store().get() {
        let mut guard = store.lock();
        for r in parsed {
            let mut created = make_record(r.dir, r.name.clone(), r.data.clone());
            created.tags = r.tags.clone();
            created.color = r.color;
            created.group = r.group.clone();
            created.pinned = r.pinned;
            guard.push(created);
        }
    }
}

fn hex_dump(data: &[u8], width: usize) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(width).enumerate() {
        use std::fmt::Write;
        let _ = write!(out, "{:04X}:  ", i * width);
        for b in chunk {
            let _ = write!(out, "{:02X} ", b);
        }
        let mut pad = (width - chunk.len()) * 3;
        while pad > 0 {
            out.push(' ');
            pad -= 1;
        }
        out.push(' ');
        for b in chunk {
            let c = if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            };
            out.push(c);
        }
        out.push('\n');
    }
    out
}

fn first_bytes_hex(data: &[u8], n: usize) -> String {
    data.iter()
        .take(n)
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join("")
}

fn contains_ascii_ci(hay: &str, needle_lc: &str) -> bool {
    if needle_lc.is_empty() {
        return true;
    }
    let n = needle_lc.as_bytes();
    let hb = hay.as_bytes();
    if n.len() > hb.len() {
        return false;
    }
    hb.windows(n.len()).any(|w| {
        w.eq_ignore_ascii_case(n)
    })
}

fn format_time(millis: u64) -> String {
    use chrono::{DateTime, Local, TimeZone, Utc};
    let secs = (millis / 1000) as i64;
    let dt = DateTime::<Utc>::from_timestamp(secs, 0)
        .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).expect("timestamp zero is valid"));
    Local.from_utc_datetime(&dt.naive_utc())
        .format("%H:%M:%S")
        .to_string()
}
