use crate::core::hwid::HWID_SPOOFER;
use crate::hooks::jhook::{HookCallback, HookDecision};
use jni::objects::{JClass, JMethodID, JObject};
use jni::sys::jvalue;
use std::mem;

pub struct HwidHook;

impl HookCallback for HwidHook {
    unsafe fn before(
        &self,
        mut env: jni::JNIEnv,
        _this: &JObject,
        _class: &JClass,
        _orig: &JMethodID,
        args: &[jvalue],
    ) -> HookDecision {
        tracing::info!("Hwid bytes called.");

        let data_output = JObject::from_raw(args[0].l);

        if let Err(e) = HWID_SPOOFER.write_hwid(&mut env, &data_output) {
            tracing::warn!("Failed to write spoofed HWID: {e:#}");
        } else {
            HWID_SPOOFER.notify_success();
        }

        HookDecision::Return(mem::zeroed())
    }
}
