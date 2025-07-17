use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicUsize, Ordering};
use std::sync::OnceLock;
use parking_lot::Mutex;
use crate::graphics::context::PayloadContext;
use crate::account::AccountManager;
use crate::ui::notification_manager::NotificationManager;

pub struct GlobalState {
    last_key_state: AtomicU32,
    frame_count: AtomicU32,
    menu_visible: AtomicBool,
    original_wndproc: AtomicUsize,
    unload_initiated: AtomicBool,
    current_window: AtomicIsize,
    context: OnceLock<Mutex<Option<PayloadContext>>>,
    account_manager: OnceLock<Mutex<AccountManager>>,
}

impl GlobalState {
    const fn new() -> Self {
        Self {
            last_key_state: AtomicU32::new(0),
            frame_count: AtomicU32::new(0),
            menu_visible: AtomicBool::new(true),
            original_wndproc: AtomicUsize::new(0),
            unload_initiated: AtomicBool::new(false),
            current_window: AtomicIsize::new(0),
            context: OnceLock::new(),
            account_manager: OnceLock::new(),
        }
    }

    pub fn instance() -> &'static Self {
        static INSTANCE: GlobalState = GlobalState::new();
        &INSTANCE
    }

    pub fn get_last_key_state(&self) -> u32 {
        self.last_key_state.load(Ordering::Acquire)
    }

    pub fn set_last_key_state(&self, state: u32) {
        self.last_key_state.store(state, Ordering::Release);
    }

    pub fn increment_frame_count(&self) -> u32 {
        self.frame_count.fetch_add(1, Ordering::Relaxed)
    }

    pub fn is_menu_visible(&self) -> bool {
        self.menu_visible.load(Ordering::Acquire)
    }

    pub fn set_menu_visible(&self, visible: bool) {
        self.menu_visible.store(visible, Ordering::Release);
    }

    pub fn get_original_wndproc(&self) -> usize {
        self.original_wndproc.load(Ordering::Acquire)
    }

    pub fn set_original_wndproc(&self, proc: usize) {
        self.original_wndproc.store(proc, Ordering::Release);
    }

    pub fn is_unload_initiated(&self) -> bool {
        self.unload_initiated.load(Ordering::Acquire)
    }

    pub fn set_unload_initiated(&self, initiated: bool) {
        self.unload_initiated.store(initiated, Ordering::Release);
    }

    pub fn get_current_window(&self) -> isize {
        self.current_window.load(Ordering::Acquire)
    }

    pub fn set_current_window(&self, window: isize) {
        self.current_window.store(window, Ordering::Release);
    }

    pub fn get_context(&self) -> &OnceLock<Mutex<Option<PayloadContext>>> {
        &self.context
    }

    pub fn get_account_manager(&self) -> &OnceLock<Mutex<AccountManager>> {
        &self.account_manager
    }

    pub fn initialize_account_manager(&self) {
        self.account_manager.get_or_init(|| Mutex::new(AccountManager::new()));
    }
}