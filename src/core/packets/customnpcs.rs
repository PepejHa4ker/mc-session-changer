use std::io;
use crate::core::packets::Bound;
use crate::core::packets::reader::ModPacketReader;
use crate::core::custom_payload::{CustomPayloadDecoder, DecodedStruct, DecodedField, DecodedValue};

pub const CHANNEL: &str = "CustomNPCs";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum EnumPacketClient {
    CHAT = 0,
    MESSAGE = 1,
    SYNCRECIPES_ADD = 2,
    SYNCRECIPES_WORKBENCH = 3,
    DIALOG = 4,
    QUEST_COMPLETION = 5,
    EDIT_NPC = 6,
    PLAY_SOUND = 7,
    PLAY_MUSIC = 8,
    UPDATE_NPC = 9,
    ROLE = 10,
    GUI = 11,
    SCRIPTED_PARTICLE = 12,
    PARTICLE = 13,
    DELETE_NPC = 14,
    SCROLL_LIST = 15,
    SCROLL_DATA = 16,
    SCROLL_DATA_PART = 17,
    SCROLL_SELECTED = 18,
    GUI_REDSTONE = 19,
    GUI_WAYPOINT = 20,
    GUI_DATA = 21,
    GUI_ERROR = 22,
    GUI_CLOSE = 23,
    VILLAGER_LIST = 24,
    CHATBUBBLE = 25,
    SYNCRECIPES_CARPENTRYBENCH = 26,
    CLONE = 27,
    OPEN_BOOK = 28,
    DIALOG_DUMMY = 29,
    CONFIG = 30,
    ISGUIOPEN = 31,
    SCRIPT_OVERLAY_DATA = 32,
    SCRIPT_OVERLAY_CLOSE = 33,
    SWING_PLAYER_ARM = 34,
    UPDATE_ITEM = 35,
    PLAYER_UPDATE_SKIN_OVERLAYS = 36,
    UPDATE_ANIMATIONS = 37,
    OVERLAY_QUEST_TRACKING = 38,
    DISABLE_MOUSE_INPUT = 39,
    PLAY_SOUND_TO = 40,
    PLAY_SOUND_TO_NO_ID = 41,
    STOP_SOUND_FOR = 42,
    PAUSE_SOUNDS = 43,
    CONTINUE_SOUNDS = 44,
    STOP_SOUNDS = 45,
}

impl EnumPacketClient {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::CHAT),
            1 => Some(Self::MESSAGE),
            2 => Some(Self::SYNCRECIPES_ADD),
            3 => Some(Self::SYNCRECIPES_WORKBENCH),
            4 => Some(Self::DIALOG),
            5 => Some(Self::QUEST_COMPLETION),
            6 => Some(Self::EDIT_NPC),
            7 => Some(Self::PLAY_SOUND),
            8 => Some(Self::PLAY_MUSIC),
            9 => Some(Self::UPDATE_NPC),
            10 => Some(Self::ROLE),
            11 => Some(Self::GUI),
            12 => Some(Self::SCRIPTED_PARTICLE),
            13 => Some(Self::PARTICLE),
            14 => Some(Self::DELETE_NPC),
            15 => Some(Self::SCROLL_LIST),
            16 => Some(Self::SCROLL_DATA),
            17 => Some(Self::SCROLL_DATA_PART),
            18 => Some(Self::SCROLL_SELECTED),
            19 => Some(Self::GUI_REDSTONE),
            20 => Some(Self::GUI_WAYPOINT),
            21 => Some(Self::GUI_DATA),
            22 => Some(Self::GUI_ERROR),
            23 => Some(Self::GUI_CLOSE),
            24 => Some(Self::VILLAGER_LIST),
            25 => Some(Self::CHATBUBBLE),
            26 => Some(Self::SYNCRECIPES_CARPENTRYBENCH),
            27 => Some(Self::CLONE),
            28 => Some(Self::OPEN_BOOK),
            29 => Some(Self::DIALOG_DUMMY),
            30 => Some(Self::CONFIG),
            31 => Some(Self::ISGUIOPEN),
            32 => Some(Self::SCRIPT_OVERLAY_DATA),
            33 => Some(Self::SCRIPT_OVERLAY_CLOSE),
            34 => Some(Self::SWING_PLAYER_ARM),
            35 => Some(Self::UPDATE_ITEM),
            36 => Some(Self::PLAYER_UPDATE_SKIN_OVERLAYS),
            37 => Some(Self::UPDATE_ANIMATIONS),
            38 => Some(Self::OVERLAY_QUEST_TRACKING),
            39 => Some(Self::DISABLE_MOUSE_INPUT),
            40 => Some(Self::PLAY_SOUND_TO),
            41 => Some(Self::PLAY_SOUND_TO_NO_ID),
            42 => Some(Self::STOP_SOUND_FOR),
            43 => Some(Self::PAUSE_SOUNDS),
            44 => Some(Self::CONTINUE_SOUNDS),
            45 => Some(Self::STOP_SOUNDS),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::CHAT => "CHAT",
            Self::MESSAGE => "MESSAGE",
            Self::SYNCRECIPES_ADD => "SYNCRECIPES_ADD",
            Self::SYNCRECIPES_WORKBENCH => "SYNCRECIPES_WORKBENCH",
            Self::DIALOG => "DIALOG",
            Self::QUEST_COMPLETION => "QUEST_COMPLETION",
            Self::EDIT_NPC => "EDIT_NPC",
            Self::PLAY_SOUND => "PLAY_SOUND",
            Self::PLAY_MUSIC => "PLAY_MUSIC",
            Self::UPDATE_NPC => "UPDATE_NPC",
            Self::ROLE => "ROLE",
            Self::GUI => "GUI",
            Self::SCRIPTED_PARTICLE => "SCRIPTED_PARTICLE",
            Self::PARTICLE => "PARTICLE",
            Self::DELETE_NPC => "DELETE_NPC",
            Self::SCROLL_LIST => "SCROLL_LIST",
            Self::SCROLL_DATA => "SCROLL_DATA",
            Self::SCROLL_DATA_PART => "SCROLL_DATA_PART",
            Self::SCROLL_SELECTED => "SCROLL_SELECTED",
            Self::GUI_REDSTONE => "GUI_REDSTONE",
            Self::GUI_WAYPOINT => "GUI_WAYPOINT",
            Self::GUI_DATA => "GUI_DATA",
            Self::GUI_ERROR => "GUI_ERROR",
            Self::GUI_CLOSE => "GUI_CLOSE",
            Self::VILLAGER_LIST => "VILLAGER_LIST",
            Self::CHATBUBBLE => "CHATBUBBLE",
            Self::SYNCRECIPES_CARPENTRYBENCH => "SYNCRECIPES_CARPENTRYBENCH",
            Self::CLONE => "CLONE",
            Self::OPEN_BOOK => "OPEN_BOOK",
            Self::DIALOG_DUMMY => "DIALOG_DUMMY",
            Self::CONFIG => "CONFIG",
            Self::ISGUIOPEN => "ISGUIOPEN",
            Self::SCRIPT_OVERLAY_DATA => "SCRIPT_OVERLAY_DATA",
            Self::SCRIPT_OVERLAY_CLOSE => "SCRIPT_OVERLAY_CLOSE",
            Self::SWING_PLAYER_ARM => "SWING_PLAYER_ARM",
            Self::UPDATE_ITEM => "UPDATE_ITEM",
            Self::PLAYER_UPDATE_SKIN_OVERLAYS => "PLAYER_UPDATE_SKIN_OVERLAYS",
            Self::UPDATE_ANIMATIONS => "UPDATE_ANIMATIONS",
            Self::OVERLAY_QUEST_TRACKING => "OVERLAY_QUEST_TRACKING",
            Self::DISABLE_MOUSE_INPUT => "DISABLE_MOUSE_INPUT",
            Self::PLAY_SOUND_TO => "PLAY_SOUND_TO",
            Self::PLAY_SOUND_TO_NO_ID => "PLAY_SOUND_TO_NO_ID",
            Self::STOP_SOUND_FOR => "STOP_SOUND_FOR",
            Self::PAUSE_SOUNDS => "PAUSE_SOUNDS",
            Self::CONTINUE_SOUNDS => "CONTINUE_SOUNDS",
            Self::STOP_SOUNDS => "STOP_SOUNDS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum EnumPacketServer {
    Delete = 0,
    RemoteMainMenu = 1,
    NpcMenuClose = 2,
    RemoteDelete = 3,
    RemoteFreeze = 4,
    RemoteReset = 5,
    SpawnMob = 6,
    MobSpawner = 7,
    MainmenuAISave = 8,
    MainmenuAIGet = 9,
    MainmenuInvSave = 10,
    MainmenuInvGet = 11,
    MainmenuStatsSave = 12,
    MainmenuStatsGet = 13,
    MainmenuDisplaySave = 14,
    MainmenuDisplayGet = 15,
    ModelDataSave = 16,
    MainmenuAdvancedSave = 17,
    MainmenuAdvancedGet = 18,
    DialogNpcSet = 19,
    DialogNpcRemove = 20,
    FactionSet = 21,
    TagSet = 22,
    TransportSave = 23,
    TransformSave = 24,
    TransformGet = 25,
    TransformLoad = 26,
    TraderMarketSave = 27,
    JobSave = 28,
    JobGet = 29,
    RoleSave = 30,
    RoleGet = 31,
    JobSpawnerAdd = 32,
    JobSpawnerRemove = 33,
    RoleCompanionUpdate = 34,
    LinkedSet = 35,
    ClonePreSave = 36,
    CloneSave = 37,
    CloneRemove = 38,
    CloneList = 39,
    CloneTagList = 40,
    CloneAllTags = 41,
    CloneAllTagsShort = 42,
    ScriptGlobalGuiDataSave = 43,
    ScriptGlobalGuiDataGet = 44,
    ScriptPlayerSave = 45,
    ScriptPlayerGet = 46,
    ScriptForgeSave = 47,
    ScriptForgeGet = 48,
    ScriptGlobalNPCSave = 49,
    ScriptGlobalNPCGet = 50,
    ScriptItemDataSave = 51,
    ScriptItemDataGet = 52,
    LinkedGetAll = 53,
    LinkedRemove = 54,
    LinkedAdd = 55,
    ScriptDataSave = 56,
    ScriptDataGet = 57,
    EventScriptDataSave = 58,
    EventScriptDataGet = 59,
    PlayerDataRemove = 60,
    PlayerDataRegen = 61,
    BankSave = 62,
    BanksGet = 63,
    BankGet = 64,
    BankRemove = 65,
    DialogCategorySave = 66,
    DialogCategoriesGet = 67,
    DialogsGetFromDialog = 68,
    DialogCategoryRemove = 69,
    DialogCategoryGet = 70,
    DialogSave = 71,
    DialogsGet = 72,
    DialogGet = 73,
    DialogRemove = 74,
    TransportCategoryRemove = 75,
    TransportGetLocation = 76,
    TransportRemove = 77,
    TransportsGet = 78,
    TransportCategorySave = 79,
    TransportCategoriesGet = 80,
    FactionRemove = 81,
    FactionSave = 82,
    FactionsGet = 83,
    FactionGet = 84,
    TagRemove = 85,
    TagSave = 86,
    TagsGet = 87,
    TagGet = 88,
    NpcTagsGet = 89,
    QuestCategorySave = 90,
    QuestCategoriesGet = 91,
    QuestRemove = 92,
    QuestCategoryRemove = 93,
    QuestRewardSave = 94,
    QuestSave = 95,
    QuestsGetFromQuest = 96,
    QuestsGet = 97,
    QuestDialogGetTitle = 98,
    RecipeSave = 99,
    RecipeRemove = 100,
    NaturalSpawnSave = 101,
    NaturalSpawnGet = 102,
    NaturalSpawnRemove = 103,
    MerchantUpdate = 104,
    PlayerRider = 105,
    SpawnRider = 106,
    MovingPathSave = 107,
    MovingPathGet = 108,
    DialogNpcGet = 109,
    AnimationListGet = 110,
    AnimationGet = 111,
    AnimationAdd = 112,
    AnimationDelete = 113,
    AnimationSave = 114,
    RecipesGet = 115,
    RecipeGet = 116,
    QuestOpenGui = 117,
    PlayerDataGet = 118,
    RemoteNpcsGet = 119,
    RemoteTpToNpc = 120,
    QuestGet = 121,
    QuestCategoryGet = 122,
    SaveTileEntity = 123,
    NaturalSpawnGetAll = 124,
    MailOpenSetup = 125,
    DimensionsGet = 126,
    DimensionTeleport = 127,
    GetTileEntity = 128,
    Gui = 129,
    IsGuiOpen = 130,
    CustomGuiButton = 131,
    CustomGuiScrollClick = 132,
    CustomGuiClose = 133,
    CustomGuiUnfocused = 134,
    ServerUpdateSkinOverlays = 135,
}

impl EnumPacketServer {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Delete),
            1 => Some(Self::RemoteMainMenu),
            2 => Some(Self::NpcMenuClose),
            3 => Some(Self::RemoteDelete),
            4 => Some(Self::RemoteFreeze),
            5 => Some(Self::RemoteReset),
            6 => Some(Self::SpawnMob),
            7 => Some(Self::MobSpawner),
            8 => Some(Self::MainmenuAISave),
            9 => Some(Self::MainmenuAIGet),
            10 => Some(Self::MainmenuInvSave),
            11 => Some(Self::MainmenuInvGet),
            12 => Some(Self::MainmenuStatsSave),
            13 => Some(Self::MainmenuStatsGet),
            14 => Some(Self::MainmenuDisplaySave),
            15 => Some(Self::MainmenuDisplayGet),
            16 => Some(Self::ModelDataSave),
            17 => Some(Self::MainmenuAdvancedSave),
            18 => Some(Self::MainmenuAdvancedGet),
            19 => Some(Self::DialogNpcSet),
            20 => Some(Self::DialogNpcRemove),
            21 => Some(Self::FactionSet),
            22 => Some(Self::TagSet),
            23 => Some(Self::TransportSave),
            24 => Some(Self::TransformSave),
            25 => Some(Self::TransformGet),
            26 => Some(Self::TransformLoad),
            27 => Some(Self::TraderMarketSave),
            28 => Some(Self::JobSave),
            29 => Some(Self::JobGet),
            30 => Some(Self::RoleSave),
            31 => Some(Self::RoleGet),
            32 => Some(Self::JobSpawnerAdd),
            33 => Some(Self::JobSpawnerRemove),
            34 => Some(Self::RoleCompanionUpdate),
            35 => Some(Self::LinkedSet),
            36 => Some(Self::ClonePreSave),
            37 => Some(Self::CloneSave),
            38 => Some(Self::CloneRemove),
            39 => Some(Self::CloneList),
            40 => Some(Self::CloneTagList),
            41 => Some(Self::CloneAllTags),
            42 => Some(Self::CloneAllTagsShort),
            43 => Some(Self::ScriptGlobalGuiDataSave),
            44 => Some(Self::ScriptGlobalGuiDataGet),
            45 => Some(Self::ScriptPlayerSave),
            46 => Some(Self::ScriptPlayerGet),
            47 => Some(Self::ScriptForgeSave),
            48 => Some(Self::ScriptForgeGet),
            49 => Some(Self::ScriptGlobalNPCSave),
            50 => Some(Self::ScriptGlobalNPCGet),
            51 => Some(Self::ScriptItemDataSave),
            52 => Some(Self::ScriptItemDataGet),
            53 => Some(Self::LinkedGetAll),
            54 => Some(Self::LinkedRemove),
            55 => Some(Self::LinkedAdd),
            56 => Some(Self::ScriptDataSave),
            57 => Some(Self::ScriptDataGet),
            58 => Some(Self::EventScriptDataSave),
            59 => Some(Self::EventScriptDataGet),
            60 => Some(Self::PlayerDataRemove),
            61 => Some(Self::PlayerDataRegen),
            62 => Some(Self::BankSave),
            63 => Some(Self::BanksGet),
            64 => Some(Self::BankGet),
            65 => Some(Self::BankRemove),
            66 => Some(Self::DialogCategorySave),
            67 => Some(Self::DialogCategoriesGet),
            68 => Some(Self::DialogsGetFromDialog),
            69 => Some(Self::DialogCategoryRemove),
            70 => Some(Self::DialogCategoryGet),
            71 => Some(Self::DialogSave),
            72 => Some(Self::DialogsGet),
            73 => Some(Self::DialogGet),
            74 => Some(Self::DialogRemove),
            75 => Some(Self::TransportCategoryRemove),
            76 => Some(Self::TransportGetLocation),
            77 => Some(Self::TransportRemove),
            78 => Some(Self::TransportsGet),
            79 => Some(Self::TransportCategorySave),
            80 => Some(Self::TransportCategoriesGet),
            81 => Some(Self::FactionRemove),
            82 => Some(Self::FactionSave),
            83 => Some(Self::FactionsGet),
            84 => Some(Self::FactionGet),
            85 => Some(Self::TagRemove),
            86 => Some(Self::TagSave),
            87 => Some(Self::TagsGet),
            88 => Some(Self::TagGet),
            89 => Some(Self::NpcTagsGet),
            90 => Some(Self::QuestCategorySave),
            91 => Some(Self::QuestCategoriesGet),
            92 => Some(Self::QuestRemove),
            93 => Some(Self::QuestCategoryRemove),
            94 => Some(Self::QuestRewardSave),
            95 => Some(Self::QuestSave),
            96 => Some(Self::QuestsGetFromQuest),
            97 => Some(Self::QuestsGet),
            98 => Some(Self::QuestDialogGetTitle),
            99 => Some(Self::RecipeSave),
            100 => Some(Self::RecipeRemove),
            101 => Some(Self::NaturalSpawnSave),
            102 => Some(Self::NaturalSpawnGet),
            103 => Some(Self::NaturalSpawnRemove),
            104 => Some(Self::MerchantUpdate),
            105 => Some(Self::PlayerRider),
            106 => Some(Self::SpawnRider),
            107 => Some(Self::MovingPathSave),
            108 => Some(Self::MovingPathGet),
            109 => Some(Self::DialogNpcGet),
            110 => Some(Self::AnimationListGet),
            111 => Some(Self::AnimationGet),
            112 => Some(Self::AnimationAdd),
            113 => Some(Self::AnimationDelete),
            114 => Some(Self::AnimationSave),
            115 => Some(Self::RecipesGet),
            116 => Some(Self::RecipeGet),
            117 => Some(Self::QuestOpenGui),
            118 => Some(Self::PlayerDataGet),
            119 => Some(Self::RemoteNpcsGet),
            120 => Some(Self::RemoteTpToNpc),
            121 => Some(Self::QuestGet),
            122 => Some(Self::QuestCategoryGet),
            123 => Some(Self::SaveTileEntity),
            124 => Some(Self::NaturalSpawnGetAll),
            125 => Some(Self::MailOpenSetup),
            126 => Some(Self::DimensionsGet),
            127 => Some(Self::DimensionTeleport),
            128 => Some(Self::GetTileEntity),
            129 => Some(Self::Gui),
            130 => Some(Self::IsGuiOpen),
            131 => Some(Self::CustomGuiButton),
            132 => Some(Self::CustomGuiScrollClick),
            133 => Some(Self::CustomGuiClose),
            134 => Some(Self::CustomGuiUnfocused),
            135 => Some(Self::ServerUpdateSkinOverlays),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Delete => "Delete",
            Self::RemoteMainMenu => "RemoteMainMenu",
            Self::NpcMenuClose => "NpcMenuClose",
            Self::RemoteDelete => "RemoteDelete",
            Self::RemoteFreeze => "RemoteFreeze",
            Self::RemoteReset => "RemoteReset",
            Self::SpawnMob => "SpawnMob",
            Self::MobSpawner => "MobSpawner",
            Self::MainmenuAISave => "MainmenuAISave",
            Self::MainmenuAIGet => "MainmenuAIGet",
            Self::MainmenuInvSave => "MainmenuInvSave",
            Self::MainmenuInvGet => "MainmenuInvGet",
            Self::MainmenuStatsSave => "MainmenuStatsSave",
            Self::MainmenuStatsGet => "MainmenuStatsGet",
            Self::MainmenuDisplaySave => "MainmenuDisplaySave",
            Self::MainmenuDisplayGet => "MainmenuDisplayGet",
            Self::ModelDataSave => "ModelDataSave",
            Self::MainmenuAdvancedSave => "MainmenuAdvancedSave",
            Self::MainmenuAdvancedGet => "MainmenuAdvancedGet",
            Self::DialogNpcSet => "DialogNpcSet",
            Self::DialogNpcRemove => "DialogNpcRemove",
            Self::FactionSet => "FactionSet",
            Self::TagSet => "TagSet",
            Self::TransportSave => "TransportSave",
            Self::TransformSave => "TransformSave",
            Self::TransformGet => "TransformGet",
            Self::TransformLoad => "TransformLoad",
            Self::TraderMarketSave => "TraderMarketSave",
            Self::JobSave => "JobSave",
            Self::JobGet => "JobGet",
            Self::RoleSave => "RoleSave",
            Self::RoleGet => "RoleGet",
            Self::JobSpawnerAdd => "JobSpawnerAdd",
            Self::JobSpawnerRemove => "JobSpawnerRemove",
            Self::RoleCompanionUpdate => "RoleCompanionUpdate",
            Self::LinkedSet => "LinkedSet",
            Self::ClonePreSave => "ClonePreSave",
            Self::CloneSave => "CloneSave",
            Self::CloneRemove => "CloneRemove",
            Self::CloneList => "CloneList",
            Self::CloneTagList => "CloneTagList",
            Self::CloneAllTags => "CloneAllTags",
            Self::CloneAllTagsShort => "CloneAllTagsShort",
            Self::ScriptGlobalGuiDataSave => "ScriptGlobalGuiDataSave",
            Self::ScriptGlobalGuiDataGet => "ScriptGlobalGuiDataGet",
            Self::ScriptPlayerSave => "ScriptPlayerSave",
            Self::ScriptPlayerGet => "ScriptPlayerGet",
            Self::ScriptForgeSave => "ScriptForgeSave",
            Self::ScriptForgeGet => "ScriptForgeGet",
            Self::ScriptGlobalNPCSave => "ScriptGlobalNPCSave",
            Self::ScriptGlobalNPCGet => "ScriptGlobalNPCGet",
            Self::ScriptItemDataSave => "ScriptItemDataSave",
            Self::ScriptItemDataGet => "ScriptItemDataGet",
            Self::LinkedGetAll => "LinkedGetAll",
            Self::LinkedRemove => "LinkedRemove",
            Self::LinkedAdd => "LinkedAdd",
            Self::ScriptDataSave => "ScriptDataSave",
            Self::ScriptDataGet => "ScriptDataGet",
            Self::EventScriptDataSave => "EventScriptDataSave",
            Self::EventScriptDataGet => "EventScriptDataGet",
            Self::PlayerDataRemove => "PlayerDataRemove",
            Self::PlayerDataRegen => "PlayerDataRegen",
            Self::BankSave => "BankSave",
            Self::BanksGet => "BanksGet",
            Self::BankGet => "BankGet",
            Self::BankRemove => "BankRemove",
            Self::DialogCategorySave => "DialogCategorySave",
            Self::DialogCategoriesGet => "DialogCategoriesGet",
            Self::DialogsGetFromDialog => "DialogsGetFromDialog",
            Self::DialogCategoryRemove => "DialogCategoryRemove",
            Self::DialogCategoryGet => "DialogCategoryGet",
            Self::DialogSave => "DialogSave",
            Self::DialogsGet => "DialogsGet",
            Self::DialogGet => "DialogGet",
            Self::DialogRemove => "DialogRemove",
            Self::TransportCategoryRemove => "TransportCategoryRemove",
            Self::TransportGetLocation => "TransportGetLocation",
            Self::TransportRemove => "TransportRemove",
            Self::TransportsGet => "TransportsGet",
            Self::TransportCategorySave => "TransportCategorySave",
            Self::TransportCategoriesGet => "TransportCategoriesGet",
            Self::FactionRemove => "FactionRemove",
            Self::FactionSave => "FactionSave",
            Self::FactionsGet => "FactionsGet",
            Self::FactionGet => "FactionGet",
            Self::TagRemove => "TagRemove",
            Self::TagSave => "TagSave",
            Self::TagsGet => "TagsGet",
            Self::TagGet => "TagGet",
            Self::NpcTagsGet => "NpcTagsGet",
            Self::QuestCategorySave => "QuestCategorySave",
            Self::QuestCategoriesGet => "QuestCategoriesGet",
            Self::QuestRemove => "QuestRemove",
            Self::QuestCategoryRemove => "QuestCategoryRemove",
            Self::QuestRewardSave => "QuestRewardSave",
            Self::QuestSave => "QuestSave",
            Self::QuestsGetFromQuest => "QuestsGetFromQuest",
            Self::QuestsGet => "QuestsGet",
            Self::QuestDialogGetTitle => "QuestDialogGetTitle",
            Self::RecipeSave => "RecipeSave",
            Self::RecipeRemove => "RecipeRemove",
            Self::NaturalSpawnSave => "NaturalSpawnSave",
            Self::NaturalSpawnGet => "NaturalSpawnGet",
            Self::NaturalSpawnRemove => "NaturalSpawnRemove",
            Self::MerchantUpdate => "MerchantUpdate",
            Self::PlayerRider => "PlayerRider",
            Self::SpawnRider => "SpawnRider",
            Self::MovingPathSave => "MovingPathSave",
            Self::MovingPathGet => "MovingPathGet",
            Self::DialogNpcGet => "DialogNpcGet",
            Self::AnimationListGet => "AnimationListGet",
            Self::AnimationGet => "AnimationGet",
            Self::AnimationAdd => "AnimationAdd",
            Self::AnimationDelete => "AnimationDelete",
            Self::AnimationSave => "AnimationSave",
            Self::RecipesGet => "RecipesGet",
            Self::RecipeGet => "RecipeGet",
            Self::QuestOpenGui => "QuestOpenGui",
            Self::PlayerDataGet => "PlayerDataGet",
            Self::RemoteNpcsGet => "RemoteNpcsGet",
            Self::RemoteTpToNpc => "RemoteTpToNpc",
            Self::QuestGet => "QuestGet",
            Self::QuestCategoryGet => "QuestCategoryGet",
            Self::SaveTileEntity => "SaveTileEntity",
            Self::NaturalSpawnGetAll => "NaturalSpawnGetAll",
            Self::MailOpenSetup => "MailOpenSetup",
            Self::DimensionsGet => "DimensionsGet",
            Self::DimensionTeleport => "DimensionTeleport",
            Self::GetTileEntity => "GetTileEntity",
            Self::Gui => "Gui",
            Self::IsGuiOpen => "IsGuiOpen",
            Self::CustomGuiButton => "CustomGuiButton",
            Self::CustomGuiScrollClick => "CustomGuiScrollClick",
            Self::CustomGuiClose => "CustomGuiClose",
            Self::CustomGuiUnfocused => "CustomGuiUnfocused",
            Self::ServerUpdateSkinOverlays => "ServerUpdateSkinOverlays",
        }
    }
}

fn read_java_string(reader: &mut ModPacketReader) -> io::Result<String> {
    let len = reader.read_u16_be()? as usize;
    if len == 0 {
        return Ok(String::new());
    }
    let mut chars = Vec::with_capacity(len);
    for _ in 0..len {
        chars.push(reader.read_u16_be()?);
    }
    String::from_utf16(&chars)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-16: {}", e)))
}

fn decode_client_packet(packet_id: i32, reader: &mut ModPacketReader) -> Option<DecodedStruct> {
    let packet_type = EnumPacketClient::from_id(packet_id)?;
    let name = packet_type.name().to_string();

    let fields = match packet_type {
        EnumPacketClient::CHATBUBBLE => {
            let entity_id = reader.read_i32_be().ok()?;
            let text = read_java_string(reader).ok()?;
            let show_in_chat = reader.read_bool().ok()?;
            vec![
                DecodedField { name: "entity_id".to_string(), value: DecodedValue::Text(entity_id.to_string()) },
                DecodedField { name: "text".to_string(), value: DecodedValue::Text(text) },
                DecodedField { name: "show_in_chat".to_string(), value: DecodedValue::Text(show_in_chat.to_string()) },
            ]
        }
        EnumPacketClient::DIALOG => {
            let entity_id = reader.read_i32_be().ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "entity_id".to_string(), value: DecodedValue::Text(entity_id.to_string()) },
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketClient::DIALOG_DUMMY => {
            let npc_name = read_java_string(reader).ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "npc_name".to_string(), value: DecodedValue::Text(npc_name) },
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketClient::EDIT_NPC | EnumPacketClient::DELETE_NPC => {
            let entity_id = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "entity_id".to_string(), value: DecodedValue::Text(entity_id.to_string()) },
            ]
        }
        EnumPacketClient::PLAY_MUSIC => {
            let sound = read_java_string(reader).ok()?;
            vec![
                DecodedField { name: "sound".to_string(), value: DecodedValue::Text(sound) },
            ]
        }
        EnumPacketClient::PLAY_SOUND => {
            let sound = read_java_string(reader).ok()?;
            let x = reader.read_f32_be().ok()?;
            let y = reader.read_f32_be().ok()?;
            let z = reader.read_f32_be().ok()?;
            vec![
                DecodedField { name: "sound".to_string(), value: DecodedValue::Text(sound) },
                DecodedField { name: "x".to_string(), value: DecodedValue::Text(x.to_string()) },
                DecodedField { name: "y".to_string(), value: DecodedValue::Text(y.to_string()) },
                DecodedField { name: "z".to_string(), value: DecodedValue::Text(z.to_string()) },
            ]
        }
        EnumPacketClient::GUI => {
            let gui_type = reader.read_i32_be().ok()?;
            let x = reader.read_i32_be().ok()?;
            let y = reader.read_i32_be().ok()?;
            let z = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "gui_type".to_string(), value: DecodedValue::Text(gui_type.to_string()) },
                DecodedField { name: "x".to_string(), value: DecodedValue::Text(x.to_string()) },
                DecodedField { name: "y".to_string(), value: DecodedValue::Text(y.to_string()) },
                DecodedField { name: "z".to_string(), value: DecodedValue::Text(z.to_string()) },
            ]
        }
        EnumPacketClient::SCROLL_SELECTED => {
            let selected = read_java_string(reader).ok()?;
            vec![
                DecodedField { name: "selected".to_string(), value: DecodedValue::Text(selected) },
            ]
        }
        EnumPacketClient::GUI_ERROR => {
            let error_code = reader.read_i32_be().ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "error_code".to_string(), value: DecodedValue::Text(error_code.to_string()) },
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketClient::GUI_CLOSE => {
            let close_code = reader.read_i32_be().ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "close_code".to_string(), value: DecodedValue::Text(close_code.to_string()) },
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketClient::OPEN_BOOK => {
            let x = reader.read_i32_be().ok()?;
            let y = reader.read_i32_be().ok()?;
            let z = reader.read_i32_be().ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "x".to_string(), value: DecodedValue::Text(x.to_string()) },
                DecodedField { name: "y".to_string(), value: DecodedValue::Text(y.to_string()) },
                DecodedField { name: "z".to_string(), value: DecodedValue::Text(z.to_string()) },
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketClient::CONFIG => {
            let config_type = reader.read_i32_be().ok()?;
            if config_type == 0 {
                let font = read_java_string(reader).ok()?;
                let size = reader.read_i32_be().ok()?;
                vec![
                    DecodedField { name: "config_type".to_string(), value: DecodedValue::Text("FONT".to_string()) },
                    DecodedField { name: "font".to_string(), value: DecodedValue::Text(font) },
                    DecodedField { name: "size".to_string(), value: DecodedValue::Text(size.to_string()) },
                ]
            } else {
                vec![
                    DecodedField { name: "config_type".to_string(), value: DecodedValue::Text(config_type.to_string()) },
                ]
            }
        }
        EnumPacketClient::SCRIPT_OVERLAY_CLOSE | EnumPacketClient::STOP_SOUND_FOR => {
            let id = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "id".to_string(), value: DecodedValue::Text(id.to_string()) },
            ]
        }
        EnumPacketClient::PLAYER_UPDATE_SKIN_OVERLAYS => {
            let player_name = read_java_string(reader).ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "player_name".to_string(), value: DecodedValue::Text(player_name) },
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketClient::DISABLE_MOUSE_INPUT => {
            let length = reader.read_i64_be().ok()?;
            let buttons = read_java_string(reader).ok()?;
            vec![
                DecodedField { name: "length_ms".to_string(), value: DecodedValue::Text(length.to_string()) },
                DecodedField { name: "buttons".to_string(), value: DecodedValue::Text(buttons) },
            ]
        }
        EnumPacketClient::PLAY_SOUND_TO => {
            let id = reader.read_i32_be().ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "sound_id".to_string(), value: DecodedValue::Text(id.to_string()) },
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketClient::MESSAGE => {
            let description = read_java_string(reader).ok()?;
            let message = read_java_string(reader).ok()?;
            vec![
                DecodedField { name: "description".to_string(), value: DecodedValue::Text(description) },
                DecodedField { name: "message".to_string(), value: DecodedValue::Text(message) },
            ]
        }
        EnumPacketClient::SYNCRECIPES_WORKBENCH |
        EnumPacketClient::SYNCRECIPES_CARPENTRYBENCH |
        EnumPacketClient::ISGUIOPEN |
        EnumPacketClient::SWING_PLAYER_ARM |
        EnumPacketClient::PAUSE_SOUNDS |
        EnumPacketClient::CONTINUE_SOUNDS |
        EnumPacketClient::STOP_SOUNDS => {
            vec![]
        }
        EnumPacketClient::SYNCRECIPES_ADD |
        EnumPacketClient::QUEST_COMPLETION |
        EnumPacketClient::UPDATE_NPC |
        EnumPacketClient::ROLE |
        EnumPacketClient::GUI_REDSTONE |
        EnumPacketClient::GUI_WAYPOINT |
        EnumPacketClient::GUI_DATA |
        EnumPacketClient::CLONE |
        EnumPacketClient::SCRIPT_OVERLAY_DATA |
        EnumPacketClient::UPDATE_ANIMATIONS |
        EnumPacketClient::OVERLAY_QUEST_TRACKING |
        EnumPacketClient::PLAY_SOUND_TO_NO_ID => {
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        _ => {
            let remaining = reader.remaining().len();
            vec![
                DecodedField { name: "raw_data".to_string(), value: DecodedValue::Text(format!("[{} bytes]", remaining)) },
            ]
        }
    };

    Some(DecodedStruct { name, fields })
}

fn decode_server_packet(packet_id: i32, reader: &mut ModPacketReader) -> Option<DecodedStruct> {
    let packet_type = EnumPacketServer::from_id(packet_id)?;
    let name = packet_type.name().to_string();

    let fields = match packet_type {
        EnumPacketServer::Delete |
        EnumPacketServer::RemoteMainMenu |
        EnumPacketServer::RemoteDelete |
        EnumPacketServer::RemoteReset |
        EnumPacketServer::RemoteTpToNpc => {
            let entity_id = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "entity_id".to_string(), value: DecodedValue::Text(entity_id.to_string()) },
            ]
        }
        EnumPacketServer::DialogNpcSet => {
            let slot = reader.read_i32_be().ok()?;
            let dialog_id = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "slot".to_string(), value: DecodedValue::Text(slot.to_string()) },
                DecodedField { name: "dialog_id".to_string(), value: DecodedValue::Text(dialog_id.to_string()) },
            ]
        }
        EnumPacketServer::DialogNpcRemove |
        EnumPacketServer::FactionSet |
        EnumPacketServer::DialogsGet |
        EnumPacketServer::DialogGet |
        EnumPacketServer::DialogRemove |
        EnumPacketServer::DialogsGetFromDialog |
        EnumPacketServer::QuestsGetFromQuest |
        EnumPacketServer::QuestsGet |
        EnumPacketServer::QuestGet |
        EnumPacketServer::QuestRemove |
        EnumPacketServer::QuestCategoryGet |
        EnumPacketServer::QuestCategoryRemove |
        EnumPacketServer::DialogCategoryGet |
        EnumPacketServer::DialogCategoryRemove |
        EnumPacketServer::BankGet |
        EnumPacketServer::BankRemove |
        EnumPacketServer::FactionGet |
        EnumPacketServer::FactionRemove |
        EnumPacketServer::TagGet |
        EnumPacketServer::TagRemove |
        EnumPacketServer::TransportsGet |
        EnumPacketServer::TransportRemove |
        EnumPacketServer::TransportCategoryRemove |
        EnumPacketServer::RecipeGet |
        EnumPacketServer::RecipeRemove |
        EnumPacketServer::NaturalSpawnGet |
        EnumPacketServer::NaturalSpawnRemove |
        EnumPacketServer::RecipesGet |
        EnumPacketServer::CloneList |
        EnumPacketServer::CloneTagList |
        EnumPacketServer::DimensionTeleport |
        EnumPacketServer::RoleCompanionUpdate => {
            let id = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "id".to_string(), value: DecodedValue::Text(id.to_string()) },
            ]
        }
        EnumPacketServer::Gui => {
            let gui_type = reader.read_i32_be().ok()?;
            let x = reader.read_i32_be().ok()?;
            let y = reader.read_i32_be().ok()?;
            let z = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "gui_type".to_string(), value: DecodedValue::Text(gui_type.to_string()) },
                DecodedField { name: "x".to_string(), value: DecodedValue::Text(x.to_string()) },
                DecodedField { name: "y".to_string(), value: DecodedValue::Text(y.to_string()) },
                DecodedField { name: "z".to_string(), value: DecodedValue::Text(z.to_string()) },
            ]
        }
        EnumPacketServer::GetTileEntity => {
            let x = reader.read_i32_be().ok()?;
            let y = reader.read_i32_be().ok()?;
            let z = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "x".to_string(), value: DecodedValue::Text(x.to_string()) },
                DecodedField { name: "y".to_string(), value: DecodedValue::Text(y.to_string()) },
                DecodedField { name: "z".to_string(), value: DecodedValue::Text(z.to_string()) },
            ]
        }
        EnumPacketServer::SpawnMob | EnumPacketServer::MobSpawner => {
            let from_server = reader.read_bool().ok()?;
            let x = reader.read_i32_be().ok()?;
            let y = reader.read_i32_be().ok()?;
            let z = reader.read_i32_be().ok()?;
            if from_server {
                let clone_name = read_java_string(reader).ok()?;
                let tab = reader.read_i32_be().ok()?;
                vec![
                    DecodedField { name: "from_server".to_string(), value: DecodedValue::Text("true".to_string()) },
                    DecodedField { name: "x".to_string(), value: DecodedValue::Text(x.to_string()) },
                    DecodedField { name: "y".to_string(), value: DecodedValue::Text(y.to_string()) },
                    DecodedField { name: "z".to_string(), value: DecodedValue::Text(z.to_string()) },
                    DecodedField { name: "clone_name".to_string(), value: DecodedValue::Text(clone_name) },
                    DecodedField { name: "tab".to_string(), value: DecodedValue::Text(tab.to_string()) },
                ]
            } else {
                let nbt_len = reader.remaining().len();
                vec![
                    DecodedField { name: "from_server".to_string(), value: DecodedValue::Text("false".to_string()) },
                    DecodedField { name: "x".to_string(), value: DecodedValue::Text(x.to_string()) },
                    DecodedField { name: "y".to_string(), value: DecodedValue::Text(y.to_string()) },
                    DecodedField { name: "z".to_string(), value: DecodedValue::Text(z.to_string()) },
                    DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
                ]
            }
        }
        EnumPacketServer::ClonePreSave => {
            let clone_name = read_java_string(reader).ok()?;
            let tab = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "clone_name".to_string(), value: DecodedValue::Text(clone_name) },
                DecodedField { name: "tab".to_string(), value: DecodedValue::Text(tab.to_string()) },
            ]
        }
        EnumPacketServer::CloneSave => {
            let clone_name = read_java_string(reader).ok()?;
            let tab = reader.read_i32_be().ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "clone_name".to_string(), value: DecodedValue::Text(clone_name) },
                DecodedField { name: "tab".to_string(), value: DecodedValue::Text(tab.to_string()) },
                DecodedField { name: "nbt_extra".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketServer::CloneRemove => {
            let tab = reader.read_i32_be().ok()?;
            let clone_name = read_java_string(reader).ok()?;
            vec![
                DecodedField { name: "tab".to_string(), value: DecodedValue::Text(tab.to_string()) },
                DecodedField { name: "clone_name".to_string(), value: DecodedValue::Text(clone_name) },
            ]
        }
        EnumPacketServer::LinkedSet |
        EnumPacketServer::LinkedAdd |
        EnumPacketServer::LinkedRemove |
        EnumPacketServer::AnimationGet |
        EnumPacketServer::AnimationDelete => {
            let name_str = read_java_string(reader).ok()?;
            vec![
                DecodedField { name: "name".to_string(), value: DecodedValue::Text(name_str) },
            ]
        }
        EnumPacketServer::AnimationSave => {
            let prev_name = read_java_string(reader).ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "prev_name".to_string(), value: DecodedValue::Text(prev_name) },
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketServer::DialogSave | EnumPacketServer::QuestSave => {
            let category_id = reader.read_i32_be().ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "category_id".to_string(), value: DecodedValue::Text(category_id.to_string()) },
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketServer::TransportSave => {
            let category_id = reader.read_i32_be().ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "category_id".to_string(), value: DecodedValue::Text(category_id.to_string()) },
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketServer::TransportCategorySave => {
            let category_name = read_java_string(reader).ok()?;
            let category_id = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "category_name".to_string(), value: DecodedValue::Text(category_name) },
                DecodedField { name: "category_id".to_string(), value: DecodedValue::Text(category_id.to_string()) },
            ]
        }
        EnumPacketServer::TraderMarketSave => {
            let market = read_java_string(reader).ok()?;
            let load = reader.read_bool().ok()?;
            vec![
                DecodedField { name: "market".to_string(), value: DecodedValue::Text(market) },
                DecodedField { name: "load".to_string(), value: DecodedValue::Text(load.to_string()) },
            ]
        }
        EnumPacketServer::TransformLoad => {
            let transform = reader.read_bool().ok()?;
            vec![
                DecodedField { name: "transform".to_string(), value: DecodedValue::Text(transform.to_string()) },
            ]
        }
        EnumPacketServer::PlayerDataGet => {
            let data_type = reader.read_i32_be().ok()?;
            if data_type != 0 {
                let player_name = read_java_string(reader).ok()?;
                vec![
                    DecodedField { name: "data_type".to_string(), value: DecodedValue::Text(data_type.to_string()) },
                    DecodedField { name: "player_name".to_string(), value: DecodedValue::Text(player_name) },
                ]
            } else {
                vec![
                    DecodedField { name: "data_type".to_string(), value: DecodedValue::Text("Players".to_string()) },
                ]
            }
        }
        EnumPacketServer::QuestDialogGetTitle => {
            let dialog1 = reader.read_i32_be().ok()?;
            let dialog2 = reader.read_i32_be().ok()?;
            let dialog3 = reader.read_i32_be().ok()?;
            vec![
                DecodedField { name: "dialog1_id".to_string(), value: DecodedValue::Text(dialog1.to_string()) },
                DecodedField { name: "dialog2_id".to_string(), value: DecodedValue::Text(dialog2.to_string()) },
                DecodedField { name: "dialog3_id".to_string(), value: DecodedValue::Text(dialog3.to_string()) },
            ]
        }
        EnumPacketServer::MerchantUpdate => {
            let entity_id = reader.read_i32_be().ok()?;
            let remaining = reader.remaining().len();
            vec![
                DecodedField { name: "entity_id".to_string(), value: DecodedValue::Text(entity_id.to_string()) },
                DecodedField { name: "recipe_list".to_string(), value: DecodedValue::Text(format!("[{} bytes]", remaining)) },
            ]
        }
        EnumPacketServer::CustomGuiButton => {
            let nbt_len = reader.remaining().len().saturating_sub(4);
            vec![
                DecodedField { name: "gui_nbt".to_string(), value: DecodedValue::Text(format!("[NBT ~{} bytes]", nbt_len)) },
                DecodedField { name: "button_id".to_string(), value: DecodedValue::Text("(at end)".to_string()) },
            ]
        }
        EnumPacketServer::CustomGuiScrollClick => {
            vec![
                DecodedField { name: "gui_nbt".to_string(), value: DecodedValue::Text("[NBT]".to_string()) },
                DecodedField { name: "scroll_id".to_string(), value: DecodedValue::Text("(int)".to_string()) },
                DecodedField { name: "index".to_string(), value: DecodedValue::Text("(int)".to_string()) },
                DecodedField { name: "selection".to_string(), value: DecodedValue::Text("(data)".to_string()) },
                DecodedField { name: "double_click".to_string(), value: DecodedValue::Text("(bool)".to_string()) },
            ]
        }
        EnumPacketServer::CustomGuiUnfocused => {
            vec![
                DecodedField { name: "gui_nbt".to_string(), value: DecodedValue::Text("[NBT]".to_string()) },
                DecodedField { name: "element_id".to_string(), value: DecodedValue::Text("(int)".to_string()) },
            ]
        }
        EnumPacketServer::JobSpawnerAdd => {
            let from_clone = reader.read_bool().ok()?;
            if from_clone {
                let clone_name = read_java_string(reader).ok()?;
                let tab = reader.read_i32_be().ok()?;
                let slot = reader.read_i32_be().ok()?;
                vec![
                    DecodedField { name: "from_clone".to_string(), value: DecodedValue::Text("true".to_string()) },
                    DecodedField { name: "clone_name".to_string(), value: DecodedValue::Text(clone_name) },
                    DecodedField { name: "tab".to_string(), value: DecodedValue::Text(tab.to_string()) },
                    DecodedField { name: "slot".to_string(), value: DecodedValue::Text(slot.to_string()) },
                ]
            } else {
                let slot = reader.read_i32_be().ok()?;
                let nbt_len = reader.remaining().len();
                vec![
                    DecodedField { name: "from_clone".to_string(), value: DecodedValue::Text("false".to_string()) },
                    DecodedField { name: "slot".to_string(), value: DecodedValue::Text(slot.to_string()) },
                    DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
                ]
            }
        }
        EnumPacketServer::QuestOpenGui => {
            let gui_id = reader.read_i32_be().ok()?;
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "gui_id".to_string(), value: DecodedValue::Text(gui_id.to_string()) },
                DecodedField { name: "quest_nbt".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        EnumPacketServer::NpcMenuClose |
        EnumPacketServer::RemoteFreeze |
        EnumPacketServer::MainmenuAIGet |
        EnumPacketServer::MainmenuInvGet |
        EnumPacketServer::MainmenuStatsGet |
        EnumPacketServer::MainmenuDisplayGet |
        EnumPacketServer::MainmenuAdvancedGet |
        EnumPacketServer::TransformGet |
        EnumPacketServer::JobGet |
        EnumPacketServer::RoleGet |
        EnumPacketServer::TransportGetLocation |
        EnumPacketServer::ScriptDataGet |
        EnumPacketServer::EventScriptDataGet |
        EnumPacketServer::ScriptPlayerGet |
        EnumPacketServer::ScriptForgeGet |
        EnumPacketServer::ScriptGlobalNPCGet |
        EnumPacketServer::ScriptItemDataGet |
        EnumPacketServer::ScriptGlobalGuiDataGet |
        EnumPacketServer::LinkedGetAll |
        EnumPacketServer::BanksGet |
        EnumPacketServer::DialogCategoriesGet |
        EnumPacketServer::TransportCategoriesGet |
        EnumPacketServer::FactionsGet |
        EnumPacketServer::TagsGet |
        EnumPacketServer::NpcTagsGet |
        EnumPacketServer::QuestCategoriesGet |
        EnumPacketServer::NaturalSpawnGetAll |
        EnumPacketServer::DimensionsGet |
        EnumPacketServer::DialogNpcGet |
        EnumPacketServer::AnimationListGet |
        EnumPacketServer::AnimationAdd |
        EnumPacketServer::RemoteNpcsGet |
        EnumPacketServer::PlayerRider |
        EnumPacketServer::PlayerDataRegen |
        EnumPacketServer::MovingPathGet |
        EnumPacketServer::CloneAllTags |
        EnumPacketServer::CloneAllTagsShort |
        EnumPacketServer::JobSpawnerRemove |
        EnumPacketServer::IsGuiOpen => {
            vec![]
        }
        EnumPacketServer::MainmenuAISave |
        EnumPacketServer::MainmenuInvSave |
        EnumPacketServer::MainmenuStatsSave |
        EnumPacketServer::MainmenuDisplaySave |
        EnumPacketServer::MainmenuAdvancedSave |
        EnumPacketServer::ModelDataSave |
        EnumPacketServer::TransformSave |
        EnumPacketServer::JobSave |
        EnumPacketServer::RoleSave |
        EnumPacketServer::TagSet |
        EnumPacketServer::ScriptDataSave |
        EnumPacketServer::BankSave |
        EnumPacketServer::DialogCategorySave |
        EnumPacketServer::QuestCategorySave |
        EnumPacketServer::QuestRewardSave |
        EnumPacketServer::FactionSave |
        EnumPacketServer::TagSave |
        EnumPacketServer::NaturalSpawnSave |
        EnumPacketServer::RecipeSave |
        EnumPacketServer::MovingPathSave |
        EnumPacketServer::SaveTileEntity |
        EnumPacketServer::MailOpenSetup |
        EnumPacketServer::SpawnRider |
        EnumPacketServer::CustomGuiClose |
        EnumPacketServer::ScriptGlobalGuiDataSave |
        EnumPacketServer::ServerUpdateSkinOverlays => {
            let nbt_len = reader.remaining().len();
            vec![
                DecodedField { name: "nbt_data".to_string(), value: DecodedValue::Text(format!("[NBT {} bytes]", nbt_len)) },
            ]
        }
        _ => {
            let remaining = reader.remaining().len();
            vec![
                DecodedField { name: "raw_data".to_string(), value: DecodedValue::Text(format!("[{} bytes]", remaining)) },
            ]
        }
    };

    Some(DecodedStruct { name, fields })
}

pub fn try_decode(payload: &[u8], bound: Bound) -> Option<DecodedStruct> {
    if payload.len() < 4 {
        return None;
    }

    let mut reader = ModPacketReader::new(payload);
    let packet_id = reader.read_i32_be().ok()?;

    match bound {
        Bound::Client => decode_client_packet(packet_id, &mut reader),
        Bound::Server => decode_server_packet(packet_id, &mut reader),
    }
}

pub struct CustomNpcsPayloadDecoder;

impl Default for CustomNpcsPayloadDecoder {
    fn default() -> Self { Self }
}

impl CustomPayloadDecoder for CustomNpcsPayloadDecoder {
    fn channel(&self) -> &'static str { CHANNEL }

    fn try_decode(&self, payload: &[u8], bound: Bound) -> Option<DecodedStruct> {
        try_decode(payload, bound)
    }
}

pub fn register_customnpcs_decoder() {
    crate::core::custom_payload::register_decoder::<CustomNpcsPayloadDecoder>();
}
