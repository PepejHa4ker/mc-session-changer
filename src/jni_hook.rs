use std::ffi::c_void;
use jni::sys::{jmethodID, JavaVM};

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum JNIHookResult {
    JnihookOk = 0,
    JnihookErrGetJni = 1,
    JnihookErrGetJvmti = 2,
    JnihookErrAddJvmtiCaps = 3,
    JnihookErrSetupClassFileLoadHook = 4,
    JnihookErrJniOperation = 5,
    JnihookErrJvmtiOperation = 6,
    JnihookErrClassFileCache = 7,
    JnihookErrJavaException = 8,
}

#[link(name = "jnihook", kind = "static")]
unsafe extern "C" {
    pub fn JNIHook_Init(jvm: *mut JavaVM) -> JNIHookResult;

    pub fn JNIHook_Attach(
        method: jmethodID,
        native_hook_method: *mut c_void,
        original_method: *mut jmethodID,
    ) -> JNIHookResult;

    pub fn JNIHook_Detach(method: jmethodID) -> JNIHookResult;

    pub fn JNIHook_Shutdown() -> JNIHookResult;
}