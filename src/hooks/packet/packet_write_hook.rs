use crate::graphics::netlog::PacketDirection;
use crate::hooks::jhook::{HookCallback, HookDecision};
use crate::hooks::packet::utils::{
    new_packet_buffer, packet_class_name, push_packet_log, read_all_bytes,
};
use crate::mappings::{classes, methods, signatures};
use jni::objects::{JClass, JMethodID, JObject, JString, JValue};
use jni::sys::jvalue;
use jni::JNIEnv;
use std::ptr;

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

        // let mut env_clone = env.unsafe_clone();
        // match self.should_block(&mut env_clone, &packet) {
        //     Ok(Some(channel)) => {
        //         tracing::info!("[BLOCK] Blocking CustomPayload on channel '{}'", channel);
        //         return HookDecision::Return(jvalue { l: ptr::null_mut() });
        //     }
        //     Ok(None) => {}
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
    ) -> anyhow::Result<Option<String>> {
        if packet.is_null() {
            return Ok(None);
        }

        env.push_local_frame(64)?;

        let name_str = packet_class_name(env, packet)?;
        if name_str != classes::CUSTOM_PAYLOAD_CLIENT {
            let _ = env.pop_local_frame(&JObject::null());
            return Ok(None);
        }

        let chan_obj = env
            .call_method(packet, methods::CUSTOM_PAYLOAD_GET_CHANNEL, "()Ljava/lang/String;", &[])?
            .l()?;

        let chan_jstr = JString::from(chan_obj);
        let channel: String = env.get_string(&chan_jstr)?.into();

        let _ = env.pop_local_frame(&JObject::null());

        if channel.starts_with("FML") || channel == "REGISTER" {
            tracing::debug!("[ALLOW] CustomPayload channel: {}", channel);
            Ok(None)
        } else {
            tracing::info!("[BLOCK] CustomPayload channel: {}", channel);
            Ok(Some(channel))
        }
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
        if name_str == classes::PLAYER_PACKET {
            let _ = env.pop_local_frame(&JObject::null());
            return Ok(());
        }

        let packet_buffer = new_packet_buffer(env, 256)?;

        env.call_method(
            packet,
            methods::PACKET_WRITE_DATA,
            signatures::PACKET_WRITE_DATA,
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
