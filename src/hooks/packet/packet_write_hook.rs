use crate::graphics::netlog::PacketDirection;
use crate::hooks::jhook::{HookCallback, HookDecision};
use crate::hooks::packet::utils::{
    new_packet_buffer, packet_class_name, push_packet_log, read_all_bytes,
};
use jni::objects::{JClass, JMethodID, JObject, JString, JValue};
use jni::sys::jvalue;
use jni::JNIEnv;

pub struct PacketWriteHook;

impl HookCallback for PacketWriteHook {
    unsafe fn before(
        &self,
        env: JNIEnv,
        _this: &JObject,
        _class: &JClass,
        _orig: &JMethodID,
        args: &[jvalue],
    ) -> HookDecision {
        let packet: JObject = JObject::from_raw(args[1].l);

        // match self.should_block(&mut env, &packet) {
        //     Ok(true) => {
        //         tracing::info!("[BLOCK] Skipping outbound CustomPayload on channel 'dwcity'");
        //         return HookDecision::Return(jvalue { l: ptr::null_mut() });
        //     }
        //     Ok(false) => {}
        //     Err(e) => {
        //         tracing::error!("should_block failed: {e}");
        //     }
        // }

        let mut env_local = env.unsafe_clone();
        if let Err(e) = self.log_pre_encode(&mut env_local, &packet) {
            tracing::error!("pre-encode log failed: {e}");
        }
        HookDecision::CallOriginal
    }
}

impl PacketWriteHook {
    unsafe fn should_block(
        &self,
        env: &mut JNIEnv,
        packet: &JObject,
    ) -> anyhow::Result<bool> {
        if packet.is_null() {
            return Ok(false);
        }

        env.push_local_frame(64)?;

        let name_str = {
            packet_class_name(env, packet)?
        };
        if name_str != "net.minecraft.network.play.client.C17PacketCustomPayload" {
            let _ = env.pop_local_frame(&JObject::null());
            return Ok(false);
        }

        let packet_buffer = new_packet_buffer(env, 256)?;


        env.call_method(
            packet,
            "func_148840_b",
            "(Lnet/minecraft/network/PacketBuffer;)V",
            &[JValue::Object(&packet_buffer)],
        )?;

        let chan = env
            .call_method(packet, "func_149559_c", "()Ljava/lang/String;", &[])?
            .l()?;

        let string = JString::from(chan);
        let chan: String = env.get_string(&string)?.into();

        let _ = env.call_method(&packet_buffer, "release", "()Z", &[]);
        let _ = env.pop_local_frame(&JObject::null());

        Ok(chan != "FML|HS")
    }

    unsafe fn log_pre_encode(
        &self,
        env: &mut JNIEnv,
        packet: &JObject,
    ) -> anyhow::Result<()> {
        if packet.is_null() {
            return Ok(());
        }

        env.push_local_frame(64)?;

        let name_str = {
            packet_class_name(env, packet)?
        };
        if name_str == "net.minecraft.network.play.client.C03PacketPlayer" {
            let _ = env.pop_local_frame(&JObject::null());
            return Ok(());
        }

        let packet_buffer = new_packet_buffer(env, 256)?;

        env.call_method(
            packet,
            "func_148840_b",
            "(Lnet/minecraft/network/PacketBuffer;)V",
            &[JValue::Object(&packet_buffer)],
        )?;

        let mut env_local = env.unsafe_clone();
        let preview = read_all_bytes(&mut env_local, &packet_buffer)?;
        push_packet_log(PacketDirection::Outbound, name_str.clone(), preview.clone());

        tracing::debug!(
            "[PRE-ENCODE] {}: len={} data={:02X?}",
            name_str,
            preview.len(),
            &preview
        );

        let _ = env.call_method(&packet_buffer, "release", "()Z", &[]);
        let _ = env.pop_local_frame(&JObject::null());
        Ok(())
    }
}
