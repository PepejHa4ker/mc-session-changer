// src/core/custom_payload.rs
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::core::packets::Bound;
use crate::core::packets::dwcity::register_mod_payload_decoders;
use crate::core::packets::customnpcs::register_customnpcs_decoder;

#[derive(Debug, Clone)]
pub enum DecodedValue {
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<DecodedValue>),
    Struct(DecodedStruct),
    Null,
}

/// Поле структуры
#[derive(Debug, Clone)]
pub struct DecodedField {
    pub name: String,
    pub value: DecodedValue,
}

/// Корневая структура расшифрованного пакета/вложенной сущности
#[derive(Debug, Clone)]
pub struct DecodedStruct {
    pub name: String,
    pub fields: Vec<DecodedField>,
}

/// Конвертация пользовательских типов в DecodedValue
pub trait ToDecodedValue {
    fn to_decoded_value(&self) -> DecodedValue;
}

/* -------- базовые типы -------- */

macro_rules! impl_td_text {
    ($($t:ty),+ $(,)?) => {
        $(impl ToDecodedValue for $t {
            fn to_decoded_value(&self) -> DecodedValue {
                DecodedValue::Text(self.to_string())
            }
        })+
    }
}
impl_td_text!(bool, i8,i16,i32,i64,isize, u8,u16,u32,u64,usize, f32,f64, String);

impl ToDecodedValue for &str {
    fn to_decoded_value(&self) -> DecodedValue { DecodedValue::Text((*self).to_string()) }
}

// Бинарные данные: именно Vec<u8> трактуем как Bytes
impl ToDecodedValue for Vec<u8> {
    fn to_decoded_value(&self) -> DecodedValue { DecodedValue::Bytes(self.clone()) }
}

impl ToDecodedValue for Vec<String> {
    fn to_decoded_value(&self) -> DecodedValue {
        DecodedValue::List(self.iter().map(|s| s.to_decoded_value()).collect())
    }
}

// Option<T>
impl<T: ToDecodedValue> ToDecodedValue for Option<T> {
    fn to_decoded_value(&self) -> DecodedValue {
        match self {
            Some(v) => v.to_decoded_value(),
            None => DecodedValue::Null,
        }
    }
}

impl ToDecodedValue for DecodedStruct {
    fn to_decoded_value(&self) -> DecodedValue { DecodedValue::Struct(self.clone()) }
}

fn read_varint_at(data: &[u8], off: &mut usize) -> Option<i32> {
    let mut num_read = 0u32;
    let mut result: i32 = 0;
    loop {
        if *off >= data.len() { return None; }
        let read = data[*off];
        *off += 1;

        let value = (read & 0x7F) as i32;
        result |= value << (7 * num_read);
        num_read += 1;

        if num_read > 5 { return None; }
        if (read & 0x80) == 0 { break; }
    }
    Some(result)
}

/// Читает VarInt, затем имя канала, затем длину payload:
///  - сначала пытается стандартный u16 BE,
///  - если не сходится по общей длине буфера — пробует u24 BE (3 байта).
///
/// Возвращает срез **payload** (без шапки).
fn slice_payload_from_full_custom_payload(full: &[u8]) -> Option<&[u8]> {
    // --- helpers ---
    fn read_varint_at(data: &[u8], off: &mut usize) -> Option<i32> {
        let mut num_read = 0u32;
        let mut result: i32 = 0;
        loop {
            if *off >= data.len() { return None; }
            let read = data[*off];
            *off += 1;

            let value = (read & 0x7F) as i32;
            result |= value << (7 * num_read);
            num_read += 1;

            if num_read > 5 { return None; }
            if (read & 0x80) == 0 { break; }
        }
        Some(result)
    }
    #[inline]
    fn read_u16_be_at(data: &[u8], off: &mut usize) -> Option<usize> {
        if *off + 2 > data.len() { return None; }
        let v = u16::from_be_bytes([data[*off], data[*off + 1]]) as usize;
        *off += 2;
        Some(v)
    }
    #[inline]
    fn read_u24_be_at(data: &[u8], off: &mut usize) -> Option<usize> {
        if *off + 3 > data.len() { return None; }
        let v = ((data[*off] as usize) << 16)
            | ((data[*off + 1] as usize) << 8)
            |  (data[*off + 2] as usize);
        *off += 3;
        Some(v)
    }

    let mut off = 0usize;

    // VarInt длина канала
    let ch_len = read_varint_at(full, &mut off)? as usize;
    if off + ch_len > full.len() { return None; }
    // имя канала нам не нужно для расчётов — просто сдвиг
    off += ch_len;

    // --- стратегия 1: стандартный u16 BE ---
    if let Some(mut off16) = Some(off) {
        if let Some(pay_len) = read_u16_be_at(full, &mut off16) {
            // проверим консистентность: остаток буфера должен равняться pay_len
            if off16 + pay_len <= full.len() && off16 + pay_len == full.len() {
                return Some(&full[off16..off16 + pay_len]);
            }
        }
    }

    // --- стратегия 2: расширенный u24 BE (3 байта) ---
    if let Some(mut off24) = Some(off) {
        if let Some(pay_len) = read_u24_be_at(full, &mut off24) {
            if off24 + pay_len <= full.len() && off24 + pay_len == full.len() {
                return Some(&full[off24..off24 + pay_len]);
            }
        }
    }

    // --- эвристика «если буфер большой, читаем 3-байтовую длину» ---
    // (на случай когда общая длина включает паддинги/хвост, и проверка равенства не сработала)
    if full.len() > 32 * 1024 {
        let mut off_h = off;
        if let Some(pay_len) = read_u24_be_at(full, &mut off_h) {
            if off_h + pay_len <= full.len() {
                return Some(&full[off_h..off_h + pay_len]);
            }
        }
    }

    None
}

/* -------- декодер и реестр -------- */

/// Декодер одного канала кастом-пайлоадов.
/// Теперь принимает `bound`, чтобы различать C/S-пакеты с одинаковым ID.
pub trait CustomPayloadDecoder: Send + Sync + 'static {
    fn channel(&self) -> &'static str;
    /// На вход — **payload** (без шапки), и биндинг (Client/Server).
    fn try_decode(&self, payload: &[u8], bound: Bound) -> Option<DecodedStruct>;
}

// Глобальный реестр декодеров по имени канала
static DECODERS: Lazy<RwLock<HashMap<&'static str, Box<dyn CustomPayloadDecoder>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Зарегистрировать декодер по типу.
pub fn register_decoder<D>()
where
    D: CustomPayloadDecoder + Default + 'static,
{
    let d = D::default();
    DECODERS.write().unwrap().insert(d.channel(), Box::new(d));
}

/// Альтернатива: зарегистрировать уже созданный экземпляр.
pub fn register_decoder_boxed<D>(decoder: D)
where
    D: CustomPayloadDecoder + 'static,
{
    let channel = decoder.channel();
    DECODERS.write().unwrap().insert(channel, Box::new(decoder));
}

/// Декодирование из **полного** буфера CustomPayload (вместе со шапкой).
/// Здесь отрезаем шапку и передаём в декодер уже чистый payload.
pub fn decode_custom_payload(channel: &str, full_buf: &[u8], bound: Bound) -> Option<DecodedStruct> {
    let payload = slice_payload_from_full_custom_payload(full_buf)?;
    let map = DECODERS.read().unwrap();
    map.get(channel).and_then(|d| d.try_decode(payload, bound))
}

pub fn init_default_decoders() {
    register_mod_payload_decoders();
    register_customnpcs_decoder();
}
