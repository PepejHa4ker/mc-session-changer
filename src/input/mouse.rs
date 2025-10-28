use egui::{Event, Pos2, PointerButton, Vec2, MouseWheelUnit};
use winapi::shared::minwindef::{WPARAM, LPARAM};
use crate::input::keyboard::get_modifiers;

pub fn handle_mouse_move(lparam: LPARAM) -> Event {
    let x = (lparam & 0xFFFF) as i16 as f32;
    let y = ((lparam >> 16) & 0xFFFF) as i16 as f32;
    Event::PointerMoved(Pos2::new(x, y))
}

pub fn handle_mouse_button(lparam: LPARAM, button: PointerButton, pressed: bool) -> Event {
    let x = (lparam & 0xFFFF) as i16 as f32;
    let y = ((lparam >> 16) & 0xFFFF) as i16 as f32;
    Event::PointerButton {
        pos: Pos2::new(x, y),
        button,
        pressed,
        modifiers: get_modifiers(),
    }
}

pub fn handle_mouse_wheel(wparam: WPARAM) -> Event {
    let delta = ((wparam >> 16) & 0xFFFF) as i16 as f32 / 120.0;
    Event::MouseWheel {
        unit: MouseWheelUnit::Line,
        delta: Vec2::new(0.0, delta),
        modifiers: get_modifiers(),
    }
}