use winapi::{
    um::winuser::{OpenClipboard, CloseClipboard, GetClipboardData, SetClipboardData, EmptyClipboard, CF_UNICODETEXT},
    shared::windef::HWND,
    um::winbase::{GlobalLock, GlobalUnlock, GlobalAlloc, GMEM_MOVEABLE},
    um::winnt::HANDLE
};
use crate::utils::SafeHWND;

pub struct ClipboardManager {
    hwnd: SafeHWND,
}

impl ClipboardManager {
    pub fn new(hwnd: HWND) -> Self {
        Self {
            hwnd: SafeHWND::new(hwnd)
        }
    }

    pub fn get_text(&self) -> Option<String> {
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

    pub fn set_text(&self, text: &str) -> bool {
        unsafe {
            if OpenClipboard(self.hwnd.get()) == 0 {
                return false;
            }

            if EmptyClipboard() == 0 {
                CloseClipboard();
                return false;
            }

            let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let size = wide_text.len() * 2;

            let handle = GlobalAlloc(GMEM_MOVEABLE, size);
            if handle.is_null() {
                CloseClipboard();
                return false;
            }

            let ptr = GlobalLock(handle);
            if ptr.is_null() {
                CloseClipboard();
                return false;
            }

            std::ptr::copy_nonoverlapping(wide_text.as_ptr(), ptr as *mut u16, wide_text.len());
            GlobalUnlock(handle);

            let result = SetClipboardData(CF_UNICODETEXT, handle as HANDLE);
            CloseClipboard();

            !result.is_null()
        }
    }
}