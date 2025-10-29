use crate::hooks::jhook::{JNIHookManager};
use anyhow::{Result};
use jni::sys::{JavaVM};
use crate::hooks::packet::hwid_hook::HwidHook;
use crate::hooks::packet::packet_write_hook::PacketWriteHook;

pub mod jhook;
pub mod opengl;
mod packet;

// impl RunTickHook {
//     fn on_tick(&self, env: &mut JNIEnv, minecraft_instance: &JObject) -> Result<()> {
//         if !self.is_in_game(env, minecraft_instance)? {
//             return Ok(());
//         }
//
//         let game_settings = self.get_game_settings(env, minecraft_instance)?;
//
//         let key_bind_attack = self.get_key_bind_attack(env, &game_settings)?;
//
//         if !self.is_key_pressed(env, &key_bind_attack)? {
//             return Ok(());
//         }
//
//         let object_mouse_over = self.get_object_mouse_over(env, minecraft_instance)?;
//
//         if !self.is_block_hit(env, &object_mouse_over)? {
//             return Ok(());
//         }
//
//         let player_controller = self.get_player_controller(env, minecraft_instance)?;
//
//         self.set_break_progress(env, &player_controller, 0)?;
//
//         let (x, y, z) = self.get_block_coordinates(env, &object_mouse_over)?;
//
//         let block = self.get_block_at(env, minecraft_instance, x, y, z)?;
//
//         let current_hardness = self.get_current_hardness(env, &player_controller)?;
//
//         let player = self.get_player(env, minecraft_instance)?;
//         let world = self.get_world(env, minecraft_instance)?;
//
//         let block_hardness =
//             self.get_block_relative_hardness(env, block, player, world, x, y, z)?;
//
//         // Обновляем прогресс ломания
//         let new_hardness = current_hardness + block_hardness;
//         self.set_current_hardness(env, player_controller, new_hardness)?;
//
//         Ok(())
//     }
//
//     fn is_in_game(&self, env: &mut JNIEnv, minecraft_instance: &JObject) -> Result<bool> {
//         let player = self.get_player(env, minecraft_instance)?;
//         let world = self.get_world(env, minecraft_instance)?;
//
//         Ok(!player.is_null() && !world.is_null())
//     }
//
//     fn get_game_settings<'a>(
//         &self,
//         env: &mut JNIEnv<'a>,
//         minecraft_instance: &JObject,
//     ) -> Result<JObject<'a>> {
//         env.get_field(
//             minecraft_instance,
//             "gameSettings",
//             "Lnet/minecraft/client/settings/GameSettings;",
//         )
//         .context("Failed to get gameSettings field")?
//         .l()
//         .context("Failed to get gameSettings object")
//     }
//
//     fn get_key_bind_attack<'a>(
//         &self,
//         env: &mut JNIEnv<'a>,
//         game_settings: &JObject,
//     ) -> Result<JObject<'a>> {
//         env.get_field(
//             game_settings,
//             "keyBindAttack",
//             "Lnet/minecraft/client/settings/KeyBinding;",
//         )
//         .context("Faled to get keyBindAttack field")?
//         .l()
//         .context("Failed to get keyBindAttack object")
//     }
//
//     fn is_key_pressed(&self, env: &mut JNIEnv, key_binding: &JObject) -> Result<bool> {
//         env.call_method(&key_binding, "getIsKeyPressed", "()Z", &[])
//             .context("Failed to call getIsKeyPressed")?
//             .z()
//             .context("Failed to get result of getIsKeyPressed")
//     }
//
//     fn get_object_mouse_over<'a>(
//         &self,
//         env: &mut JNIEnv<'a>,
//         minecraft_instance: &JObject,
//     ) -> Result<JObject<'a>> {
//         env.get_field(
//             minecraft_instance,
//             "objectMouseOver",
//             "Lnet/minecraft/util/MovingObjectPosition;",
//         )
//         .context("Failed to get objectMouseOver field")?
//         .l()
//         .context("Failed to get objectMouseOver object")
//     }
//
//     fn is_block_hit(&self, env: &mut JNIEnv, moving_object_position: &JObject) -> Result<bool> {
//         if moving_object_position.is_null() {
//             return Ok(false);
//         }
//
//         let type_of_hit = env
//             .get_field(
//                 moving_object_position,
//                 "typeOfHit",
//                 "Lnet/minecraft/util/MovingObjectPosition$MovingObjectType;",
//             )
//             .context("Failed to get typeOfHit field")?
//             .l()
//             .context("Failed to get typeOfHit object")?;
//
//         let mov_obj_type_class =
//             env.find_class("net/minecraft/util/MovingObjectPosition$MovingObjectType")?;
//
//         let block = env.get_static_field(
//             &mov_obj_type_class,
//             "BLOCK",
//             "Lnet/minecraft/util/MovingObjectPosition$MovingObjectType;",
//         )
//             .context("Failed to get BLOCK field")
//             ?.l()
//             .context("Failed to get BLOCK object")?;
//
//
//         Ok(env.is_same_object(type_of_hit, block)?)
//     }
//
//     fn get_player_controller<'a>(
//         &self,
//         env: &mut JNIEnv<'a>,
//         minecraft_instance: &JObject,
//     ) -> Result<JObject<'a>> {
//         env.get_field(
//             minecraft_instance,
//             "playerController",
//             "Lnet/minecraft/client/multiplayer/PlayerControllerMP;",
//         )
//         .context("Failed to get playerController field")?
//         .l()
//         .context("Failed to get playerController object")
//     }
//
//     fn set_break_progress(
//         &self,
//         env: &mut JNIEnv,
//         player_controller: &JObject,
//         value: i32,
//     ) -> Result<()> {
//         env.set_field(player_controller, "field_78781_i", "I", JValue::Int(value))
//             .context("Failed to set break progress")
//     }
//
//     fn get_block_coordinates(
//         &self,
//         env: &mut JNIEnv,
//         moving_object_position: &JObject,
//     ) -> Result<(i32, i32, i32)> {
//         let x = env
//             .get_field(moving_object_position, "blockX", "I")
//             .context("Failed to get blockX field")?
//             .i()
//             .context("Failed to get blockX value")?;
//         let y = env
//             .get_field(moving_object_position, "blockY", "I")
//             .context("Failed to get blockY field")?
//             .i()
//             .context("Failed to get blockY value")?;
//         let z = env
//             .get_field(moving_object_position, "blockZ", "I")
//             .context("Failed to get blockZ field")?
//             .i()
//             .context("Failed to get blockZ value")?;
//
//         Ok((x, y, z))
//     }
//
//     fn get_block_at(
//         &self,
//         env: &mut JNIEnv,
//         minecraft_instance: &JObject,
//         x: i32,
//         y: i32,
//         z: i32,
//     ) -> Result<JObject> {
//         let world = self.get_world(env, minecraft_instance)?;
//         let world_class = env.get_object_class(world)?;
//
//         let method_id = env
//             .get_method_id(&world_class, "getBlock", "(III)Lnet/minecraft/block/Block;")
//             .context("Failed to find getBlock method")?;
//
//         let args = [
//             jni::objects::JValue::Int(x),
//             jni::objects::JValue::Int(y),
//             jni::objects::JValue::Int(z),
//         ];
//
//         env.call_method_unchecked(world, method_id, jni::signature::ReturnType::Object, &args)?
//             .l()
//             .context("Failed to get block")
//     }
//
//     fn get_player<'a>(
//         &self,
//         env: &mut JNIEnv<'a>,
//         minecraft_instance: &JObject,
//     ) -> Result<JObject<'a>> {
//         env.get_field(
//             minecraft_instance,
//             "thePlayer",
//             "Lnet/minecraft/entity/player/EntityPlayer;",
//         )
//         .context("Failed to get thePlayer field")?
//         .l()
//         .context("Failed to get thePlayer object")
//     }
//
//     fn get_world<'a>(
//         &self,
//         env: &mut JNIEnv<'a>,
//         minecraft_instance: &JObject,
//     ) -> Result<JObject<'a>> {
//         env.get_field(
//             minecraft_instance,
//             "theWorld",
//             "Lnet/minecraft/client/multiplayer/WorldClient;",
//         )
//         .context("Failed to get theWorld field")?
//         .l()
//         .context("Failed to get theWorld object")
//     }
//
//     fn get_current_hardness(&self, env: &mut JNIEnv, player_controller: &JObject) -> Result<f32> {
//         env.get_field(player_controller, "field_78770_f", "F")
//             .context("Failed to get current hardness field")?
//             .f()
//             .context("Failed to get current hardness")
//     }
//
//     fn set_current_hardness(
//         &self,
//         env: &mut JNIEnv,
//         player_controller: JObject,
//         value: f32,
//     ) -> Result<()> {
//         let controller_class = env.get_object_class(player_controller)?;
//
//         let field_id = env
//             .get_field_id(&controller_class, "field_78770_f", "F")
//             .context("Failed to find current hardness field")?;
//
//         env.set_float_field(player_controller, field_id, value)
//             .context("Failed to set current hardness")
//     }
//
//     fn get_block_relative_hardness(
//         &self,
//         env: &mut JNIEnv,
//         block: JObject,
//         player: JObject,
//         world: JObject,
//         x: i32,
//         y: i32,
//         z: i32,
//     ) -> Result<f32> {
//         let block_class = env.get_object_class(block)?;
//
//         let method_id = env
//             .get_method_id(
//                 &block_class,
//                 "getPlayerRelativeBlockHardness",
//                 "(Lnet/minecraft/entity/player/EntityPlayer;Lnet/minecraft/world/World;III)F",
//             )
//             .context("Failed to find getPlayerRelativeBlockHardness method")?;
//
//         let args = [
//             jni::objects::JValue::Object(&player),
//             jni::objects::JValue::Object(&world),
//             jni::objects::JValue::Int(x),
//             jni::objects::JValue::Int(y),
//             jni::objects::JValue::Int(z),
//         ];
//
//         let result = env
//             .call_method_unchecked(
//                 block,
//                 method_id,
//                 jni::signature::ReturnType::Primitive(jni::signature::Primitive::Float),
//                 &args,
//             )
//             .context("Failed to call getPlayerRelativeBlockHardness")?;
//
//         Ok(result.f()?)
//     }
// }

pub unsafe fn setup_jni_hooks(jvm: *mut JavaVM) -> Result<()> {
    tracing::info!("Setting up packet hooks...");

    let hook_manager = JNIHookManager::obtain(jvm);
    tracing::info!("Jvm obtained");

    let packet_write_hook = PacketWriteHook;
    tracing::info!("Write hook");
    match hook_manager.hook_method(
        "net/minecraft/util/MessageSerializer",
        "encode",
        "(Lio/netty/channel/ChannelHandlerContext;Lnet/minecraft/network/Packet;Lio/netty/buffer/ByteBuf;)V",
        packet_write_hook,
    ) {
        Ok(_) => tracing::info!("Successfully hooked MessageSerializer::encode"),
        Err(e) => {
            tracing::warn!("Failed to hook MessageSerializer::encode: {}", e)
        },
    }

    let hwid_hook = HwidHook;
    tracing::info!("Hwid hook");
    match hook_manager.hook_method(
        "ru/sky_drive/dw/pF",
        "do",
        "(Ljava/io/DataOutput;)V",
        hwid_hook,
    ) {
        Ok(_) => tracing::info!("Successfully hooked HWID::writeData"),
        Err(e) => {
            tracing::warn!("Failed to hook HWID::writeData: {}", e)
        },
    }

    Ok(())
}
