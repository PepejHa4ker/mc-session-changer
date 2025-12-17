use std::io;
use uuid::Uuid;

pub struct ModPacketReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> ModPacketReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn remaining(&self) -> &[u8] {
        &self.data[self.offset..]
    }

    fn ensure_available(&self, need: usize) -> io::Result<()> {
        if self.offset + need > self.data.len() {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Not enough data: need {}, available {}",
                    need,
                    self.data.len().saturating_sub(self.offset)
                ),
            ))
        } else {
            Ok(())
        }
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        self.ensure_available(1)?;
        let v = self.data[self.offset];
        self.offset += 1;
        Ok(v)
    }

    pub fn read_i8(&mut self) -> io::Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_u16_be(&mut self) -> io::Result<u16> {
        self.ensure_available(2)?;
        let v = u16::from_be_bytes([self.data[self.offset], self.data[self.offset + 1]]);
        self.offset += 2;
        Ok(v)
    }

    pub fn read_i16_be(&mut self) -> io::Result<i16> {
        Ok(self.read_u16_be()? as i16)
    }

    pub fn read_u32_be(&mut self) -> io::Result<u32> {
        self.ensure_available(4)?;
        let v = u32::from_be_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ]);
        self.offset += 4;
        Ok(v)
    }

    pub fn read_i32_be(&mut self) -> io::Result<i32> {
        Ok(self.read_u32_be()? as i32)
    }

    pub fn read_u64_be(&mut self) -> io::Result<u64> {
        self.ensure_available(8)?;
        let v = u64::from_be_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
            self.data[self.offset + 4],
            self.data[self.offset + 5],
            self.data[self.offset + 6],
            self.data[self.offset + 7],
        ]);
        self.offset += 8;
        Ok(v)
    }
    pub fn read_f32_be(&mut self) -> io::Result<f32> {
        self.ensure_available(4)?;
        let bytes = [
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ];
        self.offset += 4;
        Ok(f32::from_bits(u32::from_be_bytes(bytes)))
    }

    pub fn read_i64_be(&mut self) -> io::Result<i64> {
        Ok(self.read_u64_be()? as i64)
    }

    pub fn read_varint(&mut self) -> io::Result<i32> {
        let mut num_read = 0u32;
        let mut result: i32 = 0;
        loop {
            let read = self.read_u8()?;
            let value = (read & 0x7F) as i32;
            result |= value << (7 * num_read);
            num_read += 1;
            if num_read > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too long",
                ));
            }
            if (read & 0x80) == 0 {
                break;
            }
        }
        Ok(result)
    }

    pub fn read_string_varint(&mut self) -> io::Result<String> {
        let len = self.read_varint()? as usize;
        self.ensure_available(len)?;
        let bytes = &self.data[self.offset..self.offset + len];
        self.offset += len;
        std::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e))
            })
    }

    pub fn read_uuid(&mut self) -> io::Result<Uuid> {
        let most = self.read_u64_be()?;
        let least = self.read_u64_be()?;
        Ok(Uuid::from_u64_pair(most, least))
    }

    pub fn read<T: ModReadable>(&mut self) -> io::Result<T> {
        T::read_from(self)
    }
}

pub trait ModReadable: Sized {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self>;
}

impl ModReadable for u8 {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_u8()
    }
}
impl ModReadable for i8 {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_i8()
    }
}
impl ModReadable for bool {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_bool()
    }
}
impl ModReadable for u16 {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_u16_be()
    }
}
impl ModReadable for i16 {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_i16_be()
    }
}
impl ModReadable for u32 {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_u32_be()
    }
}
impl ModReadable for i32 {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_i32_be()
    }
}
impl ModReadable for u64 {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_u64_be()
    }
}
impl ModReadable for i64 {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_i64_be()
    }
}
impl ModReadable for f64 {
    fn read_from(reader: &mut ModPacketReader) -> io::Result<Self> {
        let bits = reader.read_i64_be()? as u64;
        Ok(f64::from_bits(bits))
    }
}
impl ModReadable for f32 {
    fn read_from(r: &mut ModPacketReader) -> std::io::Result<Self> {
        r.read_f32_be()
    }
}

impl ModReadable for String {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_string_varint()
    }
}
impl ModReadable for Vec<u8> {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        let len = r.read_u32_be()? as usize;
        r.ensure_available(len)?;
        let bytes = r.data[r.offset..r.offset + len].to_vec();
        r.offset += len;
        Ok(bytes)
    }
}
impl ModReadable for Vec<String> {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        let len = r.read_u32_be()? as usize;
        r.ensure_available(len)?;
        let mut strings = Vec::with_capacity(len);
        for _ in 0..len {
            strings.push(r.read_string_varint()?);
        }
        Ok(strings)
    }
}

impl ModReadable for Uuid {
    fn read_from(r: &mut ModPacketReader) -> io::Result<Self> {
        r.read_uuid()
    }
}