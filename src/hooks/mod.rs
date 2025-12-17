use crate::hooks::jhook::JNIHookManager;
use anyhow::Result;
use jni::sys::JavaVM;
use crate::hooks::packet::chat_hook::ChatHook;
use crate::hooks::packet::hwid_hook::HwidHook;
use crate::hooks::packet::mod_list_hook::ModListHook;
use crate::hooks::packet::packet_read_hook::InboundOutListTap;
use crate::hooks::packet::packet_write_hook::PacketWriteHook;

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
        "net/minecraft/util/MessageSerializer",
        "encode",
        "(Lio/netty/channel/ChannelHandlerContext;Lnet/minecraft/network/Packet;Lio/netty/buffer/ByteBuf;)V",
        || PacketWriteHook
    );

    hook_entry!(
        manager,
        "PacketReader",
        "net/minecraft/util/MessageDeserializer",
        "decode",
        "(Lio/netty/channel/ChannelHandlerContext;Lio/netty/buffer/ByteBuf;Ljava/util/List;)V",
        || InboundOutListTap
    );

    // hook_entry!(
    //     manager,
    //     "HWID::writeData",
    //     "ru/sky_drive/dw/pG",
    //     "do",
    //     "(Ljava/io/DataOutput;)V",
    //     || HwidHook
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
        "Chat::printChatMessage",
        "net/minecraft/client/gui/GuiNewChat",
        "func_146227_a",
        "(Lnet/minecraft/util/IChatComponent;)V",
        || ChatHook
    );
    Ok(())
}
