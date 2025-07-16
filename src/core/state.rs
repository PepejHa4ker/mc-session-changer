use std::sync::atomic::{AtomicBool, AtomicU32, AtomicIsize, Ordering};
use std::sync::OnceLock;
use parking_lot::Mutex;
use crate::graphics::context::PayloadContext;
use crate::account::AccountManager;

static LAST_KEY_STATE: AtomicU32 = AtomicU32::new(0);
static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
static MENU_VISIBLE: AtomicBool = AtomicBool::new(true);
static ORIGINAL_WNDPROC: AtomicIsize = AtomicIsize::new(0);
static UNLOAD_INITIATED: AtomicBool = AtomicBool::new(false);
static CURRENT_WINDOW: AtomicIsize = AtomicIsize::new(0);
static CONTEXT: OnceLock<Mutex<Option<PayloadContext>>> = OnceLock::new();
static ACCOUNT_MANAGER: OnceLock<Mutex<AccountManager>> = OnceLock::new();

pub struct GlobalState;

impl GlobalState {
    pub fn get_last_key_state() -> u32 {
        LAST_KEY_STATE.load(Ordering::Relaxed)
    }

    pub fn set_last_key_state(state: u32) {
        LAST_KEY_STATE.store(state, Ordering::Relaxed);
    }

    pub fn increment_frame_count() -> u32 {
        FRAME_COUNT.fetch_add(1, Ordering::Relaxed)
    }

    pub fn is_menu_visible() -> bool {
        MENU_VISIBLE.load(Ordering::Relaxed)
    }

    pub fn set_menu_visible(visible: bool) {
        MENU_VISIBLE.store(visible, Ordering::Relaxed);
    }

    pub fn get_original_wndproc() -> isize {
        ORIGINAL_WNDPROC.load(Ordering::Relaxed)
    }

    pub fn set_original_wndproc(proc: isize) {
        ORIGINAL_WNDPROC.store(proc, Ordering::Relaxed);
    }

    pub fn is_unload_initiated() -> bool {
        UNLOAD_INITIATED.load(Ordering::Relaxed)
    }

    pub fn set_unload_initiated(initiated: bool) {
        UNLOAD_INITIATED.store(initiated, Ordering::Relaxed);
    }

    pub fn get_current_window() -> isize {
        CURRENT_WINDOW.load(Ordering::Relaxed)
    }

    pub fn set_current_window(window: isize) {
        CURRENT_WINDOW.store(window, Ordering::Relaxed);
    }

    pub fn get_context() -> &'static OnceLock<Mutex<Option<PayloadContext>>> {
        &CONTEXT
    }

    pub fn get_account_manager() -> &'static OnceLock<Mutex<AccountManager>> {
        &ACCOUNT_MANAGER
    }

    pub fn initialize_account_manager() {
        ACCOUNT_MANAGER.get_or_init(|| Mutex::new(AccountManager::new()));
    }
}