use jni::sys::{jclass, jmethodID, JavaVM};
use winapi::{
    shared::windef::{HGLRC, HWND},
    shared::minwindef::HMODULE
};

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
    pub const fn new(hmodule: HMODULE) -> Self {
        Self(hmodule)
    }

    pub fn get(&self) -> HMODULE {
        self.0
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SafeJMethodId(pub jmethodID);

unsafe impl Send for SafeJMethodId {}
unsafe impl Sync for SafeJMethodId {}

impl SafeJMethodId {
    pub const fn new(jmethod: jmethodID) -> Self {
        Self(jmethod)
    }

    pub fn get(&self) -> jmethodID {
        self.0
    }
}
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SafeJClass(pub jclass);

unsafe impl Send for SafeJClass {}
unsafe impl Sync for SafeJClass {}

impl SafeJClass {
    pub const fn new(class: jclass) -> Self {
        Self(class)
    }

    pub fn get(&self) -> jclass {
        self.0
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SafePtr<T>(pub *mut T);

unsafe impl<T> Send for SafePtr<T> {}
unsafe impl<T> Sync for SafePtr<T> {}

impl<T> SafePtr<T> {
    pub const fn new(t: *mut T) -> Self {
        Self(t)
    }

    pub fn get(&self) -> *mut T {
        self.0
    }
}
