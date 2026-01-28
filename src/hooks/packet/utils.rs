use crate::core::state::GlobalState;
use crate::graphics::netlog::{make_record, PacketDirection};
use crate::mappings::classes;
use anyhow::{Context, Result};
use jni::objects::{JByteArray, JObject, JString, JValue};
use jni::JNIEnv;

pub unsafe fn packet_class_name(
    env: &mut JNIEnv,
    packet: &JObject,
) -> Result<String> {
    let cls_obj = env
        .call_method(packet, "getClass", "()Ljava/lang/Class;", &[])
        .context("packet.getClass")?
        .l()
        .context("class obj null")?;
    let name_obj = env
        .call_method(&cls_obj, "getName", "()Ljava/lang/String;", &[])
        .context("class.getName")?
        .l()
        .context("class name null")?;
    let name_str: String = env
        .get_string(&JString::from(name_obj))
        .context("class name to string")?
        .into();
    Ok(name_str)
}

pub unsafe fn new_packet_buffer<'env, 'local>(
    env: &mut JNIEnv<'env>,
    capacity: i32,
) -> Result<JObject<'env>> {
    let unpooled = env.find_class("io/netty/buffer/Unpooled")?;
    let bytebuf = env
        .call_static_method(
            &unpooled,
            "buffer",
            "(I)Lio/netty/buffer/ByteBuf;",
            &[JValue::Int(capacity)],
        )?
        .l()
        .context("Unpooled.buffer returned null")?;

    let pb_cls = env.find_class(classes::PACKET_BUFFER)?;
    let packet_buffer = env
        .new_object(
            &pb_cls,
            "(Lio/netty/buffer/ByteBuf;)V",
            &[JValue::Object(&bytebuf)],
        )
        .context("Failed to create PacketBuffer")?;
    Ok(packet_buffer)
}

pub unsafe fn read_all_bytes(
    env: &mut JNIEnv,
    bytebuf: &JObject,
) -> Result<Vec<u8>> {
    let readable = env
        .call_method(bytebuf, "readableBytes", "()I", &[])
        .context("readableBytes")?
        .i()? as i32;
    if readable <= 0 {
        return Ok(Vec::new());
    }
    let dst: JByteArray = env.new_byte_array(readable)?.into();
    let reader_idx = env
        .call_method(bytebuf, "readerIndex", "()I", &[])
        .context("readerIndex")?
        .i()?;
    env.call_method(
        bytebuf,
        "getBytes",
        "(I[B)Lio/netty/buffer/ByteBuf;",
        &[JValue::Int(reader_idx), JValue::Object(&dst)],
    )?;
    let bytes = env.convert_byte_array(dst)?;
    Ok(bytes)
}

pub fn push_packet_log(direction: PacketDirection, name: String, bytes: Vec<u8>) {
    if let Some(store) = GlobalState::instance().get_packet_store().get() {
        if !GlobalState::instance().is_packet_paused() {
            store.lock().push(make_record(direction, name, bytes));
        }
    }
}
