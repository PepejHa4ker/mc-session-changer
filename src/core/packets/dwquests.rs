// use serde_json::json;
// // use crate::core::custom_payload::DecodedStruct; // уже есть у тебя
//
// fn try_decode(&self, payload: &[u8], bound: Bound) -> Option<DecodedStruct> {
//     let mut reader = ModPacketReader::new(payload);
//     let packet_id = reader.read_varint().ok()?;
//     match (packet_id, bound) {
//         (1, Bound::Server) => {
//             let mut tabs_out = Vec::new();
//
//             let tab_counts = reader.read_i32_be().ok()?;
//             for _ in 0..tab_counts {
//                 let tab_id = reader.read_string_varint().ok()?;
//                 let icon = reader.read_string_varint().ok()?;
//                 let sort_index = reader.read_i32_be().ok()?;
//
//                 let mut quests_out = Vec::new();
//                 let quest_count = reader.read_i32_be().ok()?;
//                 for _ in 0..quest_count {
//                     let q_tab_id   = reader.read_string_varint().ok()?;
//                     let quest_id   = reader.read_string_varint().ok()?;
//                     let display_x  = reader.read_f32_be().ok()?;
//                     let display_y  = reader.read_f32_be().ok()?;
//                     let display_name = reader.read_string_varint().ok()?;
//                     let description  = reader.read_string_varint().ok()?;
//                     let icon_item    = reader.read_string_varint().ok()?;
//
//                     let rarity     = reader.read_string_varint().ok()?;
//                     let icon_size  = reader.read_string_varint().ok()?;
//                     let node_type  = reader.read_string_varint().ok()?;
//                     let tasks_type = reader.read_string_varint().ok()?;
//
//                     // parents
//                     let parents_count = reader.read_i32_be().ok()?;
//                     let mut parents_out = Vec::with_capacity(parents_count as usize);
//                     for _ in 0..parents_count {
//                         let parent_tab_id   = reader.read_string_varint().ok()?;
//                         let parent_quest_id = reader.read_string_varint().ok()?;
//                         let parent_type     = reader.read_string_varint().ok()?;
//                         let line_type       = reader.read_string_varint().ok()?;
//
//                         parents_out.push(ParentLink {
//                             parent_tab_id,
//                             parent_quest_id,
//                             parent_type,
//                             line_type,
//                         });
//                     }
//
//                     // tasks
//                     let tasks_count = reader.read_i32_be().ok()?;
//                     let mut tasks_out = Vec::with_capacity(tasks_count as usize);
//                     for _ in 0..tasks_count {
//                         let task_type = reader.read_string_varint().ok()?;
//                         let task_id   = reader.read_i32_be().ok()?;
//
//                         let refs_size = reader.read_i32_be().ok()?;
//                         let mut refs = Vec::with_capacity(refs_size as usize);
//                         for _ in 0..refs_size { refs.push(reader.read_i32_be().ok()?); }
//
//                         let task = match task_type.as_str() {
//                             "ENTITY_KILL" => {
//                                 let target    = reader.read_i32_be().ok()?;
//                                 let entity_id = reader.read_string_varint().ok()?;
//                                 let extra_a   = reader.read_string_varint().ok()?;
//                                 let extra_b   = reader.read_string_varint().ok()?;
//                                 Task::ENTITY_KILL { id: task_id, refs, target, entity_id, extra_a, extra_b }
//                             }
//                             "TASK_CONFIRM" => {
//                                 Task::TASK_CONFIRM { id: task_id, refs }
//                             }
//                             "PLAY_TIME" => {
//                                 let target = reader.read_i32_be().ok()?;
//                                 Task::PLAY_TIME { id: task_id, refs, target }
//                             }
//                             other => {
//                                 // default ветка — читаем только target
//                                 let target = reader.read_i32_be().ok()?;
//                                 Task::OTHER { kind: other.to_string(), id: task_id, refs, target }
//                             }
//                         };
//
//                         tasks_out.push(task);
//                     }
//
//                     quests_out.push(Quest {
//                         tab_id: q_tab_id,
//                         quest_id,
//                         display_x,
//                         display_y,
//                         display_name,
//                         description,
//                         icon_item,
//                         rarity,
//                         icon_size,
//                         node_type,
//                         tasks_type,
//                         parents: parents_out,
//                         tasks: tasks_out,
//                     });
//                 }
//
//                 tabs_out.push(Tab { tab_id, icon, sort_index, quests: quests_out });
//             }
//
//             let decoded = DwQuestsDecoded { tabs: tabs_out };
//
//             // 1) если DecodedStruct умеет принимать JSON:
//             // Some(DecodedStruct::from_json("DwQuests", serde_json::to_value(decoded).ok()?))
//
//             // 2) или если у тебя есть вариант enum:
//             // Some(DecodedStruct::DwQuests(decoded))
//
//             // оставлю JSON-вариант как самый универсальный:
//             return Some(DecodedStruct::from_json("DwQuests", serde_json::to_value(decoded).ok()?));
//         }
//         _ => None,
//     }
// }
