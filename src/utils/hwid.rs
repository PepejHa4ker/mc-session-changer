use base64::{engine::general_purpose, Engine as _};
use block_padding::Pkcs7;
use cipher::{BlockEncryptMut, KeyInit};
use des::Des;
use ecb::Encryptor;
use rand::distributions::Alphanumeric;
use rand::prelude::SliceRandom;
use rand::{random, Rng};
use std::io;
use std::io::Write;

type DesEcbEnc = Encryptor<Des>;
const VENDORS: &[(&str, u16)] = &[("NVIDIA", 0x10DE), ("AMD", 0x1002)];

const NVIDIA_MODELS: &[(&str, u32)] = &[
    ("GeForce GTX 1660 SUPER", 6),
    ("GeForce RTX 3060", 12),
    ("GeForce RTX 4070 Ti", 12),
    ("GeForce RTX 4090", 24),
];

const AMD_MODELS: &[(&str, u32)] = &[
    ("Radeon RX 6600", 8),
    ("Radeon RX 6750 XT", 12),
    ("Radeon RX 7900 XTX", 24),
];

const VENDORS_MODELS: &[(&str, &[&str])] = &[
    ("Samsung", &["980 PRO", "970 EVO Plus", "870 QVO"]),
    ("WD", &["Blue SN570", "Black SN850X", "Red Plus"]),
    (
        "Seagate",
        &["FireCuda 530", "Barracuda Compute", "IronWolf Pro"],
    ),
    ("Kingston", &["KC3000", "A400", "FURY Renegade"]),
    ("HP", &["FX900 Plus", "EX900 PRO", "S750"]),
];

const OS_NAMES: &[&str] = &["Windows 10", "Windows 11"];

const SIZES_GB: &[u32] = &[120, 240, 256, 480, 512, 1000, 1024, 2000, 4096];

const SALT_OS: &[u8] = &[
    0x58, 0x70, 0x39, 0x00, 0x4C, 0x6B, 0x09, 0x32, 0x51, 0x02, 0x46, 0x6D, 0x35, 0x73,
];
const SALT_HOSTNAME: &[u8] = &[
    0x48, 0x6A, 0x36, 0x79, 0x00, 0x4B, 0x70, 0x09, 0x33, 0x57, 0x02, 0x44, 0x66, 0x31, 0x76,
];
const SALT_DISCORD: &[u8] = &[
    0x52, 0x74, 0x34, 0x6D, 0x00, 0x5A, 0x78, 0x09, 0x38, 0x4E, 0x02, 0x56, 0x62, 0x39, 0x4C,
];
const SALT_MAC: &[u8] = &[
    0x51, 0x77, 0x37, 0x73, 0x00, 0x44, 0x66, 0x09, 0x33, 0x47, 0x02, 0x35, 0x4A, 0x6B, 0x39,
];
const SALT_CPU: &[u8] = &[
    0x4D, 0x6E, 0x32, 0x62, 0x00, 0x56, 0x63, 0x09, 0x38, 0x58, 0x02, 0x7A, 0x31, 0x4C, 0x6B,
];
const MOTHERBOARD_SALT: &[u8] = &[
    0x42, 0x76, 0x35, 0x6E, 0x00, 0x4D, 0x64, 0x09, 0x37, 0x46, 0x02, 0x6B, 0x32, 0x4A, 0x68,
];
const GPU_SALT: &[u8] = &[
    0x5A, 0x78, 0x34, 0x63, 0x00, 0x56, 0x62, 0x09, 0x39, 0x4E, 0x02, 0x6D, 0x37, 0x4B, 0x6A,
];
const DISKS_SALT: &[u8] = &[
    0x54, 0x71, 0x33, 0x77, 0x00, 0x46, 0x67, 0x09, 0x38, 0x48, 0x02, 0x6A, 0x32, 0x4B, 0x70,
];

fn java_default_bytes(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    for ch in s.chars() {
        let code = ch as u32;
        if code <= 0x7F {
            out.push(code as u8);
        } else {
            out.push(b'?');
        }
    }
    out
}

fn adjust_des_key_odd_parity(key: &mut [u8; 8]) {
    for b in key.iter_mut() {
        let seven = *b & 0xFE;
        let ones = seven.count_ones();
        let lsb = if ones % 2 == 0 { 1 } else { 0 };
        *b = seven | lsb;
    }
}

fn des_ecb_pkcs5_encrypt_bytes(salt_bytes: &[u8], text: &str, key: &str) -> String {
    let mut data = Vec::with_capacity(salt_bytes.len() + text.len());
    data.extend_from_slice(salt_bytes);
    data.extend_from_slice(&java_default_bytes(text));
    //   data.extend_from_slice(text.as_bytes());
    let key_bytes = java_default_bytes(key);
    let mut k = [0u8; 8];
    k.copy_from_slice(&key_bytes[..8]);
    adjust_des_key_odd_parity(&mut k);

    let cipher = DesEcbEnc::new(&k.into());
    let ct = cipher.encrypt_padded_vec_mut::<Pkcs7>(&data);
    general_purpose::STANDARD.encode(ct)
}

fn random_ascii(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn random_mac() -> String {
    let mut bytes = [0u8; 6];
    rand::thread_rng().fill(&mut bytes);
    bytes[0] = (bytes[0] & 0xFC) | 0x02;

    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join("-")
}

fn random_serial() -> String {
    let mut rng = rand::thread_rng();
    let groups: Vec<String> = (0..8)
        .map(|_| format!("{:04X}", rng.r#gen::<u16>()))
        .collect();
    groups.join("_")
}

fn random_cpu() -> String {
    const BRANDS: &[&str] = &["AuthenticAMD", "Intel"];
    let mut rng = rand::thread_rng();

    let brand = *BRANDS.choose(&mut rng).unwrap();
    let family = rng.gen_range(5..=30);
    let model = rng.gen_range(0..=127);
    let stepping = rng.gen_range(0..=15);
    let cores = rng.gen_range(1..=64);

    let flags: u64 = random();
    let flags_hex = format!("{:016X}", flags);

    format!("{brand} Family {family} Model {model} Stepping {stepping}|{cores}|{flags_hex}")
}
fn random_motherboard() -> String {
    let raw: [u8; 16] = random();
    format!(
        "{:02X}{:02X}{:02X}{:02X}-\
         {:02X}{:02X}-\
         {:02X}{:02X}-\
         {:02X}{:02X}-\
         {:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        raw[0],
        raw[1],
        raw[2],
        raw[3],
        raw[4],
        raw[5],
        raw[6] & 0x0F | 0x40,
        raw[7],
        raw[8] & 0x3F | 0x80,
        raw[9],
        raw[10],
        raw[11],
        raw[12],
        raw[13],
        raw[14],
        raw[15],
    )
}

fn random_gpu() -> String {
    let mut rng = rand::thread_rng();

    let (vendor_name, vendor_id) = *VENDORS.choose(&mut rng).unwrap();

    let (model, vram_gib) = match vendor_name {
        "NVIDIA" => *NVIDIA_MODELS.choose(&mut rng).unwrap(),
        "AMD" => *AMD_MODELS.choose(&mut rng).unwrap(),
        _ => unreachable!(),
    };

    let vram_bytes: u64 = (vram_gib as u64) * 1024 * 1024 * 1024;

    let device_id: u16 = rng.r#gen();

    format!(
        "{vendor} {model}|{vram_bytes}|{vendor} (0x{vendor_id:04x})|0x{device_id:04x}",
        vendor = vendor_name,
        model = model,
        vram_bytes = vram_bytes,
        vendor_id = vendor_id,
        device_id = device_id,
    )
}

fn random_disk() -> String {
    let mut rng = rand::thread_rng();

    let (vendor, models) = VENDORS_MODELS.choose(&mut rng).unwrap();
    let model = models.choose(&mut rng).unwrap();

    let size_gb = SIZES_GB.choose(&mut rng).unwrap();
    let size_str = if *size_gb >= 1000 {
        format!("{}TB", size_gb / 1000)
    } else {
        format!("{}GB", size_gb)
    };

    let serial = random_serial();

    format!(
        "{vendor} {model} {size} (Стандартные дисковые накопители)|{serial}.",
        vendor = vendor,
        model = model,
        size = size_str,
        serial = serial
    )
}

fn os() -> String {
    let os = OS_NAMES.choose(&mut rand::thread_rng()).unwrap();
    des_ecb_pkcs5_encrypt_bytes(SALT_OS, os, "XXsw80PBYVbGqomo")
}

fn hostname() -> String {
    let len = rand::thread_rng().gen_range(8..=15);
    des_ecb_pkcs5_encrypt_bytes(SALT_HOSTNAME, &*random_ascii(len), "VBltRpAHTtM543rK")
}

fn discord() -> String {
    des_ecb_pkcs5_encrypt_bytes(SALT_DISCORD, "null", "FtFK4WgZgpTLPmoT")
}

fn mac() -> String {
    des_ecb_pkcs5_encrypt_bytes(SALT_MAC, &*random_mac(), "R3q987dVLf9menoG")
}

fn cpu() -> String {
    des_ecb_pkcs5_encrypt_bytes(SALT_CPU, &*random_cpu(), "TR2AWtto6U3IzFVw")
}

fn motherboard() -> String {
    des_ecb_pkcs5_encrypt_bytes(MOTHERBOARD_SALT, &*random_motherboard(), "PNNf0rynrHnqNQgQ")
}

fn gpu() -> String {
    des_ecb_pkcs5_encrypt_bytes(GPU_SALT, &*random_gpu(), "HOMPqZeF8XAF48p9")
}

fn disks() -> String {
    des_ecb_pkcs5_encrypt_bytes(DISKS_SALT, &*random_disk(), "fLzm8wj5Pb5wHR3F")
}

pub fn generate_hwid() -> [String; 8] {
    [
        os(),
        hostname(),
        discord(),
        mac(),
        cpu(),
        motherboard(),
        gpu(),
        disks(),
    ]
}

