
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering, AtomicIsize};
use std::ptr::null_mut;
use std::time::{Instant, Duration};
use backtrace::Backtrace;
use winapi::shared::windef::{HDC, HGLRC, HWND};
use winapi::shared::minwindef::{DWORD, HINSTANCE, LPVOID, UINT, WPARAM, LPARAM, LRESULT, HMODULE};
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::libloaderapi::{FreeLibraryAndExitThread, GetModuleHandleA, GetProcAddress};
use winapi::um::processthreadsapi::{CreateThread};
use winapi::um::wingdi::{wglGetCurrentContext, wglCreateContext, wglMakeCurrent, wglShareLists};
use winapi::um::winuser::{
    GetAsyncKeyState, GetClientRect, WindowFromDC, VK_INSERT, SetWindowLongPtrW,
    CallWindowProcW, GWLP_WNDPROC, WM_MOUSEMOVE,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_MOUSEWHEEL,
    WM_KEYDOWN, WM_KEYUP, WM_CHAR, WM_SYSKEYDOWN, WM_SYSKEYUP, GetKeyState, VK_BACK, VK_RETURN, VK_TAB, VK_ESCAPE, VK_SPACE,
    VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN, VK_HOME, VK_END, VK_DELETE, VK_CONTROL,
    VK_SHIFT, VK_MENU, OpenClipboard, CloseClipboard, GetClipboardData, CF_UNICODETEXT};
use winapi::um::winnt::HANDLE;
use egui::{Context, RawInput, Event, Key, Pos2, Vec2, PointerButton, Modifiers};
use parking_lot::Mutex;
use winapi::um::winbase::{GlobalLock, GlobalUnlock};

mod jvm;

use jvm::{get_minecraft_session, SessionInfo};

type FnSwapBuffers = unsafe extern "system" fn(HDC) -> i32;

static DLL_HANDLE: OnceLock<SafeHMODULE> = OnceLock::new();
static SWAP_BUFFERS: OnceLock<retour::GenericDetour<FnSwapBuffers>> = OnceLock::new();
static CONTEXT: OnceLock<Mutex<Option<PayloadContext>>> = OnceLock::new();
static LAST_KEY_STATE: AtomicU32 = AtomicU32::new(0);
static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
static MENU_VISIBLE: AtomicBool = AtomicBool::new(true);
static ORIGINAL_WNDPROC: AtomicIsize = AtomicIsize::new(0);
static SHOULD_UNLOAD: AtomicBool = AtomicBool::new(false);
static UNLOAD_INITIATED: AtomicBool = AtomicBool::new(false);
static CURRENT_WINDOW: AtomicIsize = AtomicIsize::new(0);

#[derive(Debug)]
struct SafeGLContext(HGLRC);

unsafe impl Send for SafeGLContext {}
unsafe impl Sync for SafeGLContext {}

impl SafeGLContext {
    fn new(ctx: HGLRC) -> Self {
        Self(ctx)
    }

    fn get(&self) -> HGLRC {
        self.0
    }
}

impl Drop for SafeGLContext {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                winapi::um::wingdi::wglDeleteContext(self.0);
            }
        }
    }
}

#[derive(Debug)]
struct SafeHWND(HWND);

unsafe impl Send for SafeHWND {}
unsafe impl Sync for SafeHWND {}

impl SafeHWND {
    fn new(hwnd: HWND) -> Self {
        Self(hwnd)
    }

    fn get(&self) -> HWND {
        self.0
    }
}

#[derive(Debug, Clone)]
struct SafeHMODULE(HMODULE);

unsafe impl Send for SafeHMODULE {}
unsafe impl Sync for SafeHMODULE {}

impl SafeHMODULE {
    fn new(hmodule: HMODULE) -> Self {
        Self(hmodule)
    }

    fn get(&self) -> HMODULE {
        self.0
    }
}

struct ClipboardManager {
    hwnd: SafeHWND,
}

impl ClipboardManager {
    fn new(hwnd: HWND) -> Self {
        Self {
            hwnd: SafeHWND::new(hwnd)
        }
    }

    fn get_text(&self) -> Option<String> {
        unsafe {
            if OpenClipboard(self.hwnd.get()) == 0 {
                return None;
            }

            let handle = GetClipboardData(CF_UNICODETEXT);
            if handle.is_null() {
                CloseClipboard();
                return None;
            }

            let ptr = GlobalLock(handle as HANDLE);
            if ptr.is_null() {
                CloseClipboard();
                return None;
            }

            let wide_str = std::slice::from_raw_parts(ptr as *const u16, {
                let mut len = 0;
                let mut p = ptr as *const u16;
                while *p != 0 {
                    len += 1;
                    p = p.add(1);
                }
                len
            });

            let result = String::from_utf16(wide_str).ok();

            GlobalUnlock(handle as HANDLE);
            CloseClipboard();

            result
        }
    }
}

struct PayloadContext {
    painter: egui_glow::Painter,
    egui_ctx: Context,
    dimensions: [u32; 2],
    _glow_context: std::sync::Arc<glow::Context>,
    our_gl_context: SafeGLContext,
    game_gl_context: SafeGLContext,
    start_time: Instant,
    last_frame_time: Option<Instant>,
    input_events: Vec<Event>,
    mouse_pos: Pos2,
    clipboard: ClipboardManager,
    // UI state
    new_username: String,
    new_player_id: String,
    new_access_token: String,
    new_session_type: String,
    status_message: String,
}

impl PayloadContext {
    fn add_event(&mut self, event: Event) {
        self.input_events.push(event);
    }

    fn render(&mut self, hdc: HDC) -> Result<(), String> {
        let window = unsafe { WindowFromDC(hdc) };
        if window.is_null() {
            return Err("Failed to get window from DC".to_string());
        }

        let mut rect = winapi::shared::windef::RECT::default();
        unsafe { GetClientRect(window, &mut rect) };

        let width = (rect.right - rect.left) as u32;
        let height = (rect.bottom - rect.top) as u32;

        if width == 0 || height == 0 {
            return Err("Invalid window dimensions".to_string());
        }

        if width != self.dimensions[0] || height != self.dimensions[1] {
            self.dimensions = [width, height];
        }

        unsafe {
            if wglMakeCurrent(hdc, self.our_gl_context.get()) == 0 {
                return Err("Failed to make our OpenGL context current".to_string());
            }
        }

        let egui_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let current_time = Instant::now();

            let time_since_start = if let Some(last_time) = self.last_frame_time {
                let elapsed = current_time.duration_since(last_time);
                if elapsed > Duration::from_millis(100) {
                    Duration::from_millis(16)
                } else {
                    elapsed
                }
            } else {
                Duration::from_millis(16)
            };

            let time_since_start_total = current_time.duration_since(self.start_time);
            self.last_frame_time = Some(current_time);

            let mut raw_input = RawInput::default();
            raw_input.time = Some(time_since_start_total.as_secs_f64());
            raw_input.predicted_dt = time_since_start.as_secs_f32();
            raw_input.screen_rect = Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(width as f32, height as f32),
            ));

            raw_input.events = self.input_events.drain(..).collect();

            let egui::FullOutput {
                platform_output: _platform_output,
                textures_delta,
                pixels_per_point,
                viewport_output: _,
                shapes,
            } = self.egui_ctx.run(raw_input, |ctx| {
                egui::Window::new("🎮 Minecraft Session Changer")
                    .default_size([500.0, 400.0])
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.heading("Current Session");
                        ui.separator();

                        let session_manager = get_minecraft_session();
                        let current_session = session_manager.get_current_session();

                        ui.horizontal(|ui| {
                            ui.label("Username:");
                            ui.colored_label(egui::Color32::LIGHT_BLUE, &current_session.username);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Player ID:");
                            ui.colored_label(egui::Color32::LIGHT_GREEN, &current_session.player_id);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Access Token:");
                            let token_display = if current_session.access_token.len() > 16 {
                                format!("{}...", &current_session.access_token[..16])
                            } else {
                                current_session.access_token.clone()
                            };
                            ui.colored_label(egui::Color32::GRAY, token_display);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Session Type:");
                            ui.colored_label(egui::Color32::YELLOW, &current_session.session_type);
                        });

                        ui.separator();

                        if ui.button("🔄 Refresh Session").clicked() {
                            match session_manager.refresh_session() {
                                Ok(_) => self.status_message = "✅ Session refreshed successfully".to_string(),
                                Err(e) => self.status_message = format!("❌ Failed to refresh: {}", e),
                            }
                        }

                        ui.separator();
                        ui.heading("Change Session");

                        ui.horizontal(|ui| {
                            ui.label("New Username:");
                            ui.text_edit_singleline(&mut self.new_username);
                        });

                        ui.horizontal(|ui| {
                            ui.label("New Player ID:");
                            ui.text_edit_singleline(&mut self.new_player_id);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Access Token:");
                            ui.text_edit_singleline(&mut self.new_access_token);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Session Type:");
                            egui::ComboBox::from_label("")
                                .selected_text(&self.new_session_type)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.new_session_type, "mojang".to_string(), "Mojang");
                                    ui.selectable_value(&mut self.new_session_type, "legacy".to_string(), "Legacy");
                                });
                        });

                        ui.horizontal(|ui| {
                            if ui.button("📋 Copy Current").clicked() {
                                self.new_username = current_session.username.clone();
                                self.new_player_id = current_session.player_id.clone();
                                self.new_access_token = current_session.access_token.clone();
                                self.new_session_type = current_session.session_type.clone();
                            }

                            if ui.button("🔄 Apply Changes").clicked() {
                                let new_session = SessionInfo {
                                    username: self.new_username.clone(),
                                    player_id: self.new_player_id.clone(),
                                    access_token: self.new_access_token.clone(),
                                    session_type: self.new_session_type.clone(),
                                };

                                match session_manager.change_session(new_session) {
                                    Ok(_) => self.status_message = "✅ Session changed successfully".to_string(),
                                    Err(e) => self.status_message = format!("❌ Failed to change session: {}", e),
                                }
                            }

                            if ui.button("🗑️ Clear Fields").clicked() {
                                self.new_username.clear();
                                self.new_player_id.clear();
                                self.new_access_token.clear();
                                self.new_session_type = "legacy".to_string();
                            }
                        });

                        ui.separator();

                        if !self.status_message.is_empty() {
                            ui.label(&self.status_message);
                        }

                        ui.separator();
                        ui.label("💡 Tips:");
                        ui.label("• Use 'Refresh Session' to load current game session");
                        ui.label("• Player ID should be in UUID format");
                        ui.label("• Press INSERT to toggle this menu");
                        ui.label("• Changes apply immediately to the game");
                        ui.label("• Use Ctrl+C/Ctrl+V to copy/paste text");

                        ui.separator();
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            if SHOULD_UNLOAD.load(Ordering::Relaxed) {
                                ui.colored_label(egui::Color32::YELLOW, "🔄 Unloading...");
                            } else if ui.button("🚪 Unload DLL").clicked() {
                                self.status_message = "🔄 Initiating safe unload...".to_string();
                                initiate_unload();
                            }
                        });
                    });
            });

            let clipped_primitives = self.egui_ctx.tessellate(shapes, pixels_per_point);

            self.painter.paint_and_update_textures(
                self.dimensions,
                pixels_per_point,
                &clipped_primitives,
                &textures_delta,
            );

            Ok(())
        }));

        unsafe {
            if wglMakeCurrent(hdc, self.game_gl_context.get()) == 0 {
                return Err("Failed to restore game OpenGL context".to_string());
            }
        }

        match egui_result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Panic in egui rendering".to_string()),
        }
    }
}

fn initiate_unload() {
    SHOULD_UNLOAD.store(true, Ordering::Relaxed);
    MENU_VISIBLE.store(false, Ordering::Relaxed);
}

unsafe fn cleanup_resources() {
    let window_ptr = CURRENT_WINDOW.load(Ordering::Relaxed);
    if window_ptr != 0 {
        let window = window_ptr as HWND;
        let original_proc = ORIGINAL_WNDPROC.load(Ordering::Relaxed);
        if original_proc != 0 {
            SetWindowLongPtrW(window, GWLP_WNDPROC, original_proc);
        }
    }

    if let Some(detour) = SWAP_BUFFERS.get() {
        let _ = detour.disable();
    }

    if let Some(context_mutex) = CONTEXT.get() {
        if let Some(mut context_guard) = context_mutex.try_lock() {
            *context_guard = None;
        }
    }

    std::thread::sleep(Duration::from_millis(50));
}

fn virtual_key_to_egui_key(vk: i32) -> Option<Key> {
    match vk {
        VK_BACK => Some(Key::Backspace),
        VK_TAB => Some(Key::Tab),
        VK_RETURN => Some(Key::Enter),
        VK_ESCAPE => Some(Key::Escape),
        VK_SPACE => Some(Key::Space),
        VK_LEFT => Some(Key::ArrowLeft),
        VK_UP => Some(Key::ArrowUp),
        VK_RIGHT => Some(Key::ArrowRight),
        VK_DOWN => Some(Key::ArrowDown),
        VK_HOME => Some(Key::Home),
        VK_END => Some(Key::End),
        VK_DELETE => Some(Key::Delete),

        0x30 => Some(Key::Num0),
        0x31 => Some(Key::Num1),
        0x32 => Some(Key::Num2),
        0x33 => Some(Key::Num3),
        0x34 => Some(Key::Num4),
        0x35 => Some(Key::Num5),
        0x36 => Some(Key::Num6),
        0x37 => Some(Key::Num7),
        0x38 => Some(Key::Num8),
        0x39 => Some(Key::Num9),

        0x41 => Some(Key::A),
        0x42 => Some(Key::B),
        0x43 => Some(Key::C),
        0x44 => Some(Key::D),
        0x45 => Some(Key::E),
        0x46 => Some(Key::F),
        0x47 => Some(Key::G),
        0x48 => Some(Key::H),
        0x49 => Some(Key::I),
        0x4A => Some(Key::J),
        0x4B => Some(Key::K),
        0x4C => Some(Key::L),
        0x4D => Some(Key::M),
        0x4E => Some(Key::N),
        0x4F => Some(Key::O),
        0x50 => Some(Key::P),
        0x51 => Some(Key::Q),
        0x52 => Some(Key::R),
        0x53 => Some(Key::S),
        0x54 => Some(Key::T),
        0x55 => Some(Key::U),
        0x56 => Some(Key::V),
        0x57 => Some(Key::W),
        0x58 => Some(Key::X),
        0x59 => Some(Key::Y),
        0x5A => Some(Key::Z),

        0x70 => Some(Key::F1),
        0x71 => Some(Key::F2),
        0x72 => Some(Key::F3),
        0x73 => Some(Key::F4),
        0x74 => Some(Key::F5),
        0x75 => Some(Key::F6),
        0x76 => Some(Key::F7),
        0x77 => Some(Key::F8),
        0x78 => Some(Key::F9),
        0x79 => Some(Key::F10),
        0x7A => Some(Key::F11),
        0x7B => Some(Key::F12),

        0xBD => Some(Key::Minus),
        0xBB => Some(Key::Equals),
        0xDB => Some(Key::OpenBracket),
        0xDD => Some(Key::CloseBracket),
        0xDC => Some(Key::Backslash),
        0xBA => Some(Key::Semicolon),
        0xDE => Some(Key::Quote),
        0xBC => Some(Key::Comma),
        0xBE => Some(Key::Period),
        0xBF => Some(Key::Slash),
        0xC0 => Some(Key::Backtick),

        _ => None,
    }
}

fn get_modifiers() -> Modifiers {
    unsafe {
        Modifiers {
            alt: (GetKeyState(VK_MENU as i32) & 0x8000u16 as i16) != 0,
            ctrl: (GetKeyState(VK_CONTROL as i32) & 0x8000u16 as i16) != 0,
            shift: (GetKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0,
            mac_cmd: false,
            command: (GetKeyState(VK_CONTROL as i32) & 0x8000u16 as i16) != 0,
        }
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if SHOULD_UNLOAD.load(Ordering::Relaxed) {
        let original_proc = ORIGINAL_WNDPROC.load(Ordering::Relaxed);
        if original_proc != 0 {
            return CallWindowProcW(Some(std::mem::transmute(original_proc)), hwnd, msg, wparam, lparam);
        }
        return 0;
    }

    if !MENU_VISIBLE.load(Ordering::Relaxed) {
        let original_proc = ORIGINAL_WNDPROC.load(Ordering::Relaxed);
        if original_proc != 0 {
            return CallWindowProcW(Some(std::mem::transmute(original_proc)), hwnd, msg, wparam, lparam);
        }
    }

    let should_handle = match msg {
        WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MOUSEWHEEL |
        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP | WM_CHAR => true,
        _ => false,
    };

    if should_handle {
        if let Some(context_mutex) = CONTEXT.get() {
            if let Some(mut context_guard) = context_mutex.try_lock() {
                if let Some(context) = context_guard.as_mut() {
                    match msg {
                        WM_MOUSEMOVE => {
                            let x = (lparam & 0xFFFF) as i16 as f32;
                            let y = ((lparam >> 16) & 0xFFFF) as i16 as f32;
                            context.mouse_pos = Pos2::new(x, y);
                            context.add_event(Event::PointerMoved(Pos2::new(x, y)));
                        }
                        WM_LBUTTONDOWN => {
                            let x = (lparam & 0xFFFF) as i16 as f32;
                            let y = ((lparam >> 16) & 0xFFFF) as i16 as f32;
                            context.add_event(Event::PointerButton {
                                pos: Pos2::new(x, y),
                                button: PointerButton::Primary,
                                pressed: true,
                                modifiers: get_modifiers(),
                            });
                        }
                        WM_LBUTTONUP => {
                            let x = (lparam & 0xFFFF) as i16 as f32;
                            let y = ((lparam >> 16) & 0xFFFF) as i16 as f32;
                            context.add_event(Event::PointerButton {
                                pos: Pos2::new(x, y),
                                button: PointerButton::Primary,
                                pressed: false,
                                modifiers: get_modifiers(),
                            });
                        }
                        WM_RBUTTONDOWN => {
                            let x = (lparam & 0xFFFF) as i16 as f32;
                            let y = ((lparam >> 16) & 0xFFFF) as i16 as f32;
                            context.add_event(Event::PointerButton {
                                pos: Pos2::new(x, y),
                                button: PointerButton::Secondary,
                                pressed: true,
                                modifiers: get_modifiers(),
                            });
                        }
                        WM_RBUTTONUP => {
                            let x = (lparam & 0xFFFF) as i16 as f32;
                            let y = ((lparam >> 16) & 0xFFFF) as i16 as f32;
                            context.add_event(Event::PointerButton {
                                pos: Pos2::new(x, y),
                                button: PointerButton::Secondary,
                                pressed: false,
                                modifiers: get_modifiers(),
                            });
                        }
                        WM_MOUSEWHEEL => {
                            let delta = ((wparam >> 16) & 0xFFFF) as i16 as f32 / 120.0;
                            context.add_event(Event::MouseWheel {
                                unit: egui::MouseWheelUnit::Line,
                                delta: Vec2::new(0.0, delta),
                                modifiers: get_modifiers(),
                            });
                        }
                        WM_KEYDOWN | WM_SYSKEYDOWN => {
                            let vk = wparam as i32;
                            let modifiers = get_modifiers();

                            if vk == 0x56 && modifiers.ctrl {
                                if let Some(text) = context.clipboard.get_text() {
                                    context.add_event(Event::Text(text));
                                }
                            }

                            if let Some(key) = virtual_key_to_egui_key(vk) {
                                context.add_event(Event::Key {
                                    key,
                                    physical_key: None,
                                    pressed: true,
                                    repeat: (lparam & 0x40000000) != 0,
                                    modifiers,
                                });
                            }
                        }
                        WM_KEYUP | WM_SYSKEYUP => {
                            let vk = wparam as i32;
                            if let Some(key) = virtual_key_to_egui_key(vk) {
                                context.add_event(Event::Key {
                                    key,
                                    physical_key: None,
                                    pressed: false,
                                    repeat: false,
                                    modifiers: get_modifiers(),
                                });
                            }
                        }
                        WM_CHAR => {
                            let ch = wparam as u32;
                            if let Some(character) = std::char::from_u32(ch) {
                                if character.is_control() {
                                    return 0;
                                }
                                context.add_event(Event::Text(character.to_string()));
                            }
                        }
                        _ => {}
                    }

                    if context.egui_ctx.wants_pointer_input() || context.egui_ctx.wants_keyboard_input() {
                        return 0;
                    }
                }
            }
        }
    }

    let original_proc = ORIGINAL_WNDPROC.load(Ordering::Relaxed);
    if original_proc != 0 {
        CallWindowProcW(Some(std::mem::transmute(original_proc)), hwnd, msg, wparam, lparam)
    } else {
        0
    }
}

unsafe extern "system" fn hk_swap_buffers(hdc: HDC) -> i32 {
    let swap_buffers = SWAP_BUFFERS.get().expect("swap buffers hook not initialized");

    if SHOULD_UNLOAD.load(Ordering::Relaxed) && !UNLOAD_INITIATED.load(Ordering::Relaxed) {
        UNLOAD_INITIATED.store(true, Ordering::Relaxed);

        let result = swap_buffers.call(hdc);

        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(500));
            cleanup_resources();

            std::thread::sleep(Duration::from_millis(200));

            if let Some(dll_handle) = DLL_HANDLE.get() {
                FreeLibraryAndExitThread(dll_handle.get(), 0);
            }
        });

        return result;
    }

    if UNLOAD_INITIATED.load(Ordering::Relaxed) {
        return swap_buffers.call(hdc);
    }

    let frame_count = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
    if frame_count % 30 == 0 {
        let current_key_state = GetAsyncKeyState(VK_INSERT) as u32;
        let last_key_state = LAST_KEY_STATE.load(Ordering::Relaxed);

        if current_key_state != 0 && last_key_state == 0 {
            let new_visibility = !MENU_VISIBLE.load(Ordering::Relaxed);
            MENU_VISIBLE.store(new_visibility, Ordering::Relaxed);
        }

        LAST_KEY_STATE.store(current_key_state, Ordering::Relaxed);
    }

    let current_context = wglGetCurrentContext();
    if current_context.is_null() {
        return swap_buffers.call(hdc);
    }

    if MENU_VISIBLE.load(Ordering::Relaxed) {
        let context_mutex = CONTEXT.get_or_init(|| {
            Mutex::new(create_render_context(hdc).ok())
        });

        if let Some(mut context_guard) = context_mutex.try_lock() {
            if let Some(context) = context_guard.as_mut() {
                let _ = context.render(hdc);
            }
        }
    }

    swap_buffers.call(hdc)
}

unsafe fn create_render_context(hdc: HDC) -> Result<PayloadContext, String> {
    if hdc.is_null() {
        return Err("HDC is null".to_string());
    }

    let window = WindowFromDC(hdc);
    if window.is_null() {
        return Err("Failed to get window from DC".to_string());
    }

    CURRENT_WINDOW.store(window as isize, Ordering::Relaxed);

    let mut dimensions = winapi::shared::windef::RECT::default();
    GetClientRect(window, &mut dimensions);

    let width = (dimensions.right - dimensions.left) as u32;
    let height = (dimensions.bottom - dimensions.top) as u32;

    if width == 0 || height == 0 {
        return Err("Invalid window dimensions".to_string());
    }

    let original_proc = SetWindowLongPtrW(window, GWLP_WNDPROC, window_proc as _);
    if original_proc != 0 {
        ORIGINAL_WNDPROC.store(original_proc, Ordering::Relaxed);
    }

    let game_context = wglGetCurrentContext();
    if game_context.is_null() {
        return Err("No OpenGL context available".to_string());
    }

    let our_context = wglCreateContext(hdc);
    if our_context.is_null() {
        return Err("Failed to create OpenGL context".to_string());
    }

    if wglShareLists(game_context, our_context) == 0 {
        winapi::um::wingdi::wglDeleteContext(our_context);
        return Err("Failed to share OpenGL contexts".to_string());
    }

    if wglMakeCurrent(hdc, our_context) == 0 {
        winapi::um::wingdi::wglDeleteContext(our_context);
        return Err("Failed to make our context current".to_string());
    }

    let glow_context = glow::Context::from_loader_function(|name| {
        let c_str = match std::ffi::CString::new(name) {
            Ok(c) => c,
            Err(_) => return std::ptr::null(),
        };

        let proc_addr = winapi::um::wingdi::wglGetProcAddress(c_str.as_ptr());
        if !proc_addr.is_null() {
            return proc_addr as *const std::ffi::c_void;
        }

        let opengl32 = GetModuleHandleA(b"opengl32.dll\0".as_ptr() as *const i8);
        if !opengl32.is_null() {
            let proc_addr = GetProcAddress(opengl32, c_str.as_ptr());
            if !proc_addr.is_null() {
                return proc_addr as *const std::ffi::c_void;
            }
        }

        std::ptr::null()
    });

    let glow_context = std::sync::Arc::new(glow_context);
    let egui_ctx = Context::default();

    let painter = egui_glow::Painter::new(glow_context.clone(), "", None, false)
        .map_err(|e| {
            winapi::um::wingdi::wglDeleteContext(our_context);
            format!("Failed to create painter: {}", e)
        })?;

    if wglMakeCurrent(hdc, game_context) == 0 {
        winapi::um::wingdi::wglDeleteContext(our_context);
        return Err("Failed to restore game context".to_string());
    }

    Ok(PayloadContext {
        painter,
        egui_ctx,
        dimensions: [width, height],
        _glow_context: glow_context,
        our_gl_context: SafeGLContext::new(our_context),
        game_gl_context: SafeGLContext::new(game_context),
        start_time: Instant::now(),
        last_frame_time: None,
        input_events: Vec::new(),
        mouse_pos: Pos2::ZERO,
        clipboard: ClipboardManager::new(window),
        new_username: String::new(),
        new_player_id: String::new(),
        new_access_token: String::new(),
        status_message: String::new(),
        new_session_type: "mojang".to_string(),
    })
}

fn initialize_logging() {
    use tracing_subscriber::layer::SubscriberExt;

    let stdout_log = tracing_subscriber::fmt::layer()
        .compact()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    let subscriber = tracing_subscriber::Registry::default().with(stdout_log);
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");

    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::new();
        tracing::error!("PANIC: {}", panic_info);
        tracing::error!("Backtrace: {:?}", backtrace);
    }));
}

unsafe extern "system" fn start_routine(_parameter: LPVOID) -> DWORD {
    initialize_logging();

    let gdi32 = GetModuleHandleA(b"gdi32.dll\0".as_ptr() as *const i8);
    if gdi32.is_null() {
        return 1;
    }

    let swap_buffers_addr = GetProcAddress(gdi32, b"SwapBuffers\0".as_ptr() as *const i8);
    if swap_buffers_addr.is_null() {
        return 1;
    }

    let detour = retour::GenericDetour::<FnSwapBuffers>::new(
        std::mem::transmute(swap_buffers_addr),
        hk_swap_buffers,
    ).expect("Failed to create detour");

    detour.enable().expect("Failed to enable detour");

    SWAP_BUFFERS.set(detour).expect("Failed to set swap buffers hook");

    let _ = get_minecraft_session();

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