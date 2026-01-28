use crate::hooks::jhook::{HookAfter, HookCallback, HookDecision};
use jni::objects::{JClass, JMethodID, JObject, JString, JValue};
use jni::sys::{jvalue, jint};
use jni::JNIEnv;
use std::fs::File;
use std::io::Write;

pub struct ModListHook;

impl HookCallback for ModListHook {
    unsafe fn before(
        &self,
        _env: JNIEnv,
        _this: &JObject,
        _class: &JClass,
        _orig: &JMethodID,
        _args: &[jvalue],
    ) -> HookDecision {
        HookDecision::CallOriginal
    }

    unsafe fn after(
        &self,
        mut env: JNIEnv,
        _this: &JObject,
        _class: &JClass,
        _orig: &JMethodID,
        _args: &[jvalue],
        ret: jvalue,
        _exception: jni::sys::jobject,
    ) -> HookAfter {
        let mod_list = JObject::from_raw(ret.l);

        if mod_list.is_null() {
            tracing::warn!("ModList returned null");
            return HookAfter::Keep;
        }

        if let Err(e) = write_mods_to_file(&mut env, &mod_list) {
            tracing::error!("Failed to write mods to file: {:#}", e);
        }

        HookAfter::Keep
    }
}

unsafe fn write_mods_to_file(env: &mut JNIEnv, mod_list: &JObject) -> anyhow::Result<()> {
    env.push_local_frame(128)?;

    let size_obj = env.call_method(mod_list, "size", "()I", &[])?;
    let size: jint = size_obj.i()?;

    let mut file = File::create("mods.txt")?;

    for i in 0..size {
        let mod_container_obj = env.call_method(
            mod_list,
            "get",
            "(I)Ljava/lang/Object;",
            &[JValue::Int(i)],
        )?.l()?;

        if mod_container_obj.is_null() {
            continue;
        }

        let mod_id_obj = env.call_method(
            &mod_container_obj,
            "getModId",
            "()Ljava/lang/String;",
            &[],
        )?.l()?;

        let version_obj = env.call_method(
            &mod_container_obj,
            "getVersion",
            "()Ljava/lang/String;",
            &[],
        )?.l()?;

        if mod_id_obj.is_null() || version_obj.is_null() {
            continue;
        }

        let mod_id_jstr = JString::from(mod_id_obj);
        let version_jstr = JString::from(version_obj);

        let mod_id: String = env.get_string(&mod_id_jstr)?.into();
        let version: String = env.get_string(&version_jstr)?.into();

        writeln!(file, "{}:{}", mod_id, version)?;
    }

    tracing::info!("Successfully wrote {} mods to mods.txt", size);

    env.pop_local_frame(&JObject::null())?;
    Ok(())
}
