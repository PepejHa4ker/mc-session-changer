use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketDirection {
    Inbound,  // server -> client
    Outbound, // client -> server
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PacketDetails {
    CustomPayload {
        channel: String,
        channel_len: usize,
        preview: Option<String>,
    },
}
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PacketRecord {
    pub id: u64,
    pub ts_millis: u64,
    pub dir: PacketDirection,
    pub name: String,
    pub len: usize,
    pub data: Vec<u8>,
    pub details: Option<PacketDetails>,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub color: Option<[u8; 3]>,
    pub group: Option<String>,
}

pub struct PacketStore {
    buf: VecDeque<PacketRecord>,
    current_bytes: usize,
    limits: Limits,
}

#[derive(Clone, Debug)]
pub struct Limits {
    pub max_count: Option<usize>,
    /// Учитываем только размер data (байты полезной нагрузки)
    pub max_bytes: Option<usize>,
    pub autoclear_oldest: bool,
}

impl PacketStore {
    pub fn new(cap: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(cap.min(10_000_000)),
            current_bytes: 0,
            limits: Limits {
                max_count: Some(cap.max(1)),
                max_bytes: None,
                autoclear_oldest: true,
            },
        }
    }

    pub fn push(&mut self, rec: PacketRecord) {
        let added = rec.data.len();
        self.buf.push_back(rec);
        self.current_bytes = self.current_bytes.saturating_add(added);
        self.trim_to_limits();
    }

    pub fn snapshot(&self) -> Vec<PacketRecord> {
        self.buf.iter().cloned().collect()
    }
    pub fn clear(&mut self) {
        self.buf.clear();
        self.current_bytes = 0;
    }

    pub fn set_max_count(&mut self, max: Option<usize>) {
        self.limits.max_count = max;
        self.trim_to_limits();
    }
    pub fn set_autoclear_oldest(&mut self, on: bool) {
        self.limits.autoclear_oldest = on;
    }

    pub fn pin(&mut self, id: u64, pinned: bool) {
        if let Some(r) = self.buf.iter_mut().find(|r| r.id == id) {
            if r.pinned != pinned {
                r.pinned = pinned;
            }
        }
        if self.limits.autoclear_oldest {
            self.trim_to_limits();
        }
    }

    pub fn set_tags(&mut self, id: u64, tags: Vec<String>) {
        if let Some(r) = self.buf.iter_mut().find(|r| r.id == id) {
            r.tags = tags;
        }
    }

    pub fn set_color(&mut self, id: u64, color: Option<[u8; 3]>) {
        if let Some(r) = self.buf.iter_mut().find(|r| r.id == id) {
            r.color = color;
        }
    }

    fn pop_front_and_account(&mut self) -> Option<PacketRecord> {
        let r = self.buf.pop_front()?;
        self.current_bytes = self.current_bytes.saturating_sub(r.data.len());
        Some(r)
    }
    fn trim_to_limits(&mut self) {
        if !self.limits.autoclear_oldest {
            return;
        }
        let over_count = |s: &Self| s.limits.max_count.map_or(false, |m| s.buf.len() > m);
        let over_bytes = |s: &Self| s.limits.max_bytes.map_or(false, |m| s.current_bytes > m);
        while over_count(self) || over_bytes(self) {
            if let Some(pos) = self.buf.iter().position(|r| !r.pinned) {
                let removed = self.buf.remove(pos).unwrap();
                self.current_bytes = self.current_bytes.saturating_sub(removed.data.len());
                continue;
            }
            let _ = self.pop_front_and_account();
        }
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.buf.len(), self.current_bytes)
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn read_varint(input: &[u8]) -> Option<(i32, usize)> {
    let mut num = 0i32;
    let mut shift = 0u32;
    let mut i = 0usize;
    while i < input.len() {
        let b = input[i];
        num |= ((b & 0x7F) as i32) << shift;
        i += 1;
        if (b & 0x80) == 0 {
            return Some((num, i));
        }
        shift += 7;
        if shift > 35 {
            break;
        }
    }
    None
}

pub fn try_decode_custom_payload(bytes: &[u8]) -> Option<(String, usize)> {
    let (strlen, used1) = read_varint(bytes)?;
    if strlen < 0 {
        return None;
    }
    let u = strlen as usize;
    if used1 + u > bytes.len() {
        return None;
    }
    let sbytes = &bytes[used1..used1 + u];
    let channel = std::str::from_utf8(sbytes).ok()?.to_string();
    Some((channel, used1 + u))
}

pub fn make_record(dir: PacketDirection, name: String, data: Vec<u8>) -> PacketRecord {
    let details = if name.contains("CustomPayload") {
        if let Some((channel, channel_len)) = try_decode_custom_payload(&data) {
            let rest = &data.get(channel_len..).unwrap_or(&[]);
            let mut preview: Option<String> = None;
            if !rest.is_empty() {
                if let Ok(s) = std::str::from_utf8(rest) {
                    let s_clean: String = s.chars()
                        .filter(|c| c.is_ascii_graphic() || *c == ' ')
                        .take(64).collect();
                    if !s_clean.is_empty() { preview = Some(s_clean); }
                } else if channel == "FML|HS" {
                    let d = rest[0];
                    preview = Some(format!("FML handshake step {}", d));
                }
            }
            Some(PacketDetails::CustomPayload { channel, channel_len, preview })
        } else { None }
    } else { None };
    let details_clone = details.clone();

    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    PacketRecord {
        id,
        ts_millis: now_millis(),
        dir,
        name: name.clone(),
        len: data.len(),
        data,
        details,
        pinned: false,
        tags: Vec::new(),
        color: None,
        group: details_clone.as_ref().and_then(|d| match d {
            PacketDetails::CustomPayload { channel, .. } => Some(channel.clone()),
        }).or_else(|| Some(name)),
    }
}
