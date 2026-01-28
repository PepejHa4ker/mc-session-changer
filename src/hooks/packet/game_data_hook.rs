use crate::hooks::jhook::{HookCallback, HookDecision};
use jni::objects::{JClass, JMethodID, JObject};
use jni::sys::jvalue;
use jni::JNIEnv;

pub struct GameDataHook;

impl HookCallback for GameDataHook {
    unsafe fn before(
        &self,
        mut env: JNIEnv,
        _this: &JObject,
        _class: &JClass,
        _orig: &JMethodID,
        _args: &[jvalue],
    ) -> HookDecision {
        tracing::info!("GameData.injectWorldIDMap called, returning empty list");

        let array_list_class = match env.find_class("java/util/ArrayList") {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to find ArrayList class: {:#}", e);
                return HookDecision::CallOriginal;
            }
        };

        let empty_list = match env.new_object(&array_list_class, "()V", &[]) {
            Ok(obj) => obj,
            Err(e) => {
                tracing::error!("Failed to create ArrayList: {:#}", e);
                return HookDecision::CallOriginal;
            }
        };

        HookDecision::Return(jvalue {
            l: empty_list.into_raw(),
        })
    }
}
