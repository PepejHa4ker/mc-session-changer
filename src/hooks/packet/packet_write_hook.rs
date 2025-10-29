use crate::hooks::jhook::HookCallback;
use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JMethodID, JObject, JString, JValue};
use jni::sys::jvalue;

pub struct PacketWriteHook;

impl HookCallback for PacketWriteHook {
    unsafe fn call(
        &self,
        mut env: JNIEnv,
        _this: JObject,
        _class: JClass,
        _original_method: JMethodID,
        args: &[jvalue],
    ) -> Option<jvalue> {
        let packet = JObject::from_raw(args[1].l);

        if let Err(e) = self.log_pre_encode(&mut env, &packet) {
            tracing::error!("pre-encode log failed: {e}");
        }
        None
    }
}

impl PacketWriteHook {
    unsafe fn log_pre_encode(&self, env: &mut JNIEnv, packet: &JObject) -> anyhow::Result<()> {
        if packet.is_null() {
            return Ok(());
        }

        env.push_local_frame(64)?;

        let cls_obj = env
            .call_method(packet, "getClass", "()Ljava/lang/Class;", &[])?
            .l()?;
        let name_obj = env
            .call_method(&cls_obj, "getName", "()Ljava/lang/String;", &[])?
            .l()?;
        let name_str: String = env.get_string(&JString::from(name_obj))?.into();

        if name_str == "net.minecraft.network.play.client.C03PacketPlayer" {
            let _ = env.pop_local_frame(&JObject::null());
            return Ok(());
        }

        let unpooled = env.find_class("io/netty/buffer/Unpooled")?;
        let bytebuf = env
            .call_static_method(
                &unpooled,
                "buffer",
                "(I)Lio/netty/buffer/ByteBuf;",
                &[JValue::Int(256)],
            )?
            .l()?;

        let pb_cls = env.find_class("net/minecraft/network/PacketBuffer")?;
        let packet_buffer = env.new_object(
            &pb_cls,
            "(Lio/netty/buffer/ByteBuf;)V",
            &[JValue::Object(&bytebuf)],
        )?;

        env.call_method(
            packet,
            "func_148840_b",
            "(Lnet/minecraft/network/PacketBuffer;)V",
            &[JValue::Object(&packet_buffer)],
        )?;

        let readable = env
            .call_method(&bytebuf, "readableBytes", "()I", &[])?
            .i()? as usize;
        let mut preview: Vec<u8> = Vec::new();
        if readable > 0 {
            let dst: JByteArray = env.new_byte_array(readable as i32)?.into();
            let reader_idx = env.call_method(&bytebuf, "readerIndex", "()I", &[])?.i()?;
            env.call_method(
                &bytebuf,
                "getBytes",
                "(I[B)Lio/netty/buffer/ByteBuf;",
                &[JValue::Int(reader_idx), JValue::Object(&dst)],
            )?;
            let bytes = env.convert_byte_array(dst)?;
            preview.extend_from_slice(&bytes);
        }

        tracing::info!(
            "[PRE-ENCODE] {}: len={} data={:02X?}",
            name_str,
            preview.len(),
            &preview[..preview.len().min(32)]
        );

        let _ = env.call_method(&bytebuf, "release", "()Z", &[]);
        let _ = env.pop_local_frame(&JObject::null());
        Ok(())
    }

    fn limit_str(s: &str, max: usize) -> String {
        if s.len() <= max {
            s.to_string()
        } else {
            let mut cut = s.chars().take(max).collect::<String>();
            cut.push_str("…");
            cut
        }
    }
}
