use std::io;
use anyhow::{anyhow, Context, Result};
use rand::rngs::OsRng;
use rsa::pkcs8::DecodePublicKey;
use rsa::traits::PublicKeyParts;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;

const AUTH_URL: &str = "l.mcskill.net:7240";

const HANDSHAKE_MAGIC: u32 = 1917264919;
const AUTH_REQUEST_TYPE: i32 = 4;
const CONNECT_TIMEOUT_SECS: u64 = 15;

const PUBLIC_KEY_DER_HEX: &str = "\
30820122300d06092a864886f70d01010105000382010f003082010a0282010100\
d4d4799ad9f255d3cea77aa82067b37232e3298d2a970e93f1a0028ab6ec1f8aa7\
632e752775cf96a963d90c142643b282c085b2e6995329a72246df381197fca77e\
298402c25014b0346c7f644ff2c616f70477a881119a4ec37111f6abdaf5fbe39e\
8421b5106a5553399e5a191fe76b76a040594bb5e0976fe61e3b4d6f134c7b423c\
7ce05de2f7560f3a952e5596f755d8cc6fedd9a2c0a1c986d8c0efaa7eabb5bc7d\
82c8c86c6d4e8c2602deb5ac563a2bbad89cb83b77f26fe4de6513c9dcb1aa5260\
64a18856782c4ff6661a7945416030ace7e02e95dd6a7d933e89b7157834ffd208\
300ba1dd86e21e31ef0899e44d3839d0f2d8192ce0f27cdc5d0203010001";

#[derive(Debug, Clone)]
pub struct AuthData {
    pub access_token: String,
    pub profile: String,
}

pub async fn auth(username: &str, password: &str) -> Result<AuthData> {
    tracing::info!("Begin MCSkill auth for {}", username);

    let encrypted = encrypt_password(password)?;
    tracing::info!("Password encrypted, performing auth...");

    auth_request(username, &encrypted).await
}

async fn auth_request(username: &str, encrypted_password: &[u8]) -> Result<AuthData> {
    tracing::info!("Connecting to auth server at {}", AUTH_URL);

    let mut sock = tokio::time::timeout(
        Duration::from_secs(CONNECT_TIMEOUT_SECS),
        TcpStream::connect(AUTH_URL),
    )
    .await
    .context("Connection timeout")?
    .context("Failed to connect to auth server")?;

    sock.set_nodelay(true)?;

    tracing::info!("Connected, performing handshake...");
    perform_handshake(&mut sock).await.context("Handshake failed")?;
    tracing::info!("Handshake OK, sending login...");

    let login_payload = build_login_payload(username, encrypted_password);
    sock.write_all(&login_payload).await?;
    sock.flush().await?;

    tracing::info!("Login sent, reading response...");
    let len = read_varint(&mut sock).await.context("Failed to read error length")?;

    if len > 0 {
        let mut buf = vec![0u8; len as usize];
        sock.read_exact(&mut buf).await?;
        let raw = String::from_utf8_lossy(&buf);
        let msg = raw
            .strip_prefix("Ошибка: ")
            .or_else(|| raw.strip_prefix("Error: "))
            .unwrap_or(&raw)
            .to_string();
        return Err(anyhow!("Auth error: {}", msg));
    }

    tracing::info!("Reading game profile...");
    let (profile_uuid, _profile_name) = read_game_profile(&mut sock).await.context("Failed to read game profile")?;
    let selected_profile = profile_uuid.to_string().replace('-', "");

    tracing::info!("Reading token...");
    let token = read_token_string(&mut sock).await.context("Failed to read token")?;

    tracing::info!("Auth done: token={}, profile={}", token, selected_profile);

    Ok(AuthData {
        access_token: token,
        profile: selected_profile,
    })
}

async fn perform_handshake(sock: &mut TcpStream) -> Result<()> {
    let pubkey = rsa_pubkey()?;
    let padded = padded_modulus_be(&pubkey);
    let hs = build_handshake(&padded, AUTH_REQUEST_TYPE);

    tracing::info!("Sending handshake: {} bytes", hs.len());
    sock.write_all(&hs).await?;
    sock.flush().await?;

    let mut ok_b = [0u8; 1];
    sock.read_exact(&mut ok_b).await.context("Failed to read handshake response")?;

    tracing::info!("Handshake response: {}", ok_b[0]);
    if ok_b[0] != 1 {
        return Err(anyhow!("Handshake rejected by server (code={})", ok_b[0]));
    }

    Ok(())
}

fn encrypt_password(password: &str) -> Result<Vec<u8>> {
    let pubkey = rsa_pubkey()?;
    let mut rng = OsRng;
    let encrypted = pubkey
        .encrypt(&mut rng, Pkcs1v15Encrypt, password.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;
    Ok(encrypted)
}

fn rsa_pubkey() -> Result<RsaPublicKey> {
    let der = hex::decode(PUBLIC_KEY_DER_HEX)?;
    let pubkey = RsaPublicKey::from_public_key_der(&der)?;
    Ok(pubkey)
}

fn padded_modulus_be(pubkey: &RsaPublicKey) -> Vec<u8> {
    let mut modulus = pubkey.n().to_bytes_be();
    if modulus.first().is_some_and(|b| b & 0x80 != 0) {
        modulus.insert(0, 0);
    }
    let mut padded = vec![0u8; 257 - modulus.len()];
    padded.extend_from_slice(&modulus);
    padded
}

fn build_handshake(padded_modulus: &[u8], request_type: i32) -> Vec<u8> {
    let mut hs = Vec::with_capacity(4 + 5 + padded_modulus.len() + 5);
    hs.extend_from_slice(&HANDSHAKE_MAGIC.to_be_bytes());
    hs.extend_from_slice(&encode_varint(padded_modulus.len() as i32));
    hs.extend_from_slice(padded_modulus);
    hs.extend_from_slice(&encode_varint(request_type));
    hs
}

fn build_login_payload(username: &str, encrypted_password: &[u8]) -> Vec<u8> {
    let mut login = Vec::with_capacity(5 + username.len() + 5 + encrypted_password.len() + 1);
    login.extend_from_slice(&encode_varint(username.len() as i32));
    login.extend_from_slice(username.as_bytes());
    login.extend_from_slice(&encode_varint(encrypted_password.len() as i32));
    login.extend_from_slice(encrypted_password);
    login.push(1);
    login
}

fn encode_varint(mut value: i32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(5);
    loop {
        let mut byte = (value & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
    buf
}

pub async fn read_varint<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<i32> {
    let mut num_read = 0;
    let mut result = 0i32;

    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf).await?;
        let byte = buf[0];
        let value = (byte & 0x7F) as i32;

        if num_read == 5 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt is too big",
            ));
        }

        result |= value << (7 * num_read);
        num_read += 1;

        if (byte & 0x80) == 0 {
            break;
        }
    }

    Ok(result)
}

pub async fn read_i64_be<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<i64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf).await?;
    Ok(i64::from_be_bytes(buf))
}

pub async fn read_uuid<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Uuid> {
    let most_sig = read_i64_be(reader).await? as u64;
    let least_sig = read_i64_be(reader).await? as u64;
    Ok(Uuid::from_u64_pair(most_sig, least_sig))
}

pub async fn recv_exactly<R>(r: &mut R, n: usize) -> io::Result<Vec<u8>>
where
    R: AsyncReadExt + Unpin,
{
    let mut buf = vec![0u8; n];
    r.read_exact(&mut buf).await?;
    Ok(buf)
}


pub async fn read_game_profile<R>(r: &mut R) -> io::Result<(Uuid, String)>
where
    R: AsyncReadExt + Unpin,
{
    let profile_id = read_uuid(r).await?;
    let username = read_string(r, 64).await?;
    if read_bool(r).await? {
        let _skin_url = read_string(r, 2048).await?;
        let _skin_digest = recv_exactly(r, 32).await?;
    }
    if read_bool(r).await? {
        let _cloak_url = read_string(r, 2048).await?;
        let _cloak_digest = recv_exactly(r, 32).await?;
    }
    Ok((profile_id, username))
}

pub async fn read_string<R>(r: &mut R, max_len: usize) -> io::Result<String>
where
    R: AsyncReadExt + Unpin,
{
    let len = read_varint(r).await? as usize;
    if max_len > 0 && len > max_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "string too long",
        ));
    }
    let bytes = recv_exactly(r, len).await?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}


pub async fn read_bool<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<bool> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).await?;
    Ok(buf[0] != 0)
}

async fn read_token_string(sock: &mut TcpStream) -> Result<String> {
    let mut tok_int_bytes = [0u8; 4];
    sock.read_exact(&mut tok_int_bytes).await.context("Failed to read token length bytes")?;
    let tok_int = i32::from_be_bytes(tok_int_bytes);
    let tok_len = tok_int.unsigned_abs() as usize;
    tracing::info!("Token length raw: {}, parsed: {}", tok_int, tok_len);

    if tok_len > 1024 {
        return Err(anyhow!("Invalid token length: {}", tok_len));
    }

    let mut token_bytes = vec![0u8; tok_len];
    sock.read_exact(&mut token_bytes).await.context("Failed to read token bytes")?;
    Ok(String::from_utf8_lossy(&token_bytes).into_owned())
}
