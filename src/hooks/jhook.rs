use crate::jni_hook::{JNIHook_Attach, JNIHook_Detach, JNIHook_Init, JNIHook_Shutdown, JNIHookResult};
use crate::utils::{SafeJClass, SafeJMethodId, SafePtr};
use anyhow::{bail, Context, Result};
use jni::sys::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::{c_void, CString};
use std::os::raw::c_void as raw_void;
use std::ptr;
use std::sync::{Arc, Mutex, OnceLock};
use jni::objects::{JClass, JMethodID, JObject};
use libffi::{
    low as ffi_low,
    raw as ffi_raw,
    middle::{Cif, Type},
};

pub static HOOK_REGISTRY: Lazy<Arc<Mutex<HookRegistry>>> =
    Lazy::new(|| Arc::new(Mutex::new(HookRegistry::new())));
pub static HOOK_MANAGER: OnceLock<JNIHookManager> = OnceLock::new();

pub struct HookRegistry {
    pub(crate) hooks: HashMap<SafeJMethodId, HookInfo>,
    pub(crate) original_methods: HashMap<SafeJMethodId, SafeJMethodId>,
    pub(crate) classes: HashMap<SafeJMethodId, SafeJClass>,
    trampolines: HashMap<SafeJMethodId, NativeTrampoline>,
}
unsafe impl Send for HookRegistry {}
unsafe impl Sync for HookRegistry {}

pub struct HookInfo {
    callback: Arc<dyn HookCallback + Send + Sync>,
    is_static: bool,
}

impl HookInfo {
    fn new(callback: Arc<dyn HookCallback + Send + Sync>, is_static: bool) -> Self {
        Self {
            callback,
            is_static,
        }
    }
}

pub trait HookCallback {
    unsafe fn call(
        &self,
        env: jni::JNIEnv,
        this: JObject,
        class: JClass,
        original_method: JMethodID,
        args: &[jvalue],
    ) -> Option<jvalue>;
}

impl HookRegistry {
    fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            original_methods: HashMap::new(),
            classes: HashMap::new(),
            trampolines: HashMap::new(),
        }
    }
}

pub struct JNIHookManager {
    jvm: SafePtr<JavaVM>,
}
impl JNIHookManager {
    pub fn new(jvm: *mut JavaVM) -> Self {
        Self { jvm: SafePtr::new(jvm) }
    }
    pub fn obtain(jvm: *mut JavaVM) -> &'static Self {
        HOOK_MANAGER.get_or_init(|| unsafe {
            JNIHook_Init(jvm);
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
        let (class, method_id, is_static) = self.find_method(class_name, method_name, method_signature)?;
        let (tramp, native_hook_fn) = create_universal_trampoline(method_id, method_signature)?;
        let mut original_method: jmethodID = ptr::null_mut();
        let result = JNIHook_Attach(method_id, native_hook_fn as *mut c_void, &mut original_method);
        match result {
            JNIHookResult::JnihookOk => {
                let mut registry = HOOK_REGISTRY.lock().expect("registry");
                registry.hooks.insert(
                    SafeJMethodId(method_id),
                    HookInfo::new(
                        Arc::new(callback),
                        is_static,
                    ),
                );
                registry.original_methods.insert(SafeJMethodId(method_id), SafeJMethodId(original_method));
                registry.classes.insert(SafeJMethodId(method_id), SafeJClass(class));
                registry.trampolines.insert(SafeJMethodId(method_id), tramp);
                Ok(())
            }
            _ => bail!("Failed to attach hook"),
        }
    }

    unsafe fn find_forge_launch_class_loader(&self, env: *mut JNIEnv) -> Option<jobject> {
        let launch_class_name = CString::new("net/minecraft/launchwrapper/Launch").ok()?;
        let launch_class = (**env).FindClass.unwrap()(env, launch_class_name.as_ptr());
        if launch_class.is_null() {
            return None;
        }
        let field_name = CString::new("classLoader").ok()?;
        let field_signature = CString::new("Lnet/minecraft/launchwrapper/LaunchClassLoader;").ok()?;
        let field_id = (**env).GetStaticFieldID.unwrap()(env, launch_class, field_name.as_ptr(), field_signature.as_ptr());
        if field_id.is_null() {
            return None;
        }
        let class_loader_obj = (**env).GetStaticObjectField.unwrap()(env, launch_class, field_id);
        if class_loader_obj.is_null() {
            return None;
        }
        Some(class_loader_obj)
    }

    pub unsafe fn forge_find_class(&self, env: *mut JNIEnv, class_name: &str) -> Option<jclass> {
        let launch_class_loader = self.find_forge_launch_class_loader(env)?;
        let class_name_cstr = CString::new(class_name).ok()?;
        let class_name_jstr = (**env).NewStringUTF.unwrap()(env, class_name_cstr.as_ptr());
        if class_name_jstr.is_null() {
            return None;
        }
        let loader_class = (**env).GetObjectClass.unwrap()(env, launch_class_loader);
        if loader_class.is_null() {
            (**env).DeleteLocalRef.unwrap()(env, class_name_jstr);
            return None;
        }
        let method_name = CString::new("findClass").ok()?;
        let method_signature = CString::new("(Ljava/lang/String;)Ljava/lang/Class;").ok()?;
        let find_class_method = (**env).GetMethodID.unwrap()(env, loader_class, method_name.as_ptr(), method_signature.as_ptr());
        if find_class_method.is_null() {
            (**env).DeleteLocalRef.unwrap()(env, class_name_jstr);
            (**env).DeleteLocalRef.unwrap()(env, loader_class);
            return None;
        }
        let value = jvalue { l: class_name_jstr };
        let class_obj = (**env).CallObjectMethodA.unwrap()(env, launch_class_loader, find_class_method, &value as *const jvalue);
        (**env).DeleteLocalRef.unwrap()(env, class_name_jstr);
        (**env).DeleteLocalRef.unwrap()(env, loader_class);
        if class_obj.is_null() {
            return None;
        }
        Some(class_obj as jclass)
    }

    unsafe fn find_method(
        &self,
        class_name: &str,
        method_name: &str,
        method_signature: &str,
    ) -> Result<(jclass, jmethodID, bool)> {
        let mut env: *mut JNIEnv = ptr::null_mut();
        let get_env = (**self.jvm.0).GetEnv.context("GetEnv is null")?;
        get_env(self.jvm.0, &mut env as *mut _ as *mut *mut c_void, JNI_VERSION_1_8 as i32);
        if env.is_null() {
            bail!("Failed to get JNI environment");
        }

        let local_cls = self
            .forge_find_class(env, class_name)
            .context(format!("Failed to find class: {}", class_name))?;

        let new_global = (**env).NewGlobalRef.unwrap()(env, local_cls as jobject);
        if new_global.is_null() {
            (**env).DeleteLocalRef.unwrap()(env, local_cls as jobject);
            bail!("NewGlobalRef({}) failed", class_name);
        }
        (**env).DeleteLocalRef.unwrap()(env, local_cls as jobject);

        let class = new_global as jclass;

        let method_name_cstr = CString::new(method_name)?;
        let method_sig_cstr = CString::new(method_signature)?;

        let get_static = (**env).GetStaticMethodID.context("GetStaticMethodID is null")?;
        let static_mid = get_static(env, class, method_name_cstr.as_ptr(), method_sig_cstr.as_ptr());
        if !static_mid.is_null() {
            return Ok((class, static_mid, true));
        }

        if (**env).ExceptionCheck.unwrap()(env) == JNI_TRUE {
            (**env).ExceptionClear.unwrap()(env);
        }

        let get_inst = (**env).GetMethodID.context("GetMethodID is null")?;
        let inst_mid = get_inst(env, class, method_name_cstr.as_ptr(), method_sig_cstr.as_ptr());
        if inst_mid.is_null() {
            bail!(
            "Method not found: {}::{} {}",
            class_name, method_name, method_signature
        );
        }
        Ok((class, inst_mid, false))
    }
}

pub unsafe fn shutdown() -> Result<()> {
    match JNIHook_Shutdown() {
        JNIHookResult::JnihookOk => Ok(()),
        _ => bail!("JNIHook shutdown failed"),
    }
}

pub unsafe fn unhook_all() -> Result<()> {
    let mut env_ptr: *mut JNIEnv = ptr::null_mut();
    if let Some(get_env) = (**HOOK_MANAGER.get().unwrap().jvm.0).GetEnv {
        let rc = get_env(
            HOOK_MANAGER.get().unwrap().jvm.0,
            &mut env_ptr as *mut _ as *mut *mut c_void,
            JNI_VERSION_1_8 as i32,
        );
        if rc == JNI_EDETACHED {
            if let Some(attach) = (**HOOK_MANAGER.get().unwrap().jvm.0).AttachCurrentThread {
                let _ = attach(
                    HOOK_MANAGER.get().unwrap().jvm.0,
                    &mut env_ptr as *mut _ as *mut *mut c_void,
                    ptr::null_mut(),
                );
            }
        }
    }

    let mut registry = HOOK_REGISTRY.lock().expect("registry");
    for (method, _) in registry.hooks.iter() {
        let r = JNIHook_Detach(method.0);
        match r {
            JNIHookResult::JnihookOk => {}
            other => {
                tracing::error!("JNIHook_Detach({:p}) => {:?}", method.0, other);
            }
        }
    }

    for (_, tramp) in registry.trampolines.drain() {
        tramp.destroy();
    }
    registry.hooks.clear();
    registry.original_methods.clear();
    for (_mid, cls) in registry.classes.drain() {
        if !env_ptr.is_null() && !cls.0.is_null() {
            unsafe {
                (**env_ptr).DeleteGlobalRef.unwrap()(env_ptr, cls.0 as jobject);
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum JTypeKind {
    Void,
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Object,
}
#[derive(Debug)]
struct ParsedSig {
    args: Vec<JTypeKind>,
    ret: JTypeKind,
}
fn parse_jvm_sig(sig: &str) -> Result<ParsedSig> {
    let b = sig.as_bytes();
    let mut i = 0usize;
    if b.get(i) != Some(&b'(') {
        bail!("bad jni signature");
    }
    i += 1;
    let mut args = Vec::new();
    while i < b.len() && b[i] != b')' {
        let (k, ni) = parse_one_type(b, i)?;
        args.push(k);
        i = ni;
    }
    if i >= b.len() || b[i] != b')' {
        bail!("bad jni signature");
    }
    i += 1;
    if i >= b.len() {
        bail!("bad jni signature");
    }
    let (ret, ni) = parse_one_type(b, i)?;
    if ni != b.len() {
        bail!("bad jni signature");
    }
    Ok(ParsedSig { args, ret })
}
fn parse_one_type(b: &[u8], mut i: usize) -> Result<(JTypeKind, usize)> {
    if i >= b.len() {
        bail!("bad jni signature");
    }
    let c = b[i];
    match c {
        b'V' => Ok((JTypeKind::Void, i + 1)),
        b'Z' => Ok((JTypeKind::Boolean, i + 1)),
        b'B' => Ok((JTypeKind::Byte, i + 1)),
        b'C' => Ok((JTypeKind::Char, i + 1)),
        b'S' => Ok((JTypeKind::Short, i + 1)),
        b'I' => Ok((JTypeKind::Int, i + 1)),
        b'J' => Ok((JTypeKind::Long, i + 1)),
        b'F' => Ok((JTypeKind::Float, i + 1)),
        b'D' => Ok((JTypeKind::Double, i + 1)),
        b'L' => {
            i += 1;
            while i < b.len() && b[i] != b';' {
                i += 1;
            }
            if i >= b.len() {
                bail!("bad jni signature");
            }
            Ok((JTypeKind::Object, i + 1))
        }
        b'[' => {
            i += 1;
            while i < b.len() && b[i] == b'[' {
                i += 1;
            }
            if i >= b.len() {
                bail!("bad jni signature");
            }
            if b[i] == b'L' {
                i += 1;
                while i < b.len() && b[i] != b';' {
                    i += 1;
                }
                if i >= b.len() {
                    bail!("bad jni signature");
                }
                i += 1
            } else {
                i += 1
            }
            Ok((JTypeKind::Object, i))
        }
        _ => bail!("bad jni signature"),
    }
}
fn ffi_type_of(k: JTypeKind) -> Type {
    match k {
        JTypeKind::Void => Type::void(),
        JTypeKind::Boolean | JTypeKind::Byte => Type::i8(),
        JTypeKind::Char => Type::u16(),
        JTypeKind::Short => Type::i16(),
        JTypeKind::Int => Type::i32(),
        JTypeKind::Long => Type::i64(),
        JTypeKind::Float => Type::f32(),
        JTypeKind::Double => Type::f64(),
        JTypeKind::Object => Type::pointer(),
    }
}

struct TrampolineUserData {
    method_id: SafeJMethodId,
    sig: ParsedSig,
}
struct NativeTrampoline {
    _cif: Box<Cif>,
    closure_handle: *mut ffi_low::ffi_closure,
    code_ptr: ffi_low::CodePtr,
    userdata: *mut TrampolineUserData,
}
impl NativeTrampoline {
    fn code_ptr(&self) -> *const c_void {
        self.code_ptr.as_mut_ptr() as *const c_void
    }
    fn destroy(self) {
        unsafe {
            ffi_low::closure_free(self.closure_handle);
            drop(Box::from_raw(self.userdata));
        }
    }
}

unsafe extern "C" fn raw_trampoline(
    _cif: *mut ffi_low::ffi_cif,
    result: *mut raw_void,
    args: *mut *mut raw_void,
    userdata: *mut raw_void,
) {
    let ud = &*(userdata as *const TrampolineUserData);
    let argv = args as *const *const raw_void;
    let env_ptr = *argv.add(0) as *const *mut JNIEnv;
    let env: *mut JNIEnv = *env_ptr;
    let this_ptr = *argv.add(1) as *const jobject;
    let mut jargs: Vec<jvalue> = Vec::with_capacity(ud.sig.args.len());

    for (i, k) in ud.sig.args.iter().enumerate() {
        let ap = *argv.add(2 + i);
        let jv = match k {
            JTypeKind::Boolean => {
                let v = *(ap as *const jboolean);
                jvalue { z: v as u8 }
            }
            JTypeKind::Byte => jvalue { b: *(ap as *const jbyte) },
            JTypeKind::Char => jvalue { c: *(ap as *const jchar) },
            JTypeKind::Short => jvalue { s: *(ap as *const jshort) },
            JTypeKind::Int => jvalue { i: *(ap as *const jint) },
            JTypeKind::Long => jvalue { j: *(ap as *const jlong) },
            JTypeKind::Float => jvalue { f: *(ap as *const jfloat) },
            JTypeKind::Double => jvalue { d: *(ap as *const jdouble) },
            JTypeKind::Object => jvalue { l: *(ap as *const jobject) },
            JTypeKind::Void => jvalue { l: ptr::null_mut() },
        };
        jargs.push(jv);
    }
    let (cb, is_static, class, original) = {
        let registry = HOOK_REGISTRY.lock().expect("registry");
        let h = match registry.hooks.get(&ud.method_id) {
            Some(h) => h,
            None => return,
        };
        let class = registry.classes.get(&ud.method_id).map(|c| c.0).unwrap_or(ptr::null_mut());
        let original = registry.original_methods.get(&ud.method_id).map(|m| m.0).unwrap_or(ptr::null_mut());
        (Arc::clone(&h.callback), h.is_static, class, original)
    };


    let safe_env = jni::JNIEnv::from_raw(env).expect("failed to get JNIEnv from raw pointer");
    let this_obj = JObject::from_raw(*this_ptr);
    let class_obj = JClass::from_raw(class);
    let original_method = JMethodID::from_raw(original);
    let ret_opt = cb.call(safe_env, this_obj, class_obj, original_method, &jargs);
    let result_jv = if let Some(v) = ret_opt {
        v
    } else {
        call_original(env, *this_ptr, class, original, &jargs, is_static, &ud.sig)
    };

    match ud.sig.ret {
        JTypeKind::Void    => {}
        JTypeKind::Boolean => *(result as *mut jboolean) = result_jv.z as jboolean,
        JTypeKind::Byte    => *(result as *mut jbyte)    = result_jv.b,
        JTypeKind::Char    => *(result as *mut jchar)    = result_jv.c,
        JTypeKind::Short   => *(result as *mut jshort)   = result_jv.s,
        JTypeKind::Int     => *(result as *mut jint)     = result_jv.i,
        JTypeKind::Long    => *(result as *mut jlong)    = result_jv.j,
        JTypeKind::Float   => *(result as *mut jfloat)   = result_jv.f,
        JTypeKind::Double  => *(result as *mut jdouble)  = result_jv.d,
        JTypeKind::Object  => *(result as *mut jobject)  = result_jv.l,
    }
}

fn create_universal_trampoline(
    method_id: jmethodID,
    method_signature: &str,
) -> Result<(NativeTrampoline, *const c_void)> {
    let sig = parse_jvm_sig(method_signature)?;
    let mut ffi_args: Vec<Type> = Vec::with_capacity(2 + sig.args.len());
    ffi_args.push(Type::pointer());
    ffi_args.push(Type::pointer());
    for k in &sig.args {
        ffi_args.push(ffi_type_of(*k));
    }
    let ffi_ret = ffi_type_of(sig.ret);
    let cif = Box::new(Cif::new(ffi_args.into_iter(), ffi_ret));
    let (closure_handle, code_ptr) = ffi_low::closure_alloc();
    let userdata = TrampolineUserData {
        method_id: SafeJMethodId(method_id),
        sig,
    };
    let userdata_ptr: *mut TrampolineUserData = Box::into_raw(Box::new(userdata));
    let status = unsafe {
        ffi_raw::ffi_prep_closure_loc(
            closure_handle,
            cif.as_raw_ptr(),
            Some(raw_trampoline),
            userdata_ptr as *mut raw_void,
            code_ptr.as_mut_ptr(),
        )
    };
    if status != ffi_raw::ffi_status_FFI_OK {
        unsafe {
            ffi_low::closure_free(closure_handle);
            drop(Box::from_raw(userdata_ptr));
        }
        bail!("ffi_prep_closure_loc failed: status {}", status);
    }
    let tramp = NativeTrampoline {
        _cif: cif,
        closure_handle,
        code_ptr,
        userdata: userdata_ptr,
    };
    let fn_ptr = tramp.code_ptr();
    Ok((tramp, fn_ptr))
}

unsafe fn call_original(
    env: *mut JNIEnv,
    this_or_cls: jobject,
    class: jclass,
    method: jmethodID,
    args: &[jvalue],
    is_static: bool,
    sig: &ParsedSig,
) -> jvalue {
    macro_rules! call_num {
        ($inst:ident, $stat:ident, $fld:ident, $ty:ty) => {{
            if is_static {
                let v = (**env).$stat.unwrap()(env, class, method, args.as_ptr());
                jvalue { $fld: v as $ty }
            } else {
                let v = (**env).$inst.unwrap()(env, this_or_cls, class, method, args.as_ptr());
                jvalue { $fld: v as $ty }
            }
        }};
    }
    match sig.ret {
        JTypeKind::Void => {
            if is_static {
                (**env).CallStaticVoidMethodA.unwrap()(env, class, method, args.as_ptr());
            } else {
                (**env).CallNonvirtualVoidMethodA.unwrap()(env, this_or_cls, class, method, args.as_ptr());
            }
            jvalue { l: ptr::null_mut() }
        }
        JTypeKind::Boolean => call_num!(CallNonvirtualBooleanMethodA, CallStaticBooleanMethodA, z, jboolean),
        JTypeKind::Byte => call_num!(CallNonvirtualByteMethodA, CallStaticByteMethodA, b, jbyte),
        JTypeKind::Char => call_num!(CallNonvirtualCharMethodA, CallStaticCharMethodA, c, jchar),
        JTypeKind::Short => call_num!(CallNonvirtualShortMethodA, CallStaticShortMethodA, s, jshort),
        JTypeKind::Int => call_num!(CallNonvirtualIntMethodA, CallStaticIntMethodA, i, jint),
        JTypeKind::Long => call_num!(CallNonvirtualLongMethodA, CallStaticLongMethodA, j, jlong),
        JTypeKind::Float => call_num!(CallNonvirtualFloatMethodA, CallStaticFloatMethodA, f, jfloat),
        JTypeKind::Double => call_num!(CallNonvirtualDoubleMethodA, CallStaticDoubleMethodA, d, jdouble),
        JTypeKind::Object => call_num!(CallNonvirtualObjectMethodA, CallStaticObjectMethodA, l, jobject),
    }
}
