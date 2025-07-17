use crate::graphics::svg_icons::SvgIconManager;
use egui::{Color32, Rect, Response, Sense, Ui, Vec2};

pub fn render_icon(
    icon_manager: &mut SvgIconManager,
    ui: &mut Ui,
    icon_name: &str,
    rect: Rect,
    color: Color32,
    size: Option<u32>,
) {
    let size = size.unwrap_or(24);

    if let Some(texture) = icon_manager.get_icon_texture(ui.ctx(), icon_name, size, color) {
        ui.painter().image(
            texture.id(),
            rect,
            Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}


pub fn render_clickable_icon_with_text(
    icon_manager: &mut SvgIconManager,
    ui: &mut Ui,
    icon_name: &str,
    text: &str,
    color: Color32,
    icon_size: Option<u32>,
    hover_text: &str,
) -> Response {
    let icon_size = icon_size.unwrap_or(16) as f32;
    let spacing = 8.0;

    let text_size = ui.fonts(|f| f.layout_no_wrap(text.to_string(), egui::FontId::default(), Color32::WHITE).size());

    let total_size = Vec2::new(
        icon_size + spacing + text_size.x,
        icon_size.max(text_size.y),
    );

    let (rect, response) = ui.allocate_exact_size(total_size, Sense::click());

    if response.hovered() {
        ui.painter().rect_filled(
            rect.expand(2.0),
            4.0,
            Color32::from_white_alpha(20),
        );
    }

    let icon_rect = Rect::from_min_size(
        rect.min,
        Vec2::new(icon_size, icon_size),
    );
    render_icon(icon_manager, ui, icon_name, icon_rect, color, Some(icon_size as u32));

    let text_pos = rect.min + Vec2::new(icon_size + spacing, (rect.height() - text_size.y) * 0.5);
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_TOP,
        text,
        egui::FontId::default(),
        color,
    );

    response.on_hover_text(hover_text)
}

pub fn render_decorative_icon(
    icon_manager: &mut SvgIconManager,
    ui: &mut Ui,
    icon_name: &str,
    color: Color32,
    size: Option<u32>,
) {
    let size = size.unwrap_or(16) as f32;
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(size, size),
        Sense::hover(),
    );

    render_icon(icon_manager, ui, icon_name, rect, color, Some(size as u32));
}