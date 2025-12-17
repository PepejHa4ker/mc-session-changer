use crate::{generate_mod_payload_decoders, mod_packets};

mod_packets! {
    channel: dwcity,
    packets {
        EmeraldBalanceRequest(1, C) {
            username: String
        },
        EmeraldBalanceResponse(1, S) {
            username: String,
            emeralds: f32
        },
        CoinBalanceRequest(3, C) {},
        CoinBalanceResponse(3, S) {
            coins: f32
        },
        PlayerStatsResponse(9, S) {
            username: String,
            first_int: i32,
            second_int: i32,

            last_updated_at: i64,
            total_playtime: i64,
            playtime: i64,
            blocks_mined: i64,
            blocks_placed: i64,
            player_kills: i64,
            counter_h: i64,
            mob_kills: i64,
            dragon_kills: i64,
            counter_i: i64,
            crafted_items: i64,
            walked_distance: f64,
            total_distance: f64,

            jumps: i64,
            consumed: i64,
            enchanted: i64,
            global_messages: i64,
            casino: i64,

            grades: i64,
            timer_b: i64,
            timer_c: i64,
            timer_d: i64
        },
        PlayerStatsRequest(9, C) {
            username: String
        },
        SendClientName(18, C) {
            username: String
        },
        UpdateTileLimits(27, S) {
            limits: Vec<String>
        }
    }
}

generate_mod_payload_decoders! {
    "dwcity" => dwcity,
}