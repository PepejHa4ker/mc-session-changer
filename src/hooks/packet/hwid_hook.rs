use crate::hooks::jhook::{HookCallback, HookDecision};
use jni::objects::{JClass, JMethodID, JObject, JString, JValue};
use jni::sys::jvalue;
use jni::JNIEnv;

pub struct HwidHook;

impl HookCallback for HwidHook {
    unsafe fn before(
        &self,
        mut env: jni::JNIEnv,
        this: &JObject,
        _class: &JClass,
        _orig: &JMethodID,
        _args: &[jvalue],
    ) -> HookDecision {
        tracing::info!("=== HWID Collection Started ===");

        if let Err(e) = log_hwid_components(&mut env, this) {
            tracing::warn!("Failed to log HWID components: {e:#}");
        }

        tracing::info!("=== HWID Collection Finished ===");
        
        HookDecision::CallOriginal
    }
}

unsafe fn log_hwid_components(env: &mut JNIEnv, pg_instance: &JObject) -> anyhow::Result<()> {
    let system_class = env.find_class("java/lang/System")?;
    let os_name_key = env.new_string("os.name")?;
    let os_name = env
        .call_static_method(
            &system_class,
            "getProperty",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(&os_name_key.into())],
        )?
        .l()?;
    let os_name_str = if os_name.is_null() {
        "<null>".to_string()
    } else {
        env.get_string(&JString::from(os_name))?.into()
    };
    tracing::info!("[HWID] os.name = {:?}", os_name_str);

    let system_info_class = env.find_class("oshi/SystemInfo")?;
    let system_info = env.new_object(&system_info_class, "()V", &[])?;

    let hardware = env
        .call_method(&system_info, "getHardware", "()Loshi/hardware/HardwareAbstractionLayer;", &[])?
        .l()?;

    let hostname = call_string_method(env, pg_instance, "enum", "()Ljava/lang/String;", &[])?;
    tracing::info!("[HWID] hostname = {:?}", hostname);

    let processor = call_string_method(
        env,
        pg_instance,
        "do",
        "(Loshi/hardware/HardwareAbstractionLayer;)Ljava/lang/String;",
        &[JValue::Object(&hardware)],
    )?;
    tracing::info!("[HWID] processor = {:?}", processor);

    let hw_uuid = call_string_method(
        env,
        pg_instance,
        "if",
        "(Loshi/hardware/HardwareAbstractionLayer;)Ljava/lang/String;",
        &[JValue::Object(&hardware)],
    )?;
    tracing::info!("[HWID] hardwareUUID = {:?}", hw_uuid);

    let graphics = call_string_method(
        env,
        pg_instance,
        "for",
        "(Loshi/hardware/HardwareAbstractionLayer;)Ljava/lang/String;",
        &[JValue::Object(&hardware)],
    )?;
    tracing::info!("[HWID] graphics = {:?}", graphics);

    let disks = call_string_method(
        env,
        pg_instance,
        "int",
        "(Loshi/hardware/HardwareAbstractionLayer;)Ljava/lang/String;",
        &[JValue::Object(&hardware)],
    )?;
    tracing::info!("[HWID] disks = {:?}", disks);

    let mac = call_string_method(env, pg_instance, "long", "()Ljava/lang/String;", &[])?;
    tracing::info!("[HWID] mac = {:?}", mac);

    Ok(())
}

unsafe fn call_string_method(
    env: &mut JNIEnv,
    obj: &JObject,
    method: &str,
    sig: &str,
    args: &[JValue],
) -> anyhow::Result<String> {
    let result = env.call_method(obj, method, sig, args)?.l()?;
    if result.is_null() {
        return Ok("<null>".to_string());
    }
    let jstr = JString::from(result);
    Ok(env.get_string(&jstr)?.into())
}
