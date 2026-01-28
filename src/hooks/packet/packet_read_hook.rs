use crate::hooks::packet::utils::{packet_class_name, push_packet_log};
use crate::hooks::jhook::{HookAfter, HookCallback};
use crate::mappings::classes;

use jni::JNIEnv;
use jni::objects::*;
use jni::sys::jvalue;

pub struct InboundOutListTap;

impl HookCallback for InboundOutListTap {
    unsafe fn after(
        &self,
        env: JNIEnv,
        _this: &JObject,
        _class: &JClass,
        _orig: &JMethodID,
        args: &[jvalue],
        _ret: jvalue,
        _exception: jni::sys::jobject,
    ) -> HookAfter {
        let in_buf: JObject = JObject::from_raw(args[1].l);
        let out_list: JObject = JObject::from_raw(args[2].l);

        if in_buf.is_null() || out_list.is_null() {
            return HookAfter::Keep;
        }

        let mut env_local = env.unsafe_clone();
        if let Err(e) = log_from_in_and_out(&mut env_local, &in_buf, &out_list) {
            tracing::error!("inbound/out-list log failed: {e:?}");
        }

        HookAfter::Keep
    }
}

unsafe fn log_from_in_and_out(
    env: &mut JNIEnv,
    in_buf: &JObject,
    out_list: &JObject,
) -> anyhow::Result<()> {
    env.push_local_frame(128)?;

    let frame = copy_full_frame_from_bytebuf(env, in_buf)?;
    let pkt_names = collect_packet_class_names(env, out_list)?;

    let (_, id_len) = read_varint_prefix(&frame);
    let payload = if id_len <= frame.len() {
        &frame[id_len..]
    } else {
        &[]
    };

    for name in pkt_names {
        log_record(name, payload.to_vec());
    }

    let _ = env.pop_local_frame(&JObject::null());

    Ok(())
}

fn log_record(name: String, bytes: Vec<u8>) {
    push_packet_log(crate::graphics::netlog::PacketDirection::Inbound, name, bytes);
}

unsafe fn copy_full_frame_from_bytebuf(
    env: &mut JNIEnv,
    bytebuf: &JObject,
) -> anyhow::Result<Vec<u8>> {
    let len = env.call_method(bytebuf, "writerIndex", "()I", &[])?.i()? as i32;
    if len <= 0 {
        return Ok(Vec::new());
    }

    let dst: JByteArray = env.new_byte_array(len)?.into();
    env.call_method(
        bytebuf,
        "getBytes",
        "(I[B)Lio/netty/buffer/ByteBuf;",
        &[JValue::Int(0), JValue::Object(&dst)],
    )?;
    let bytes = env.convert_byte_array(dst)?;
    Ok(bytes)
}

unsafe fn collect_packet_class_names(
    env: &mut JNIEnv,
    out_list: &JObject,
) -> anyhow::Result<Vec<String>> {
    let size = env.call_method(out_list, "size", "()I", &[])?.i()?;
    if size <= 0 {
        return Ok(Vec::new());
    }

    let pkt_cls = env.find_class(classes::PACKET)?;
    let mut names = Vec::with_capacity(size as usize);

    for i in 0..size {
        let pkt = env
            .call_method(out_list, "get", "(I)Ljava/lang/Object;", &[JValue::Int(i)])?
            .l()?;
        if !env.is_instance_of(&pkt, &pkt_cls)? {
            continue;
        }
        let mut env_local = env.unsafe_clone();
        names.push(packet_class_name(&mut env_local, &pkt)?);
    }

    Ok(names)
}

fn read_varint_prefix(buf: &[u8]) -> (u32, usize) {
    let mut num: u32 = 0;
    let mut shift = 0;
    let mut i = 0;
    while i < buf.len() && i < 5 {
        let b = buf[i] as u32;
        num |= (b & 0x7F) << shift;
        i += 1;
        if (b & 0x80) == 0 {
            break;
        }
        shift += 7;
    }
    (num, i)
}
