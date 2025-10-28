use super::notifications::*;
use crate::graphics::svg_icons::SvgIconManager;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use egui::{Vec2, Pos2, Rect, Color32, FontId, Align2, CornerRadius, Stroke, StrokeKind};

pub struct NotificationManager {
    notifications: Arc<Mutex<VecDeque<Notification>>>,
    next_id: Arc<Mutex<u64>>,
    max_notifications: usize,
    notification_width: f32,
    notification_height: f32,
    margin: f32,
    spacing: f32,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(Mutex::new(VecDeque::new())),
            next_id: Arc::new(Mutex::new(0)),
            max_notifications: 5,
            notification_width: 350.0,
            notification_height: 80.0,
            margin: 20.0,
            spacing: 10.0,
        }
    }

    pub fn add_notification(
        &self,
        title: String,
        message: String,
        notification_type: NotificationType,
        duration: Option<Duration>,
    ) -> u64 {
        let mut notifications = self.notifications.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();

        let id = *next_id;
        *next_id += 1;

        let duration = duration.unwrap_or(Duration::from_secs(3));

        let notification = Notification::new(title, message, notification_type)
            .with_duration(duration);

        notifications.push_back(notification);

        while notifications.len() > self.max_notifications {
            notifications.pop_front();
        }

        tracing::debug!("Added notification: {} - {}", id, notifications.back().unwrap().title);
        id
    }

    pub fn update(&self, dt: Duration) {
        let mut notifications = self.notifications.lock().unwrap();

        for notification in notifications.iter_mut() {
            notification.update(dt);
        }

        notifications.retain(|n| !n.is_expired());
    }

    pub fn render_in_context(&self, egui_ctx: &egui::Context, icon_manager: &mut SvgIconManager, screen_size: Vec2) {
        let notifications = self.notifications.lock().unwrap();

        if notifications.is_empty() {
            return;
        }

        let mut y_offset = self.margin;

        for notification in notifications.iter() {
            self.render_notification_in_context(egui_ctx, icon_manager, notification, screen_size, y_offset);
            y_offset += self.notification_height + self.spacing;
        }
    }

    fn render_notification_in_context(
        &self,
        egui_ctx: &egui::Context,
        icon_manager: &mut SvgIconManager,
        notification: &Notification,
        screen_size: Vec2,
        y_offset: f32,
    ) {
        let alpha = notification.alpha();
        if alpha <= 0.0 {
            return;
        }

        let x = screen_size.x - self.notification_width - self.margin;
        let y = y_offset;

        let slide_offset = match notification.state {
            NotificationState::FadingIn => (1.0 - notification.progress) * 100.0,
            NotificationState::FadingOut => notification.progress * 100.0,
            _ => 0.0,
        };

        let pos = Pos2::new(x + slide_offset, y);
        let rect = Rect::from_min_size(pos, Vec2::new(self.notification_width, self.notification_height));

        self.render_notification_content_in_context(egui_ctx, icon_manager, notification, rect, alpha);
    }

    fn render_notification_content_in_context(
        &self,
        egui_ctx: &egui::Context,
        icon_manager: &mut SvgIconManager,
        notification: &Notification,
        rect: Rect,
        alpha: f32,
    ) {
        let base_color = notification.notification_type.color();
        let bg_color = Color32::from_rgba_premultiplied(
            base_color.r(),
            base_color.g(),
            base_color.b(),
            (200.0 * alpha) as u8,
        );

        let text_color = Color32::from_rgba_premultiplied(255, 255, 255, (255.0 * alpha) as u8);

        let painter = egui_ctx.layer_painter(egui::LayerId::background());

        painter.rect_filled(
            rect,
            CornerRadius::same(8),
            bg_color,
        );

        painter.rect_stroke(
            rect,
            CornerRadius::same(8),
            Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, (100.0 * alpha) as u8)),
            StrokeKind::Inside
        );

        let progress_rect = Rect::from_min_size(
            rect.min + Vec2::new(0.0, rect.height() - 3.0),
            Vec2::new(rect.width(), 3.0),
        );

        painter.rect_filled(
            progress_rect,
            CornerRadius::same(0),
            Color32::from_rgba_premultiplied(255, 255, 255, (60.0 * alpha) as u8),
        );

        let progress = notification.get_progress_percentage();
        let progress_fill_rect = Rect::from_min_size(
            progress_rect.min,
            Vec2::new(progress_rect.width() * progress, progress_rect.height()),
        );

        painter.rect_filled(
            progress_fill_rect,
            CornerRadius::same(0),
            Color32::from_rgba_premultiplied(255, 255, 255, (150.0 * alpha) as u8),
        );

        let icon_rect = Rect::from_min_size(
            rect.min + Vec2::new(15.0, 15.0),
            Vec2::new(24.0, 24.0)
        );

        let icon_color = Color32::from_rgba_premultiplied(255, 255, 255, (255.0 * alpha) as u8);

        if let Some(texture) = icon_manager.get_icon_texture(
            egui_ctx,
            notification.notification_type.icon_name(),
            24,
            icon_color,
        ) {
            painter.image(
                texture.id(),
                icon_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        let title_pos = rect.min + Vec2::new(50.0, 15.0);
        painter.text(
            title_pos,
            Align2::LEFT_TOP,
            &notification.title,
            FontId::proportional(16.0),
            text_color,
        );

        let message_pos = rect.min + Vec2::new(50.0, 35.0);
        painter.text(
            message_pos,
            Align2::LEFT_TOP,
            &notification.message,
            FontId::proportional(14.0),
            Color32::from_rgba_premultiplied(255, 255, 255, (200.0 * alpha) as u8),
        );
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManager {
    pub fn show_info(&self, title: &str, message: &str) -> u64 {
        self.add_notification(
            title.to_string(),
            message.to_string(),
            NotificationType::Info,
            None,
        )
    }

    pub fn show_success(&self, title: &str, message: &str) -> u64 {
        self.add_notification(
            title.to_string(),
            message.to_string(),
            NotificationType::Success,
            None,
        )
    }

    pub fn show_warning(&self, title: &str, message: &str) -> u64 {
        self.add_notification(
            title.to_string(),
            message.to_string(),
            NotificationType::Warning,
            Some(Duration::from_secs(5)),
        )
    }

    pub fn show_error(&self, title: &str, message: &str) -> u64 {
        self.add_notification(
            title.to_string(),
            message.to_string(),
            NotificationType::Error,
            Some(Duration::from_secs(7)),
        )
    }
}