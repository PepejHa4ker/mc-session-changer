use winapi::shared::windef::{HGLRC, HWND};
use winapi::shared::minwindef::HMODULE;

#[derive(Debug)]
pub struct SafeGLContext(HGLRC);

unsafe impl Send for SafeGLContext {}
unsafe impl Sync for SafeGLContext {}

impl SafeGLContext {
    pub fn new(ctx: HGLRC) -> Self {
        Self(ctx)
    }

    pub fn get(&self) -> HGLRC {
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
pub struct SafeHWND(HWND);

unsafe impl Send for SafeHWND {}
unsafe impl Sync for SafeHWND {}

impl SafeHWND {
    pub fn new(hwnd: HWND) -> Self {
        Self(hwnd)
    }

    pub fn get(&self) -> HWND {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct SafeHMODULE(HMODULE);

unsafe impl Send for SafeHMODULE {}
unsafe impl Sync for SafeHMODULE {}

impl SafeHMODULE {
    pub fn new(hmodule: HMODULE) -> Self {
        Self(hmodule)
    }

    pub fn get(&self) -> HMODULE {
        self.0
    }
}