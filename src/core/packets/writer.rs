use std::io;
use uuid::Uuid;

pub struct ModPacketWriter {
    data: Vec<u8>,
}

#[allow(dead_code)]
impl ModPacketWriter {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn write_i8(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    pub fn write_bool(&mut self, value: bool) {
        self.data.push(if value { 1 } else { 0 });
    }

    pub fn write_u16_be(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i16_be(&mut self, value: i16) {
        self.data.extend_from_slice(&(value as u16).to_be_bytes());
    }

    pub fn write_u32_be(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i32_be(&mut self, value: i32) {
        self.data.extend_from_slice(&(value as u32).to_be_bytes());
    }

    pub fn write_f32_be(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_bits().to_be_bytes());
    }

    pub fn write_u64_be(&mut self, value: u64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i64_be(&mut self, value: i64) {
        self.data.extend_from_slice(&(value as u64).to_be_bytes());
    }

    pub fn write_varint(&mut self, mut value: i32) {
        loop {
            let mut temp = (value & 0x7F) as u8;
            value >>= 7;
            if (value != 0 && value != -1) && !((value == -1) && (temp & 0x40) != 0) {
                temp |= 0x80;
                self.data.push(temp);
            } else {
                self.data.push(temp);
                break;
            }
        }
    }

    pub fn write_string_be_len(&mut self, value: &str) -> io::Result<()> {
        let bytes = value.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
        Ok(())
    }

    pub fn write_uuid(&mut self, uuid: &Uuid) {
        let (most, least) = uuid.as_u64_pair();
        self.write_u64_be(most);
        self.write_u64_be(least);
    }

    pub fn write<T: ModWritable>(&mut self, value: &T) {
        value.write_to(self);
    }
}

pub trait ModWritable {
    fn write_to(&self, w: &mut ModPacketWriter);
}

impl ModWritable for u8 {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_u8(*self)
    }
}
impl ModWritable for i8 {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_i8(*self)
    }
}
impl ModWritable for bool {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_bool(*self)
    }
}
impl ModWritable for u16 {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_u16_be(*self)
    }
}
impl ModWritable for i16 {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_i16_be(*self)
    }
}
impl ModWritable for u32 {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_u32_be(*self)
    }
}
impl ModWritable for i32 {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_i32_be(*self)
    }
}
impl ModWritable for f32 {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_f32_be(*self);
    }
}
impl ModWritable for u64 {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_u64_be(*self)
    }
}
impl ModWritable for i64 {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_i64_be(*self)
    }
}
impl ModWritable for f64 {
    fn write_to(&self, writer: &mut ModPacketWriter) {
        writer.write_i64_be(self.to_bits() as i64);
    }
}
impl ModWritable for String {
    fn write_to(&self, w: &mut ModPacketWriter) {
        let _ = w.write_string_be_len(self);
    }
}
impl ModWritable for &str {
    fn write_to(&self, w: &mut ModPacketWriter) {
        let _ = w.write_string_be_len(self);
    }
}
impl ModWritable for Vec<u8> {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_u32_be(self.len() as u32);
        w.data.extend_from_slice(self);
    }
}

impl ModWritable for Vec<String> {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_u32_be(self.len() as u32);
        for s in self {
            w.write(s);
        }
    }
}

impl ModWritable for &[u8] {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_u32_be(self.len() as u32);
        w.data.extend_from_slice(self);
    }
}

impl ModWritable for Uuid {
    fn write_to(&self, w: &mut ModPacketWriter) {
        w.write_uuid(self);
    }
}
