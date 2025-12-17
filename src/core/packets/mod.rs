#[allow(dead_code)]
#[allow(unused_variables)]

pub mod dwcity;
pub mod reader;
pub mod writer;
pub mod customnpcs;
mod dwquests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bound {
    Client,
    Server,
}

#[macro_export]
macro_rules! bound_from_ident {
    (C) => { $crate::core::packets::Bound::Client };
    (S) => { $crate::core::packets::Bound::Server };
}

#[macro_export]
macro_rules! mod_packets {
    (
        channel: $channel_name:ident,
        packets {
            $(
                $packet_name:ident($packet_id:expr, $bound:ident) {
                    $( $field_name:ident : $field_type:ty ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        pub mod $channel_name {
            use std::io;
            use $crate::core::packets::Bound;
            use $crate::core::packets::reader::ModPacketReader;
            use $crate::core::packets::writer::ModPacketWriter;

            // Для UI-декодера
            use $crate::core::custom_payload::{DecodedStruct, DecodedField, ToDecodedValue};

            /// Имя сетевого канала (реально используется в генераторе декодеров)
            pub const CHANNEL: &str = stringify!($channel_name);

            /// Перечень пакетов этого канала (может не использоваться напрямую)
            #[allow(dead_code)]
            #[derive(Debug, Clone)]
            pub enum Packet {
                $( $packet_name($packet_name), )*
                Unknown(i32, Vec<u8>),
            }

            $(
                #[derive(Debug, Clone)]
                pub struct $packet_name { $( pub $field_name: $field_type, )* }

                impl $packet_name {
                    /// VarInt-дискриминатор внутри payload
                    pub const PACKET_ID: i32 = $packet_id as i32;
                    /// Для какого направления определён этот тип
                    pub const PACKET_BOUND: Bound = $crate::bound_from_ident!($bound);

                    pub fn parse_from_reader(reader: &mut ModPacketReader) -> io::Result<Self> {
                        Ok(Self { $( $field_name: reader.read::<$field_type>()?, )* })
                    }
                }

                // encode генерим только для Client-bound (C)
                $crate::mod_packets!(@gen_encode_impl $packet_name, [$($field_name : $field_type),*], $bound);
            )*

            /// Парсер payload -> enum Packet (может не использоваться в проекте)
            #[allow(dead_code)]
            pub fn parse_packet(payload: &[u8]) -> io::Result<Packet> {
                if payload.is_empty() {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty packet payload"));
                }
                let mut reader = ModPacketReader::new(payload);
                let packet_id = reader.read_varint()?;
                match packet_id {
                    $(
                        id if id == <$packet_name>::PACKET_ID
                           && <$packet_name>::PACKET_BOUND == Bound::Server =>
                        {
                            let packet = $packet_name::parse_from_reader(&mut reader)?;
                            Ok(Packet::$packet_name(packet))
                        }
                    )*
                    _ => {
                        let rest = reader.remaining().to_vec();
                        Ok(Packet::Unknown(packet_id, rest))
                    }
                }
            }

            /// Компактный декодер для UI (payload уже без шапки CustomPayload)
            pub fn try_decode(payload: &[u8], bound: Bound) -> Option<DecodedStruct> {
                let mut reader = ModPacketReader::new(payload);
                let packet_id = reader.read_varint().ok()?;
                match packet_id {
                    $(
                        id if id == <$packet_name>::PACKET_ID
                           && bound == <$packet_name>::PACKET_BOUND =>
                        {
                            let pkt = $packet_name::parse_from_reader(&mut reader).ok()?;
                            let fields = vec![
                                $( DecodedField {
                                    name: stringify!($field_name).to_string(),
                                    value: (&pkt.$field_name).to_decoded_value(),
                                }, )*
                            ];
                            Some(DecodedStruct {
                                name: stringify!($packet_name).to_string(),
                                fields,
                            })
                        }
                    ),*
                    _ => None
                }
            }
        }
    };

    (@gen_encode_impl $packet_name:ident, [$($field_name:ident : $field_ty:ty),*], C) => {
        impl $packet_name {
            #[allow(dead_code)]
            pub fn encode(&self) -> Vec<u8> {
                let mut writer = ModPacketWriter::new();
                writer.write_varint(Self::PACKET_ID);
                $( writer.write(&self.$field_name); )*
                writer.into_bytes()
            }
        }
    };
    (@gen_encode_impl $packet_name:ident, [$($field_name:ident : $field_ty:ty),*], S) => {};
}

#[macro_export]
macro_rules! generate_mod_payload_decoders {
    // форма: module
    ( $( $module:ident ),+ $(,)? ) => {
        $crate::generate_mod_payload_decoders!(@impl_from_modules $( $module ),+ );
    };
    // обратная совместимость: "channel" => module (литерал игнорируем, берём module::CHANNEL)
    ( $( $channel:literal => $module:ident ),+ $(,)? ) => {
        $crate::generate_mod_payload_decoders!(@impl_from_modules $( $module ),+ );
    };

    (@impl_from_modules $( $module:ident ),+ ) => {
        paste::paste! {
            $(
                pub struct [<$module:camel PayloadDecoder>];

                impl Default for [<$module:camel PayloadDecoder>] {
                    fn default() -> Self { Self }
                }

                impl $crate::core::custom_payload::CustomPayloadDecoder for [<$module:camel PayloadDecoder>] {
                    fn channel(&self) -> &'static str { $module::CHANNEL }
                    fn try_decode(
                        &self,
                        payload: &[u8],
                        bound: $crate::core::packets::Bound
                    ) -> Option<$crate::core::custom_payload::DecodedStruct> {
                        $module::try_decode(payload, bound)
                    }
                }
            )*

            pub fn register_mod_payload_decoders() {
                $(
                    $crate::core::custom_payload::register_decoder::<[<$module:camel PayloadDecoder>]>();
                )*
            }
        }
    };
}

