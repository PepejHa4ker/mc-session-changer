use std::collections::HashMap;
use egui::{Context, TextureHandle, Color32, ColorImage, TextureOptions};
use resvg::{usvg, tiny_skia};

#[derive(Default)]
pub struct SvgIconManager {
    textures: HashMap<String, TextureHandle>,
    svg_cache: HashMap<String, usvg::Tree>,
}

impl SvgIconManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            svg_cache: HashMap::new(),
        }
    }

    pub fn get_icon_texture(
        &mut self,
        ctx: &Context,
        icon_name: &str,
        size: u32,
        color: Color32,
    ) -> Option<&TextureHandle> {
        let cache_key = format!("{}_{}_{}_{}", icon_name, size, color.r(), color.g());

        if self.textures.contains_key(&cache_key) {
            return self.textures.get(&cache_key);
        }

        if let Some(texture) = self.create_icon_texture(ctx, icon_name, size, color) {
            self.textures.insert(cache_key.clone(), texture);
            return self.textures.get(&cache_key);
        }

        None
    }

    fn create_icon_texture(
        &mut self,
        ctx: &Context,
        icon_name: &str,
        size: u32,
        color: Color32,
    ) -> Option<TextureHandle> {
        let svg_data = self.get_svg_data(icon_name)?;
        let tree = self.get_or_parse_svg(icon_name, &svg_data)?;

        let mut pixmap = tiny_skia::Pixmap::new(size, size)?;

        let transform = tiny_skia::Transform::from_scale(
            size as f32 / tree.size().width(),
            size as f32 / tree.size().height(),
        );

        resvg::render(&tree, transform, &mut pixmap.as_mut());

        let mut rgba_data = Vec::with_capacity((size * size * 4) as usize);
        for pixel in pixmap.pixels() {
            let r = pixel.red();
            let g = pixel.green();
            let b = pixel.blue();
            let a = pixel.alpha();

            let tint_factor = a as f32 / 255.0;
            let final_r = ((r as f32 * (1.0 - tint_factor)) + (color.r() as f32 * tint_factor)) as u8;
            let final_g = ((g as f32 * (1.0 - tint_factor)) + (color.g() as f32 * tint_factor)) as u8;
            let final_b = ((b as f32 * (1.0 - tint_factor)) + (color.b() as f32 * tint_factor)) as u8;

            rgba_data.push(final_r);
            rgba_data.push(final_g);
            rgba_data.push(final_b);
            rgba_data.push(a);
        }

        let color_image = ColorImage::from_rgba_unmultiplied(
            [size as usize, size as usize],
            &rgba_data,
        );

        Some(ctx.load_texture(
            format!("icon_{}", icon_name),
            color_image,
            TextureOptions::LINEAR,
        ))
    }

    fn get_or_parse_svg(&mut self, icon_name: &str, svg_data: &str) -> Option<usvg::Tree> {
        if let Some(tree) = self.svg_cache.get(icon_name) {
            return Some(tree.clone());
        }

        let options = usvg::Options::default();
        if let Ok(tree) = usvg::Tree::from_str(svg_data, &options) {
            self.svg_cache.insert(icon_name.to_string(), tree.clone());
            Some(tree)
        } else {
            None
        }
    }

    fn get_svg_data(&self, icon_name: &str) -> Option<String> {
        match icon_name {
            "add" => Some(include_str!("../../assets/icons/add.svg").to_string()),
            "copy" => Some(include_str!("../../assets/icons/copy.svg").to_string()),
            "edit" => Some(include_str!("../../assets/icons/edit.svg").to_string()),
            "save" => Some(include_str!("../../assets/icons/save.svg").to_string()),
            "cancel" => Some(include_str!("../../assets/icons/cancel.svg").to_string()),
            "user" => Some(include_str!("../../assets/icons/user.svg").to_string()),
            "id" => Some(include_str!("../../assets/icons/id.svg").to_string()),
            "key" => Some(include_str!("../../assets/icons/key.svg").to_string()),
            "time" => Some(include_str!("../../assets/icons/time.svg").to_string()),
            "refresh" => Some(include_str!("../../assets/icons/refresh.svg").to_string()),
            "success" => Some(include_str!("../../assets/icons/success.svg").to_string()),
            "error" => Some(include_str!("../../assets/icons/error.svg").to_string()),
            "warning" => Some(include_str!("../../assets/icons/warning.svg").to_string()),
            "info" => Some(include_str!("../../assets/icons/info.svg").to_string()),
            "account" => Some(include_str!("../../assets/icons/account.svg").to_string()),
            "session_active" => Some(include_str!("../../assets/icons/session_active.svg").to_string()),
            "account_add" => Some(include_str!("../../assets/icons/account_add.svg").to_string()),
            "unload" => Some(include_str!("../../assets/icons/unload.svg").to_string()),
            "delete" => Some(include_str!("../../assets/icons/delete.svg").to_string()),
            "clear" => Some(include_str!("../../assets/icons/clear.svg").to_string()),
            "apply" => Some(include_str!("../../assets/icons/apply.svg").to_string()),
            "session_changer" => Some(include_str!("../../assets/icons/session_changer.svg").to_string()),
            _ => None,
        }
    }
}