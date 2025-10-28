use winapi::um::winuser::{GetKeyState, VK_MENU, VK_CONTROL, VK_SHIFT};
use winapi::um::winuser::{
    VK_BACK, VK_RETURN, VK_TAB, VK_ESCAPE, VK_SPACE, VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN,
    VK_HOME, VK_END, VK_DELETE
};
use egui::{Key, Modifiers};

pub fn virtual_key_to_egui_key(vk: i32) -> Option<Key> {
    match vk {
        VK_BACK => Some(Key::Backspace),
        VK_TAB => Some(Key::Tab),
        VK_RETURN => Some(Key::Enter),
        VK_ESCAPE => Some(Key::Escape),
        VK_SPACE => Some(Key::Space),
        VK_LEFT => Some(Key::ArrowLeft),
        VK_UP => Some(Key::ArrowUp),
        VK_RIGHT => Some(Key::ArrowRight),
        VK_DOWN => Some(Key::ArrowDown),
        VK_HOME => Some(Key::Home),
        VK_END => Some(Key::End),
        VK_DELETE => Some(Key::Delete),

        0x30 => Some(Key::Num0),
        0x31 => Some(Key::Num1),
        0x32 => Some(Key::Num2),
        0x33 => Some(Key::Num3),
        0x34 => Some(Key::Num4),
        0x35 => Some(Key::Num5),
        0x36 => Some(Key::Num6),
        0x37 => Some(Key::Num7),
        0x38 => Some(Key::Num8),
        0x39 => Some(Key::Num9),

        0x41 => Some(Key::A),
        0x42 => Some(Key::B),
        0x43 => Some(Key::C),
        0x44 => Some(Key::D),
        0x45 => Some(Key::E),
        0x46 => Some(Key::F),
        0x47 => Some(Key::G),
        0x48 => Some(Key::H),
        0x49 => Some(Key::I),
        0x4A => Some(Key::J),
        0x4B => Some(Key::K),
        0x4C => Some(Key::L),
        0x4D => Some(Key::M),
        0x4E => Some(Key::N),
        0x4F => Some(Key::O),
        0x50 => Some(Key::P),
        0x51 => Some(Key::Q),
        0x52 => Some(Key::R),
        0x53 => Some(Key::S),
        0x54 => Some(Key::T),
        0x55 => Some(Key::U),
        0x56 => Some(Key::V),
        0x57 => Some(Key::W),
        0x58 => Some(Key::X),
        0x59 => Some(Key::Y),
        0x5A => Some(Key::Z),

        0x70 => Some(Key::F1),
        0x71 => Some(Key::F2),
        0x72 => Some(Key::F3),
        0x73 => Some(Key::F4),
        0x74 => Some(Key::F5),
        0x75 => Some(Key::F6),
        0x76 => Some(Key::F7),
        0x77 => Some(Key::F8),
        0x78 => Some(Key::F9),
        0x79 => Some(Key::F10),
        0x7A => Some(Key::F11),
        0x7B => Some(Key::F12),

        0xBD => Some(Key::Minus),
        0xBB => Some(Key::Equals),
        0xDB => Some(Key::OpenBracket),
        0xDD => Some(Key::CloseBracket),
        0xDC => Some(Key::Backslash),
        0xBA => Some(Key::Semicolon),
        0xDE => Some(Key::Quote),
        0xBC => Some(Key::Comma),
        0xBE => Some(Key::Period),
        0xBF => Some(Key::Slash),
        0xC0 => Some(Key::Backtick),

        _ => None,
    }
}

pub fn get_modifiers() -> Modifiers {
    unsafe {
        Modifiers {
            alt: (GetKeyState(VK_MENU as i32) & 0x8000u16 as i16) != 0,
            ctrl: (GetKeyState(VK_CONTROL as i32) & 0x8000u16 as i16) != 0,
            shift: (GetKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0,
            mac_cmd: false,
            command: (GetKeyState(VK_CONTROL as i32) & 0x8000u16 as i16) != 0,
        }
    }
}