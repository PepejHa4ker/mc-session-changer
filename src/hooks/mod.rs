use crate::hooks::jhook::JNIHookManager;
use crate::mappings::{classes, signatures};
use anyhow::Result;
use jni::sys::JavaVM;
use crate::hooks::packet::mod_list_hook::ModListHook;
use crate::hooks::packet::packet_read_hook::InboundOutListTap;
use crate::hooks::packet::packet_write_hook::PacketWriteHook;
use crate::hooks::packet::hwid_hook::HwidHook;

pub mod jhook;
pub mod opengl;
mod packet;

macro_rules! hook_entry {
    ($manager:expr, $name:expr, $class:expr, $method:expr, $sig:expr, $factory:expr) => {{
        tracing::info!("Hooking {} :: {}.{}{}", $name, $class, $method, $sig);
        match $manager.hook_method($class, $method, $sig, $factory()) {
            Ok(_) => tracing::info!("Successfully hooked {}", $name),
            Err(e) => tracing::warn!("Failed to hook {}: {:#}", $name, e),
        }
    }};
}

pub unsafe fn setup_jni_hooks(jvm: *mut JavaVM) -> Result<()> {
    tracing::info!("Setting up packet hooks...");

    let hook_manager = JNIHookManager::obtain(jvm)?;
    let manager = &*hook_manager;
    tracing::info!("Jvm obtained");

    hook_entry!(
        manager,
        "PacketWriter",
        classes::NETTY_PACKET_ENCODER,
        "encode",
        signatures::PACKET_ENCODER_ENCODE,
        || PacketWriteHook
    );

    hook_entry!(
        manager,
        "PacketReader",
        classes::NETTY_PACKET_DECODER,
        "decode",
        signatures::PACKET_DECODER_DECODE,
        || InboundOutListTap
    );

    // #[cfg(feature = "mc_1_7_10")]
    // hook_entry!(
    //     manager,
    //     "FMLHandshakeMessage$ModList::toBytes",
    //     classes::FML_MOD_LIST_MESSAGE,
    //     "toBytes",
    //     "(Lio/netty/buffer/ByteBuf;)V",
    //     || ModListHook
    // );

    hook_entry!(
        manager,
        "ModList::getActiveModList",
        "cpw/mods/fml/common/Loader",
        "getActiveModList",
        "()Ljava/util/List;",
        || ModListHook
    );
    hook_entry!(
        manager,
        "pG::do (writeHwid)",
        classes::PG_HWID,
        "do",
        "(Ljava/io/DataOutput;)V",
        || HwidHook
    );

    Ok(())
}
