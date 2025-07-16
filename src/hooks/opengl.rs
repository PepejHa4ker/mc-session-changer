use std::sync::OnceLock;
use winapi::shared::windef::HDC;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::winuser::GetAsyncKeyState;
use winapi::um::winuser::VK_INSERT;
use winapi::um::wingdi::wglGetCurrentContext;
use parking_lot::Mutex;
use crate::core::state::GlobalState;
use crate::graphics::opengl::create_render_context;
use crate::{SHOULD_UNLOAD};

type FnSwapBuffers = unsafe extern "system" fn(HDC) -> i32;

pub static SWAP_BUFFERS: OnceLock<retour::GenericDetour<FnSwapBuffers>> = OnceLock::new();

pub fn initialize_opengl_hooks() -> Result<(), String> {
    unsafe {
        let gdi32 = GetModuleHandleA(b"gdi32.dll\0".as_ptr() as *const i8);
        if gdi32.is_null() {
            return Err("Failed to get gdi32.dll handle".to_string());
        }

        let swap_buffers_addr = GetProcAddress(gdi32, b"SwapBuffers\0".as_ptr() as *const i8);
        if swap_buffers_addr.is_null() {
            return Err("Failed to get SwapBuffers address".to_string());
        }

        let detour = retour::GenericDetour::<FnSwapBuffers>::new(
            std::mem::transmute(swap_buffers_addr),
            hk_swap_buffers,
        ).map_err(|e| format!("Failed to create detour: {:?}", e))?;

        detour.enable().map_err(|e| format!("Failed to enable detour: {:?}", e))?;

        SWAP_BUFFERS.set(detour).map_err(|_| "Failed to set swap buffers hook")?;

        Ok(())
    }
}

unsafe extern "system" fn hk_swap_buffers(hdc: HDC) -> i32 {
    let swap_buffers = SWAP_BUFFERS.get().expect("swap buffers hook not initialized");

    if SHOULD_UNLOAD.load(std::sync::atomic::Ordering::Relaxed) && !GlobalState::is_unload_initiated() {
        GlobalState::set_unload_initiated(true);
        let result = swap_buffers.call(hdc);
        crate::core::cleanup::initiate_cleanup();
        return result;
    }

    if GlobalState::is_unload_initiated() {
        return swap_buffers.call(hdc);
    }

    handle_input();

    let current_context = wglGetCurrentContext();
    if current_context.is_null() {
        return swap_buffers.call(hdc);
    }

    if GlobalState::is_menu_visible() {
        render_overlay(hdc);
    }

    swap_buffers.call(hdc)
}

fn handle_input() {
    let frame_count = GlobalState::increment_frame_count();
    if frame_count % 30 == 0 {
        unsafe {
            let current_key_state = GetAsyncKeyState(VK_INSERT) as u32;
            let last_key_state = GlobalState::get_last_key_state();

            if current_key_state != 0 && last_key_state == 0 {
                let new_visibility = !GlobalState::is_menu_visible();
                GlobalState::set_menu_visible(new_visibility);
            }

            GlobalState::set_last_key_state(current_key_state);
        }
    }
}

fn render_overlay(hdc: HDC) {
    let context_mutex = GlobalState::get_context().get_or_init(|| {
        unsafe {
            Mutex::new(create_render_context(hdc).ok())
        }
    });

    if let Some(mut context_guard) = context_mutex.try_lock() {
        if let Some(context) = context_guard.as_mut() {
            let _ = context.render(hdc);
        }
    }
}