use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use winapi::shared::minwindef::{DWORD, HINSTANCE, LPVOID};
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::libloaderapi::{DisableThreadLibraryCalls};
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
    SHOULD_UNLOAD.store(true, Ordering::Relaxed);
    GlobalState::set_menu_visible(false);
}

unsafe extern "system" fn start_routine(_parameter: LPVOID) -> DWORD {
    initialize_logging();

    if let Err(e) = initialize_opengl_hooks() {
        tracing::error!("Failed to initialize OpenGL hooks: {}", e);
        return 1;
    }

    let _ = get_minecraft_session();
    tracing::info!("Minecraft session initialized");

    loop {
        if SHOULD_UNLOAD.load(Ordering::Relaxed) {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

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
            DLL_HANDLE.set(SafeHMODULE::new(dll_module)).expect("Failed to set DLL handle");

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