use crate::account::AccountManager;
use crate::graphics::context::PayloadContext;
use parking_lot::Mutex;
use std::{
    sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicUsize, Ordering},
    sync::OnceLock
};
use crate::graphics::netlog::PacketStore;

pub struct GlobalState {
    last_key_state: AtomicU32,
    frame_count: AtomicU32,
    menu_visible: AtomicBool,
    original_wndproc: AtomicUsize,
    unload_initiated: AtomicBool,
    current_window: AtomicIsize,
    context: OnceLock<Mutex<Option<PayloadContext>>>,
    account_manager: OnceLock<Mutex<AccountManager>>,
    packet_store: OnceLock<Mutex<PacketStore>>,
    packet_paused: AtomicBool,

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
            packet_store: OnceLock::new(),
            packet_paused: AtomicBool::new(false),
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

    pub fn get_packet_store(&self) -> &OnceLock<Mutex<PacketStore>> {
        &self.packet_store
    }
    pub fn initialize_packet_store(&self) {
        self.packet_store.get_or_init(|| Mutex::new(PacketStore::new(500)));
    }

    pub fn is_packet_paused(&self) -> bool {
        self.packet_paused.load(Ordering::Acquire)
    }
    pub fn set_packet_paused(&self, v: bool) {
        self.packet_paused.store(v, Ordering::Release)
    }
}