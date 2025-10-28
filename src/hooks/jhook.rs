use crate::jni_hook::{
    JNIHook_Attach, JNIHook_Detach, JNIHook_Init, JNIHook_Shutdown, JNIHookResult,
};
use crate::utils::{SafeJClass, SafeJMethodId, SafePtr};
use anyhow::{Context, Result};
use jni::sys::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::{CString, c_void};
use std::sync::{Arc, Mutex, OnceLock};

pub static HOOK_REGISTRY: Lazy<Arc<Mutex<HookRegistry>>> =
    Lazy::new(|| Arc::new(Mutex::new(HookRegistry::new())));

pub static HOOK_MANAGER: OnceLock<JNIHookManager> = OnceLock::new();

pub struct HookRegistry {
    pub(crate) hooks: HashMap<SafeJMethodId, HookInfo>,
    pub(crate) original_methods: HashMap<SafeJMethodId, SafeJMethodId>,
    pub(crate) classes: HashMap<SafeJMethodId, SafeJClass>,
    pub(crate) native_to_method: HashMap<SafePtr<()>, SafeJMethodId>,
}

pub struct HookInfo {
    callback: Box<dyn HookCallback + Send + Sync>,
    method_signature: String,
    class_name: String,
    method_name: String,
}

impl HookInfo {
    pub fn new(
        callback: Box<dyn HookCallback + Send + Sync>,
        method_signature: String,
        class_name: String,
        method_name: String,
    ) -> Self {
        Self {
            callback,
            method_signature,
            class_name,
            method_name,
        }
    }
}

pub trait HookCallback {
    unsafe fn call(
        &self,
        env: *mut JNIEnv,
        this: jobject,
        class: jclass,
        method: jmethodID,
        args: &[jvalue],
    ) -> Option<jvalue>;
}

impl HookRegistry {
    fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            original_methods: HashMap::new(),
            classes: HashMap::new(),
            native_to_method: HashMap::new(),
        }
    }
}

pub struct JNIHookManager {
    jvm: SafePtr<JavaVM>,
}

impl JNIHookManager {
    pub fn new(jvm: *mut JavaVM) -> Self {
        Self {
            jvm: SafePtr::new(jvm),
        }
    }

    pub fn obtain(jvm: *mut JavaVM) -> &'static Self {
        HOOK_MANAGER.get_or_init(|| unsafe {
            tracing::info!("Initializing JNIHook: {:p}", jvm);
            JNIHook_Init(jvm);
            tracing::info!("JNIHook initialized successfully");
            Self::new(jvm)
        })
    }

    pub unsafe fn hook_method<T: HookCallback + Send + Sync + 'static>(
        &self,
        class_name: &str,
        method_name: &str,
        method_signature: &str,
        callback: T,
    ) -> Result<()> {
        tracing::info!(
            "Attempting to hook {}::{} with signature {}",
            class_name,
            method_name,
            method_signature
        );

        let (class, method_id) = self.find_method(class_name, method_name, method_signature)?;
        tracing::info!(
            "Found method - class: {:p}, method_id: {:p}",
            class,
            method_id
        );

        let native_hook_fn = self.create_method_specific_hook(method_id, method_signature)?;
        tracing::info!("Created hook function: {:p}", native_hook_fn);

        let mut original_method: jmethodID = std::ptr::null_mut();

        let mut env: *mut JNIEnv = std::ptr::null_mut();
        if let Some(get_env) = (**self.jvm.0).GetEnv {
            let env_result = get_env(
                self.jvm.0,
                &mut env as *mut _ as *mut *mut c_void,
                JNI_VERSION_1_8 as i32,
            );

            if env_result != JNI_OK {
                tracing::error!("Failed to get JNI env before hook attach: {}", env_result);
                if env_result == JNI_EDETACHED {
                    if let Some(attach) = (**self.jvm.0).AttachCurrentThread {
                        let attach_result = attach(
                            self.jvm.0,
                            &mut env as *mut _ as *mut *mut c_void,
                            std::ptr::null_mut(),
                        );
                        tracing::info!("AttachCurrentThread result: {}", attach_result);
                    }
                }
            } else {
                tracing::debug!("JNI env obtained successfully: {:p}", env);
            }

            if !env.is_null() {
                let exception_check = (**env).ExceptionCheck.unwrap()(env);
                if exception_check == JNI_TRUE {
                    tracing::warn!("Clearing existing Java exception before hook attach");
                    (**env).ExceptionDescribe.unwrap()(env);
                    (**env).ExceptionClear.unwrap()(env);
                }
            }
        }

        tracing::info!("Calling JNIHook_Attach...");
        let result = JNIHook_Attach(
            method_id,
            native_hook_fn as *mut c_void,
            &mut original_method,
        );
        tracing::info!("JNIHook_Attach returned: {:?}", result);

        if result == JNIHookResult::JnihookErrJavaException && !env.is_null() {
            tracing::error!("=== JAVA EXCEPTION DETAILS ===");

            let exception_check = (**env).ExceptionCheck.unwrap()(env);
            if exception_check == JNI_TRUE {
                tracing::error!("Java exception is present, getting details...");

                (**env).ExceptionDescribe.unwrap()(env);

                let throwable = (**env).ExceptionOccurred.unwrap()(env);
                if !throwable.is_null() {
                    self.log_exception_details(env, throwable);
                }

                (**env).ExceptionClear.unwrap()(env);
            } else {
                tracing::error!(
                    "JNIHookResult indicated Java exception but no exception is present in JNI env"
                );
            }
            tracing::error!("=== END EXCEPTION DETAILS ===");
        }

        match result {
            JNIHookResult::JnihookOk => {
                let mut registry = HOOK_REGISTRY.lock().expect("Failed to lock registry");

                registry.hooks.insert(
                    SafeJMethodId(method_id),
                    HookInfo::new(
                        Box::new(callback),
                        method_signature.to_string(),
                        class_name.to_string(),
                        method_name.to_string(),
                    ),
                );

                registry
                    .original_methods
                    .insert(SafeJMethodId(method_id), SafeJMethodId(original_method));

                registry
                    .classes
                    .insert(SafeJMethodId(method_id), SafeJClass(class));

                registry
                    .native_to_method
                    .insert(SafePtr::new(native_hook_fn as _), SafeJMethodId(method_id));

                tracing::info!("Successfully hooked {}::{}", class_name, method_name);
                Ok(())
            }
            _ => {
                let error_msg = self.get_hook_error_message(result);
                anyhow::bail!("Failed to attach hook: {}", error_msg);
            }
        }
    }

    unsafe fn log_exception_details(&self, env: *mut JNIEnv, throwable: jthrowable) {
        let exception_class = (**env).GetObjectClass.unwrap()(env, throwable);
        if exception_class.is_null() {
            tracing::error!("Failed to get exception class");
            return;
        }

        if let Some(exception_class_name) = self.get_class_name(env, exception_class as jobject) {
            tracing::error!("Exception class: {}", exception_class_name);
        }

        if let Some(message) = self.get_exception_message(env, throwable) {
            tracing::error!("Exception message: {}", message);
        }

        self.log_exception_cause(env, throwable);
    }

    unsafe fn get_class_name(&self, env: *mut JNIEnv, obj: jobject) -> Option<String> {
        let class_obj = (**env).GetObjectClass.unwrap()(env, obj);
        if class_obj.is_null() {
            return None;
        }

        let class_class =
            (**env).FindClass.unwrap()(env, b"java/lang/Class\0".as_ptr() as *const i8);
        if class_class.is_null() {
            return None;
        }

        let get_name_method = (**env).GetMethodID.unwrap()(
            env,
            class_class,
            b"getName\0".as_ptr() as *const i8,
            b"()Ljava/lang/String;\0".as_ptr() as *const i8,
        );
        if get_name_method.is_null() {
            return None;
        }

        let name_obj = (**env).CallObjectMethod.unwrap()(env, class_obj, get_name_method);
        if name_obj.is_null() {
            return None;
        }

        let name_chars = (**env).GetStringUTFChars.unwrap()(env, name_obj, std::ptr::null_mut());
        if name_chars.is_null() {
            return None;
        }

        let class_name = std::ffi::CStr::from_ptr(name_chars)
            .to_string_lossy()
            .into_owned();
        (**env).ReleaseStringUTFChars.unwrap()(env, name_obj, name_chars);

        Some(class_name)
    }

    unsafe fn get_exception_message(
        &self,
        env: *mut JNIEnv,
        throwable: jthrowable,
    ) -> Option<String> {
        let throwable_class =
            (**env).FindClass.unwrap()(env, b"java/lang/Throwable\0".as_ptr() as *const i8);
        if throwable_class.is_null() {
            return None;
        }

        let get_message_method = (**env).GetMethodID.unwrap()(
            env,
            throwable_class,
            b"getMessage\0".as_ptr() as *const i8,
            b"()Ljava/lang/String;\0".as_ptr() as *const i8,
        );
        if get_message_method.is_null() {
            return None;
        }

        let message_obj = (**env).CallObjectMethod.unwrap()(env, throwable, get_message_method);
        if message_obj.is_null() {
            return None;
        }

        let message_chars =
            (**env).GetStringUTFChars.unwrap()(env, message_obj, std::ptr::null_mut());
        if message_chars.is_null() {
            return None;
        }

        let message = std::ffi::CStr::from_ptr(message_chars)
            .to_string_lossy()
            .into_owned();
        (**env).ReleaseStringUTFChars.unwrap()(env, message_obj, message_chars);

        Some(message)
    }

    unsafe fn log_exception_cause(&self, env: *mut JNIEnv, throwable: jthrowable) {
        let throwable_class =
            (**env).FindClass.unwrap()(env, b"java/lang/Throwable\0".as_ptr() as *const i8);
        if throwable_class.is_null() {
            return;
        }

        let get_cause_method = (**env).GetMethodID.unwrap()(
            env,
            throwable_class,
            b"getCause\0".as_ptr() as *const i8,
            b"()Ljava/lang/Throwable;\0".as_ptr() as *const i8,
        );
        if get_cause_method.is_null() {
            return;
        }

        let cause_obj = (**env).CallObjectMethod.unwrap()(env, throwable, get_cause_method);
        if cause_obj.is_null() {
            return;
        }

        tracing::error!("Exception has a cause:");
        if let Some(cause_class_name) = self.get_class_name(env, cause_obj) {
            tracing::error!("  Cause class: {}", cause_class_name);
        }
        if let Some(cause_message) = self.get_exception_message(env, cause_obj as jthrowable) {
            tracing::error!("  Cause message: {}", cause_message);
        }
    }

    unsafe fn create_method_specific_hook(
        &self,
        _method_id: jmethodID,
        method_signature: &str,
    ) -> Result<*const c_void> {
        tracing::info!("Creating hook for signature: {}", method_signature);

        match method_signature {
            "()V" => {
                tracing::info!("Using void_no_args_hook");
                Ok(void_no_args_hook as *const c_void)
            }
            _ if method_signature.ends_with(")V") => {
                tracing::info!(
                    "Using generic void_with_args_hook for: {}",
                    method_signature
                );
                Ok(void_with_args_hook as *const c_void)
            }
            _ => {
                tracing::warn!("Unknown signature pattern: {}", method_signature);
                Ok(void_with_args_hook as *const c_void)
            }
        }
    }

    unsafe fn find_forge_launch_class_loader(&self, env: *mut JNIEnv) -> Option<jobject> {
        let launch_class_name = CString::new("net/minecraft/launchwrapper/Launch").ok()?;
        let launch_class = (**env).FindClass.unwrap()(env, launch_class_name.as_ptr());

        if launch_class.is_null() {
            tracing::warn!("Failed to find Launch class");
            return None;
        }

        let field_name = CString::new("classLoader").ok()?;
        let field_signature =
            CString::new("Lnet/minecraft/launchwrapper/LaunchClassLoader;").ok()?;

        let field_id = (**env).GetStaticFieldID.unwrap()(
            env,
            launch_class,
            field_name.as_ptr(),
            field_signature.as_ptr(),
        );

        if field_id.is_null() {
            tracing::warn!("Failed to find classLoader field");
            return None;
        }

        let class_loader_obj = (**env).GetStaticObjectField.unwrap()(env, launch_class, field_id);

        if class_loader_obj.is_null() {
            tracing::warn!("classLoader field is null");
            return None;
        }

        tracing::debug!("Found Forge LaunchClassLoader: {:p}", class_loader_obj);
        Some(class_loader_obj)
    }

    pub unsafe fn forge_find_class(&self, env: *mut JNIEnv, class_name: &str) -> Option<jclass> {
        let launch_class_loader = self.find_forge_launch_class_loader(env)?;

        let class_name_cstr = CString::new(class_name).ok()?;
        let class_name_jstr = (**env).NewStringUTF.unwrap()(env, class_name_cstr.as_ptr());

        if class_name_jstr.is_null() {
            tracing::warn!("Failed to create jstring for class name: {}", class_name);
            return None;
        }

        let loader_class = (**env).GetObjectClass.unwrap()(env, launch_class_loader);
        if loader_class.is_null() {
            tracing::warn!("Failed to get LaunchClassLoader class");
            (**env).DeleteLocalRef.unwrap()(env, class_name_jstr);
            return None;
        }

        let method_name = CString::new("findClass").ok()?;
        let method_signature = CString::new("(Ljava/lang/String;)Ljava/lang/Class;").ok()?;

        let find_class_method = (**env).GetMethodID.unwrap()(
            env,
            loader_class,
            method_name.as_ptr(),
            method_signature.as_ptr(),
        );

        if find_class_method.is_null() {
            tracing::warn!("Failed to find findClass method");
            (**env).DeleteLocalRef.unwrap()(env, class_name_jstr);
            (**env).DeleteLocalRef.unwrap()(env, loader_class);
            return None;
        }

        let value = jvalue { l: class_name_jstr };
        let class_obj = (**env).CallObjectMethodA.unwrap()(
            env,
            launch_class_loader,
            find_class_method,
            &raw const value,
        );

        let exception_occurred = (**env).ExceptionCheck.unwrap()(env);
        if exception_occurred == JNI_TRUE {
            tracing::warn!("Exception occurred while finding class: {}", class_name);
            (**env).ExceptionDescribe.unwrap()(env);
            (**env).ExceptionClear.unwrap()(env);

            (**env).DeleteLocalRef.unwrap()(env, class_name_jstr);
            (**env).DeleteLocalRef.unwrap()(env, loader_class);
            return None;
        }

        (**env).DeleteLocalRef.unwrap()(env, class_name_jstr);
        (**env).DeleteLocalRef.unwrap()(env, loader_class);

        if class_obj.is_null() {
            tracing::warn!("findClass returned null for: {}", class_name);
            return None;
        }

        tracing::debug!(
            "Found class through Forge: {} -> {:p}",
            class_name,
            class_obj
        );
        Some(class_obj as jclass)
    }

    unsafe fn find_method(
        &self,
        class_name: &str,
        method_name: &str,
        method_signature: &str,
    ) -> Result<(jclass, jmethodID)> {
        let mut env: *mut JNIEnv = std::ptr::null_mut();

        let get_env = (**self.jvm.0).GetEnv.context("GetEnv is null")?;
        get_env(
            self.jvm.0,
            &mut env as *mut _ as *mut *mut c_void,
            JNI_VERSION_1_8 as i32,
        );

        if env.is_null() {
            anyhow::bail!("Failed to get JNI environment");
        }

        let class = self
            .forge_find_class(env, class_name)
            .context(format!("Failed to find class: {}", class_name))?;
        let method_name_cstr = CString::new(method_name)?;
        let method_sig_cstr = CString::new(method_signature)?;

        let method_id = (**env).GetMethodID.context("GetMethodID is null")?(
            env,
            class,
            method_name_cstr.as_ptr(),
            method_sig_cstr.as_ptr(),
        );

        if method_id.is_null() {
            anyhow::bail!("Method not found: {}::{}", class_name, method_name);
        }

        Ok((class, method_id))
    }

    fn get_hook_error_message(&self, result: JNIHookResult) -> &'static str {
        match result {
            JNIHookResult::JnihookErrJniOperation => "JNI operation failed",
            JNIHookResult::JnihookErrJvmtiOperation => "JVMTI operation failed",
            JNIHookResult::JnihookErrClassFileCache => "Class file cache error",
            JNIHookResult::JnihookErrJavaException => "Java exception occurred",
            _ => "Unknown hook error",
        }
    }
}

pub unsafe fn shutdown() -> Result<()> {
    let result = JNIHook_Shutdown();

    match result {
        JNIHookResult::JnihookOk => {
            tracing::info!("JNIHook shutdown successfully");
            Ok(())
        }
        _ => {
            anyhow::bail!("JNIHook shutdown failed");
        }
    }
}

pub unsafe fn unhook_all() -> Result<()> {
    let mut registry = HOOK_REGISTRY.lock().expect("Failed to lock registry");
    for (method, _) in registry.hooks.iter() {
        JNIHook_Detach(method.0);
    }
    registry.hooks.clear();
    registry.original_methods.clear();
    registry.native_to_method.clear();
    tracing::info!("Successfully unhooked all methods");
    Ok(())
}

unsafe extern "C" fn void_no_args_hook(env: *mut JNIEnv, this: jobject) {
    let caller = void_no_args_hook as *const c_void;
    handle_hook_call(env, this, &[], caller);
}

unsafe extern "C" fn void_with_args_hook(env: *mut JNIEnv, this: jobject, arg1: jobject) {
    let caller = void_with_args_hook as *const c_void;
    let args = [jvalue { l: arg1 }];
    handle_hook_call(env, this, &args, caller);
}

unsafe fn handle_hook_call(
    env: *mut JNIEnv,
    this: jobject,
    args: &[jvalue],
    caller: *const c_void,
) -> Option<jvalue> {
    let registry = HOOK_REGISTRY.lock().ok()?;

    if let Some(method_id) = registry.native_to_method.get(&SafePtr(caller as _)) {
        if let Some(hook_info) = registry.hooks.get(method_id) {
            let class = registry.classes.get(method_id)?;
            let original_method = registry
                .original_methods
                .get(method_id)
                .map(|orig| orig.0)
                .unwrap_or(std::ptr::null_mut());

            tracing::debug!(
                "Calling hook for {}::{} with {} args",
                hook_info.class_name,
                hook_info.method_name,
                args.len()
            );

            return hook_info
                .callback
                .call(env, this, class.0, original_method, args);
        }
    }

    tracing::warn!("No hook found for caller: {:p}", caller);
    None
}
