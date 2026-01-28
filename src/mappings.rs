#[cfg(all(feature = "mc_1_7_10", feature = "mc_1_12_2"))]
compile_error!("Features 'mc_1_7_10' and 'mc_1_12_2' are mutually exclusive");

#[cfg(not(any(feature = "mc_1_7_10", feature = "mc_1_12_2")))]
compile_error!("One of 'mc_1_7_10' or 'mc_1_12_2' feature must be enabled. Use --features mc_1_12_2 or --features mc_1_7_10");

pub mod classes {
    #[cfg(feature = "mc_1_7_10")]
    pub const NETTY_PACKET_ENCODER: &str = "net/minecraft/util/MessageSerializer";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const NETTY_PACKET_ENCODER: &str = "net/minecraft/network/NettyPacketEncoder";

    #[cfg(feature = "mc_1_7_10")]
    pub const NETTY_PACKET_DECODER: &str = "net/minecraft/util/MessageDeserializer";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const NETTY_PACKET_DECODER: &str = "net/minecraft/network/NettyPacketDecoder";

    #[cfg(feature = "mc_1_7_10")]
    pub const FML_LOADER: &str = "cpw/mods/fml/common/Loader";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const FML_LOADER: &str = "net/minecraftforge/fml/common/Loader";

    #[cfg(feature = "mc_1_7_10")]
    pub const GUI_NEW_CHAT: &str = "net/minecraft/client/gui/GuiNewChat";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const GUI_NEW_CHAT: &str = "net/minecraft/client/gui/GuiNewChat";

    #[cfg(feature = "mc_1_7_10")]
    pub const PACKET: &str = "net/minecraft/network/Packet";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const PACKET: &str = "net/minecraft/network/Packet";

    #[cfg(feature = "mc_1_7_10")]
    pub const PACKET_BUFFER: &str = "net/minecraft/network/PacketBuffer";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const PACKET_BUFFER: &str = "net/minecraft/network/PacketBuffer";

    #[cfg(feature = "mc_1_7_10")]
    pub const CUSTOM_PAYLOAD_CLIENT: &str = "net.minecraft.network.play.client.C17PacketCustomPayload";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const CUSTOM_PAYLOAD_CLIENT: &str = "net.minecraft.network.play.client.CPacketCustomPayload";

    #[cfg(feature = "mc_1_7_10")]
    pub const PLAYER_PACKET: &str = "net.minecraft.network.play.client.C03PacketPlayer";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const PLAYER_PACKET: &str = "net.minecraft.network.play.client.CPacketPlayer";

    #[cfg(feature = "mc_1_7_10")]
    pub const GAME_DATA: &str = "cpw.mods.fml.common.registry.GameData";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const GAME_DATA: &str = "net.minecraftforge.registries.GameData";

    #[cfg(feature = "mc_1_7_10")]
    pub const FML_MOD_LIST_MESSAGE: &str = "cpw.mods.fml.common.network.handshake.FMLHandshakeMessage$ModList";

    pub const PG_HWID: &str = "ru.sky_drive.dw.pG";
}

pub mod methods {
    #[cfg(feature = "mc_1_7_10")]
    pub const CHAT_PRINT_MESSAGE: &str = "func_146227_a";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const CHAT_PRINT_MESSAGE: &str = "func_146227_a";

    #[cfg(feature = "mc_1_7_10")]
    pub const CHAT_GET_TEXT: &str = "func_150260_c";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const CHAT_GET_TEXT: &str = "func_150260_c";

    #[cfg(feature = "mc_1_7_10")]
    pub const PACKET_WRITE_DATA: &str = "func_148840_b";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const PACKET_WRITE_DATA: &str = "func_148840_b";

    #[cfg(feature = "mc_1_7_10")]
    pub const CUSTOM_PAYLOAD_GET_CHANNEL: &str = "func_149559_c";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const CUSTOM_PAYLOAD_GET_CHANNEL: &str = "func_149559_c";
}

pub mod signatures {
    #[cfg(feature = "mc_1_7_10")]
    pub const PACKET_ENCODER_ENCODE: &str = "(Lio/netty/channel/ChannelHandlerContext;Ljava/lang/Object;Lio/netty/buffer/ByteBuf;)V";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const PACKET_ENCODER_ENCODE: &str = "(Lio/netty/channel/ChannelHandlerContext;Lnet/minecraft/network/Packet;Lio/netty/buffer/ByteBuf;)V";

    #[cfg(feature = "mc_1_7_10")]
    pub const PACKET_DECODER_DECODE: &str = "(Lio/netty/channel/ChannelHandlerContext;Lio/netty/buffer/ByteBuf;Ljava/util/List;)V";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const PACKET_DECODER_DECODE: &str = "(Lio/netty/channel/ChannelHandlerContext;Lio/netty/buffer/ByteBuf;Ljava/util/List;)V";

    #[cfg(feature = "mc_1_7_10")]
    pub const CHAT_PRINT_MESSAGE: &str = "(Lnet/minecraft/util/IChatComponent;)V";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const CHAT_PRINT_MESSAGE: &str = "(Lnet/minecraft/util/text/ITextComponent;)V";

    #[cfg(feature = "mc_1_7_10")]
    pub const PACKET_WRITE_DATA: &str = "(Lnet/minecraft/network/PacketBuffer;)V";
    #[cfg(all(feature = "mc_1_12_2", not(feature = "mc_1_7_10")))]
    pub const PACKET_WRITE_DATA: &str = "(Lnet/minecraft/network/PacketBuffer;)V";

    #[cfg(feature = "mc_1_7_10")]
    pub const INJECT_WORLD_ID_MAP: &str = "(Ljava/util/Map;Ljava/util/Set;Ljava/util/Map;Ljava/util/Map;Ljava/util/Set;Ljava/util/Set;ZZ)Ljava/util/List;";

    pub const PG_GET_HOSTNAME: &str = "()Ljava/lang/String;";
    pub const PG_GET_PROCESSOR_INFO: &str = "(Loshi/hardware/HardwareAbstractionLayer;)Ljava/lang/String;";
    pub const PG_GET_HARDWARE_UUID: &str = "(Loshi/hardware/HardwareAbstractionLayer;)Ljava/lang/String;";
    pub const PG_GET_GRAPHICS_INFO: &str = "(Loshi/hardware/HardwareAbstractionLayer;)Ljava/lang/String;";
    pub const PG_GET_DISK_INFO: &str = "(Loshi/hardware/HardwareAbstractionLayer;)Ljava/lang/String;";
    pub const PG_GET_MAC_ADDRESS: &str = "()Ljava/lang/String;";
}
