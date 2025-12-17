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

        let resource_location_class = jvm
            .forge_find_class(env, "net/minecraft/util/ResourceLocation")
            .unwrap();

        let sound_name = env.new_string("random.orb")?;
        let resource_location = env.new_object(
            resource_location_class,
            "(Ljava/lang/String;)V",
            &[JValue::Object(&JObject::from(sound_name))],
        )?;

        let sound_record_class = jvm
            .forge_find_class(env, "net/minecraft/client/audio/PositionedSoundRecord")
            .unwrap();

        let sound_record = env
            .call_static_method(
                sound_record_class,
                "func_147673_a",
                "(Lnet/minecraft/util/ResourceLocation;)Lnet/minecraft/client/audio/PositionedSoundRecord;",
                &[JValue::Object(&resource_location)],
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
