use crate::hooks::jhook::HookCallback;
use crate::utils::generate_hwid;
use jni::objects::{JClass, JMethodID, JObject, JString, JValue};
use jni::sys::{jvalue};

pub struct HwidHook;

impl HookCallback for HwidHook {
    unsafe fn call(
        &self,
        mut env: jni::JNIEnv,
        _: JObject,
        _: JClass,
        _: JMethodID,
        args: &[jvalue],
    ) -> Option<jvalue> {
        tracing::info!("Hwid bytes called.");

        let data_output = JObject::from_raw(args[0].l);
        let hwid = generate_hwid();

        for h in hwid {
            let to_write = format!("\u{1}{}", h);

            let jstr = env.new_string(to_write).expect("idc");
            let jstr_obj = jstr.into();

            env.call_method(
                &data_output,
                "writeUTF",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&jstr_obj)],
            )
                .expect("Failed to call writeUTF");
        }
        Some(std::mem::zeroed())
    }
}
