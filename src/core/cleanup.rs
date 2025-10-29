use std::time::Duration;
use winapi::{
    um::winuser::{SetWindowLongPtrW, GWLP_WNDPROC},
    shared::windef::HWND,
    um::libloaderapi::FreeLibraryAndExitThread
};
use crate::core::state::GlobalState;
use crate::DLL_HANDLE;
use crate::hooks::jhook;
use crate::hooks::opengl::cleanup_opengl_hooks;
use crate::jni_hook::JNIHook_Shutdown;

pub struct CleanupManager;

impl CleanupManager {
    pub unsafe fn cleanup_resources() -> Result<(), Box<dyn std::error::Error>> {
        let state = GlobalState::instance();

        Self::restore_window_procedure(state)?;

        if let Err(e) = cleanup_opengl_hooks() {
            tracing::error!("Failed to cleanup OpenGL hooks: {}", e);
        }

        Self::cleanup_graphics_context(state)?;
        jhook::unhook_all()?;
        jhook::shutdown()?;
        tracing::info!("Resources cleaned up successfully");
        Ok(())
    }

    unsafe fn restore_window_procedure(state: &GlobalState) -> Result<(), Box<dyn std::error::Error>> {
        let window_ptr = state.get_current_window();
        if window_ptr != 0 {
            let window = window_ptr as HWND;
            let original_proc = state.get_original_wndproc();
            if original_proc != 0 {
                SetWindowLongPtrW(window, GWLP_WNDPROC, original_proc as isize);
                tracing::debug!("Window procedure restored");
            }
        }
        Ok(())
    }

    fn cleanup_graphics_context(state: &GlobalState) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(context_mutex) = state.get_context().get() {
            match context_mutex.try_lock_for(Duration::from_millis(100)) {
                Some(mut context_guard) => {
                    *context_guard = None;
                    tracing::debug!("Graphics context cleaned up");
                }
                None => {
                    tracing::warn!("Failed to acquire context lock for cleanup");
                }
            }
        }
        Ok(())
    }

    pub unsafe fn initiate_cleanup() {
        std::thread::spawn(|| {
            tracing::info!("Initiating cleanup sequence");
            if let Err(e) = Self::cleanup_resources() {
                tracing::error!("Cleanup failed: {}", e);
            }

            std::thread::sleep(Duration::from_millis(200));

            if let Some(dll_handle) = DLL_HANDLE.get() {
                tracing::info!("Unloading DLL");
                FreeLibraryAndExitThread(dll_handle.get(), 0);
            }
        });
    }
}

pub unsafe fn initiate_cleanup() {
    CleanupManager::initiate_cleanup();
}