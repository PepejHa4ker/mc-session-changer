use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT};
use winapi::um::winuser::{
    CallWindowProcW, WM_MOUSEMOVE, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_MOUSEWHEEL, WM_KEYDOWN, WM_KEYUP, WM_CHAR, WM_SYSKEYDOWN, WM_SYSKEYUP
};
use egui::{Event, PointerButton};
use crate::core::state::GlobalState;
use crate::input::mouse::{handle_mouse_move, handle_mouse_button, handle_mouse_wheel};
use crate::input::keyboard::{virtual_key_to_egui_key, get_modifiers};
use crate::SHOULD_UNLOAD;

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if SHOULD_UNLOAD.load(std::sync::atomic::Ordering::Relaxed) {
        let original_proc = GlobalState::instance().get_original_wndproc();
        if original_proc != 0 {
            return CallWindowProcW(Some(std::mem::transmute(original_proc)), hwnd, msg, wparam, lparam);
        }
        return 0;
    }

    if !GlobalState::instance().is_menu_visible() {
        let original_proc = GlobalState::instance().get_original_wndproc();
        if original_proc != 0 {
            return CallWindowProcW(Some(std::mem::transmute(original_proc)), hwnd, msg, wparam, lparam);
        }
    }

    let should_handle = matches!(msg,
        WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | 
        WM_MOUSEWHEEL | WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP | WM_CHAR
    );

    if should_handle {
        if let Some(context_mutex) = GlobalState::instance().get_context().get() {
            if let Some(mut context_guard) = context_mutex.try_lock() {
                if let Some(context) = context_guard.as_mut() {
                    let event = match msg {
                        WM_MOUSEMOVE => Some(handle_mouse_move(lparam)),
                        WM_LBUTTONDOWN => Some(handle_mouse_button(lparam, PointerButton::Primary, true)),
                        WM_LBUTTONUP => Some(handle_mouse_button(lparam, PointerButton::Primary, false)),
                        WM_RBUTTONDOWN => Some(handle_mouse_button(lparam, PointerButton::Secondary, true)),
                        WM_RBUTTONUP => Some(handle_mouse_button(lparam, PointerButton::Secondary, false)),
                        WM_MOUSEWHEEL => Some(handle_mouse_wheel(wparam)),
                        WM_KEYDOWN | WM_SYSKEYDOWN => {
                            handle_key_down(wparam, lparam, context)
                        }
                        WM_KEYUP | WM_SYSKEYUP => {
                            handle_key_up(wparam, lparam)
                        }
                        WM_CHAR => {
                            handle_char(wparam)
                        }
                        _ => None,
                    };

                    if let Some(event) = event {
                        context.add_event(event);
                    }

                    if context.egui_ctx.wants_pointer_input() || context.egui_ctx.wants_keyboard_input() {
                        return 0;
                    }
                }
            }
        }
    }

    let original_proc = GlobalState::instance().get_original_wndproc();
    if original_proc != 0 {
        CallWindowProcW(Some(std::mem::transmute(original_proc)), hwnd, msg, wparam, lparam)
    } else {
        0
    }
}

fn handle_key_down(wparam: WPARAM, lparam: LPARAM, context: &mut crate::graphics::context::PayloadContext) -> Option<Event> {
    let vk = wparam as i32;
    let modifiers = get_modifiers();

    if vk == 0x56 && modifiers.ctrl {
        if let Some(text) = context.clipboard.get_text() {
            return Some(Event::Text(text));
        }
    }

    if let Some(key) = virtual_key_to_egui_key(vk) {
        Some(Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: (lparam & 0x40000000) != 0,
            modifiers,
        })
    } else {
        None
    }
}

fn handle_key_up(wparam: WPARAM, _lparam: LPARAM) -> Option<Event> {
    let vk = wparam as i32;
    if let Some(key) = virtual_key_to_egui_key(vk) {
        Some(Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: get_modifiers(),
        })
    } else {
        None
    }
}

fn handle_char(wparam: WPARAM) -> Option<Event> {
    let ch = wparam as u32;
    if let Some(character) = std::char::from_u32(ch) {
        if character.is_control() {
            return None;
        }
        Some(Event::Text(character.to_string()))
    } else {
        None
    }
}