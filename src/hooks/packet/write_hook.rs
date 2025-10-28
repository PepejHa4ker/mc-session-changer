use jni::JNIEnv;
use jni::objects::{JByteArray, JObject, JString, JValue};
use jni::sys::{jclass, jmethodID, jobject, jvalue};
use anyhow::Result;
use crate::hooks::jhook::HookCallback;

pub struct PacketWriteHook;

impl HookCallback for PacketWriteHook {
    unsafe fn call(
        &self,
        env: *mut jni::sys::JNIEnv,
        this: jobject,
        class: jclass,
        method: jmethodID,
        args: &[jvalue],
    ) -> Option<jvalue> {
        if let Ok(mut safe_env) = JNIEnv::from_raw(env) {
            if args.len() >= 1 {
                let packet_obj = JObject::from_raw(args[0].l);
                if let Err(e) = self.log_outgoing_packet(&mut safe_env, &packet_obj) {
                    tracing::error!("Error logging outgoing packet: {}", e);
                }
            }
        }

        (**env).CallNonvirtualVoidMethodA.unwrap()(env, this, class, method, args.as_ptr());

        let exception_occurred = (**env).ExceptionCheck.unwrap()(env);
        if exception_occurred == jni::sys::JNI_TRUE {
            tracing::warn!("Exception occurred in dispatchPacket");
            (**env).ExceptionDescribe.unwrap()(env);
        }

        None
    }
}

impl PacketWriteHook {
    unsafe fn log_outgoing_packet(&self, env: &mut JNIEnv, packet: &JObject) -> Result<()> {
        if packet.is_null() {
            return Ok(());
        }

        let packet_class = env.get_object_class(packet)?;
        let class_obj = JObject::from_raw(packet_class.into_raw());
        let class_name = env.call_method(&class_obj, "getName", "()Ljava/lang/String;", &[])?
            .l()?;

        let class_name_jstring = JString::from(class_name);
        let class_name_string = env.get_string(&class_name_jstring)?;

        match self.get_packet_data(env, packet) {
            Ok(packet_data) => {
                tracing::info!(
                    "[OUTGOING PACKET] Class: {}, Data length: {} bytes, Data: {:02X?}",
                    class_name_string.to_str().unwrap_or("Unknown"),
                    packet_data.len(),
                    &packet_data[..std::cmp::min(packet_data.len(), 32)] 
                );
            }
            Err(_) => {
                tracing::info!(
                    "[OUTGOING PACKET] Class: {}, Data: Could not extract packet data",
                    class_name_string.to_str().unwrap_or("Unknown")
                );
            }
        }

        Ok(())
    }

    fn get_packet_data(&self, env: &mut JNIEnv, packet: &JObject) -> Result<Vec<u8>> {
        let baos_class = env.find_class("java/io/ByteArrayOutputStream")?;
        let baos = env.new_object(&baos_class, "()V", &[])?;
        let dos_class = env.find_class("java/io/DataOutputStream")?;
        let dos = env.new_object(&dos_class, "(Ljava/io/OutputStream;)V", &[JValue::Object(&baos)])?;
        let packet_buffer_class = env.find_class("net/minecraft/network/PacketBuffer")?;
        let packet_buffer = env.new_object(&packet_buffer_class, "(Ljava/io/DataOutputStream;)V", &[JValue::Object(&dos)])?;
        env.call_method(packet, "func_148840_b", "(Lnet/minecraft/network/PacketBuffer;)V", &[JValue::Object(&packet_buffer)])?;
        let byte_array = env.call_method(&baos, "toByteArray", "()[B", &[])?.l()?;
        let byte_array = JByteArray::from(byte_array);

        let bytes = env.convert_byte_array(byte_array)?;
        Ok(bytes)
    }
}