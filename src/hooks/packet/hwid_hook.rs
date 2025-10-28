use crate::hooks::jhook::{HookCallback};
use jni::sys::{jobject, jvalue};
use jni::sys::{jclass, jmethodID};
use crate::utils::{collect_hwid};

pub struct HwidBytesHook;

impl HookCallback for HwidBytesHook {
    unsafe fn call(
        &self,
        env: *mut jni::sys::JNIEnv,
        _: jobject,
        _: jclass,
        _: jmethodID,
        args: &[jvalue],
    ) -> Option<jvalue> {
        tracing::info!("Hwid bytes called.");

        use jni::objects::{JObject, JString, JValue};
        use jni::JNIEnv;

        if let Ok(mut env) = JNIEnv::from_raw(env) {
            let data_output = JObject::from_raw(args[0].l);
            let hwid = collect_hwid();

            for h in hwid {
                let to_write = format!("\u{1}{}", h);

                let jstr: JString = env.new_string(to_write).expect("Failed to create jstring");
                let jstr_obj: JObject = jstr.into();

                env.call_method(
                    &data_output,
                    "writeUTF",
                    "(Ljava/lang/String;)V",
                    &[JValue::Object(&jstr_obj)],
                ).expect("Failed to call writeUTF");

                env.delete_local_ref(jstr_obj).expect("Failed to delete local ref");
            }
        }
        None
    }
}
