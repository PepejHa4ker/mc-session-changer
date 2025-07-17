use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use winapi::shared::minwindef::{DWORD, HINSTANCE, LPVOID};
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::libloaderapi::DisableThreadLibraryCalls;
use winapi::um::processthreadsapi::CreateThread;
use std::ptr::null_mut;

mod core;
mod ui;
mod input;
mod graphics;
mod hooks;
mod jvm;
mod account;
mod utils;

use crate::core::state::GlobalState;
use crate::core::logging::initialize_logging;
use crate::hooks::opengl::initialize_opengl_hooks;
use crate::utils::SafeHMODULE;
use crate::jvm::get_minecraft_session;

static DLL_HANDLE: OnceLock<SafeHMODULE> = OnceLock::new();
static SHOULD_UNLOAD: AtomicBool = AtomicBool::new(false);

pub fn initiate_unload() {
    SHOULD_UNLOAD.store(true, Ordering::Release);
    GlobalState::instance().set_menu_visible(false);
    tracing::info!("Unload initiated");
}

unsafe extern "system" fn start_routine(_parameter: LPVOID) -> DWORD {
    if let Err(_) = initialize_logging() {
        return 1;
    }

    tracing::info!("Starting initialization sequence");

    if let Err(e) = initialize_opengl_hooks() {
        tracing::error!("Failed to initialize OpenGL hooks: {}", e);
        return 1;
    }

    let _ = get_minecraft_session();
    tracing::info!("Minecraft session initialized");

    GlobalState::instance().initialize_account_manager();

    loop {
        if SHOULD_UNLOAD.load(Ordering::Acquire) {
            tracing::info!("Shutdown signal received");
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    tracing::info!("Main loop exited");
    0
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: DWORD,
    _reserved: LPVOID,
) -> i32 {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(dll_module);

            if DLL_HANDLE.set(SafeHMODULE::new(dll_module)).is_err() {
                return 0;
            }

            let thread = CreateThread(
                null_mut(),
                0,
                Some(start_routine),
                dll_module as _,
                0,
                null_mut(),
            );

            if thread.is_null() {
                return 0;
            }

            1
        }
        _ => 1,
    }
}