use crate::jvm::get_jvm;
use jni::JNIEnv;
use jni::objects::{JObject, JValue};

pub struct SoundNotification;

impl SoundNotification {
    pub unsafe fn play_sound(env: &mut JNIEnv) -> anyhow::Result<()> {
        let jvm = get_jvm();

        let minecraft_class = jvm
            .forge_find_class(env, "net/minecraft/client/Minecraft")
            .unwrap();

        let minecraft = env
            .call_static_method(
                minecraft_class,
                "func_71410_x",
                "()Lnet/minecraft/client/Minecraft;",
                &[],
            )?
            .l()?;

        let sound_handler = env
            .get_field(
                &minecraft,
                "field_147127_av",
                "Lnet/minecraft/client/audio/SoundHandler;",
            )?
            .l()?;

        if sound_handler.is_null() {
            tracing::warn!("SoundHandler is null");
            return Ok(());
        }

        let sound_events_class = jvm
            .forge_find_class(env, "net/minecraft/init/SoundEvents")
            .unwrap();

        let sound_event = env
            .get_static_field(
                sound_events_class,
                "field_187802_bS",
                "Lnet/minecraft/util/SoundEvent;",
            )?
            .l()?;

        let sound_record_class = jvm
            .forge_find_class(env, "net/minecraft/client/audio/PositionedSoundRecord")
            .unwrap();

        let sound_record = env
            .call_static_method(
                sound_record_class,
                "func_184371_a",
                "(Lnet/minecraft/util/SoundEvent;F)Lnet/minecraft/client/audio/PositionedSoundRecord;",
                &[JValue::Object(&sound_event), JValue::Float(1.0)],
            )?
            .l()?;

        env.call_method(
            &sound_handler,
            "func_147682_a",
            "(Lnet/minecraft/client/audio/ISound;)V",
            &[JValue::Object(&sound_record)],
        )?;

        tracing::info!("Sound played successfully");

        Ok(())
    }

    pub fn is_local_message(text: &str) -> bool {
        text.contains("[L]")
    }
}
