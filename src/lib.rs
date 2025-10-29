#![allow(static_mut_refs)]

use std::ptr::null_mut;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use winapi::{
    um::processthreadsapi::CreateThread,
    um::libloaderapi::DisableThreadLibraryCalls,
    shared::minwindef::{DWORD, HINSTANCE, LPVOID},
    um::winnt::{DLL_PROCESS_ATTACH}
};

mod account;
mod core;
mod graphics;
mod hooks;
mod input;
mod jvm;
mod ui;
mod utils;
mod jni_hook;

use crate::core::logging::initialize_logging;
use crate::core::state::GlobalState;
use crate::hooks::opengl::initialize_opengl_hooks;
use crate::jvm::get_jvm;
use crate::utils::SafeHMODULE;

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

    let _ = get_jvm();
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