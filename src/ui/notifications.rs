use std::time::{Duration, Instant};
use egui::Color32;

#[derive(Debug, Clone)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationType {
    pub fn color(&self) -> Color32 {
        match self {
            NotificationType::Info => Color32::from_rgb(66, 165, 245),
            NotificationType::Success => Color32::from_rgb(76, 175, 80),
            NotificationType::Warning => Color32::from_rgb(255, 193, 7),
            NotificationType::Error => Color32::from_rgb(244, 67, 54),
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            NotificationType::Info => "info",
            NotificationType::Success => "success",
            NotificationType::Warning => "warning",
            NotificationType::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub enum NotificationState {
    FadingIn,
    Visible,
    FadingOut,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: Instant,
    pub duration: Duration,
    pub state: NotificationState,
    pub progress: f32,
    pub fade_duration: Duration,
}

impl Notification {
    pub fn new(title: String, message: String, notification_type: NotificationType) -> Self {
        Self {
            title,
            message,
            notification_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(5),
            state: NotificationState::FadingIn,
            progress: 0.0,
            fade_duration: Duration::from_millis(300),
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn update(&mut self, _dt: Duration) {
        let elapsed = self.created_at.elapsed();
        let total_duration = self.duration + self.fade_duration * 2;

        if elapsed <= self.fade_duration {

            self.state = NotificationState::FadingIn;
            self.progress = elapsed.as_secs_f32() / self.fade_duration.as_secs_f32();
        } else if elapsed <= self.duration + self.fade_duration {

            self.state = NotificationState::Visible;
            self.progress = 1.0;
        } else if elapsed <= total_duration {

            self.state = NotificationState::FadingOut;
            let fade_start = self.duration + self.fade_duration;
            let fade_elapsed = elapsed - fade_start;
            self.progress = fade_elapsed.as_secs_f32() / self.fade_duration.as_secs_f32();
        }
    }

    pub fn is_expired(&self) -> bool {
        let total_duration = self.duration + self.fade_duration * 2;
        self.created_at.elapsed() > total_duration
    }

    pub fn alpha(&self) -> f32 {
        match self.state {
            NotificationState::FadingIn => self.progress,
            NotificationState::Visible => 1.0,
            NotificationState::FadingOut => 1.0 - self.progress,
        }
    }

    pub fn get_progress_percentage(&self) -> f32 {
        let elapsed = self.created_at.elapsed();
        let fade_start = self.fade_duration;
        let visible_duration = self.duration;

        if elapsed <= fade_start {

            0.0
        } else if elapsed <= fade_start + visible_duration {

            let visible_elapsed = elapsed - fade_start;
            visible_elapsed.as_secs_f32() / visible_duration.as_secs_f32()
        } else {

            1.0
        }
    }
}