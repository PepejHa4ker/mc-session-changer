use crate::hooks::jhook::{HookAfter, HookCallback};
use crate::mappings::methods;
use anyhow::Context;
use jni::JNIEnv;
use jni::objects::*;
use jni::sys::jvalue;
use crate::core::sound::SoundNotification;

const SOUND_RETRY_LIMIT: usize = 3;

pub struct ChatHook;

impl HookCallback for ChatHook {
    unsafe fn after(
        &self,
        mut env: JNIEnv,
        _this: &JObject,
        _class: &JClass,
        _orig: &JMethodID,
        args: &[jvalue],
        _ret: jvalue,
        _exception: jni::sys::jobject,
    ) -> HookAfter {
        if args.is_empty() {
            return HookAfter::Keep;
        }

        let chat_component = JObject::from_raw(args[0].l);

        if chat_component.is_null() {
            return HookAfter::Keep;
        }

        let message = match get_chat_text(&mut env, &chat_component) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Failed to read chat text: {e:#}");
                return HookAfter::Keep;
            }
        };

        if SoundNotification::is_local_message(&message) {
            for _ in 0..SOUND_RETRY_LIMIT {
                match SoundNotification::play_sound(&mut env) {
                    Ok(_) => break,
                    Err(e) => tracing::error!("Failed to play sound: {e:#}"),
                }
            }
        }

        HookAfter::Keep
    }
}

unsafe fn get_chat_text(env: &mut JNIEnv, chat_component: &JObject) -> anyhow::Result<String> {
    let text = env
        .call_method(
            chat_component,
            methods::CHAT_GET_TEXT,
            "()Ljava/lang/String;",
            &[],
        )
        .context("call chat get text")?
        .l()
        .context("chat get text returned null")?;

    if text.is_null() {
        return Ok(String::new());
    }

    let result: String = env
        .get_string(&JString::from(text))
        .context("get_string on chat text")?
        .into();
    Ok(result)
}
