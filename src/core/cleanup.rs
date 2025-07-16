use std::time::Duration;
use winapi::shared::windef::HWND;
use winapi::um::winuser::{SetWindowLongPtrW, GWLP_WNDPROC};
use winapi::um::libloaderapi::FreeLibraryAndExitThread;
use crate::core::state::GlobalState;
use crate::hooks::opengl::SWAP_BUFFERS;
use crate::DLL_HANDLE;

pub unsafe fn cleanup_resources() {
    let window_ptr = GlobalState::get_current_window();
    if window_ptr != 0 {
        let window = window_ptr as HWND;
        let original_proc = GlobalState::get_original_wndproc();
        if original_proc != 0 {
            SetWindowLongPtrW(window, GWLP_WNDPROC, original_proc);
        }
    }

    if let Some(detour) = SWAP_BUFFERS.get() {
        let _ = detour.disable();
    }

    if let Some(context_mutex) = GlobalState::get_context().get() {
        if let Some(mut context_guard) = context_mutex.try_lock() {
            *context_guard = None;
        }
    }

    std::thread::sleep(Duration::from_millis(50));
}

pub unsafe fn initiate_cleanup() {
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(500));
        cleanup_resources();

        std::thread::sleep(Duration::from_millis(200));

        if let Some(dll_handle) = DLL_HANDLE.get() {
            FreeLibraryAndExitThread(dll_handle.get(), 0);
        }
    });
}