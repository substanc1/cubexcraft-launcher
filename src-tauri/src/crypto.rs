// AES-256-GCM: формат как у старого лаунчера (JavaFXApplication.encryptCredential): base64(nonce12 | шифр | tag16),
// ключ - 32 случайных байта в файле рядом. Так же читаются remembered.dat/quick.dat старого лаунчера (ключ cxc.key).
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use rand::RngCore;
use std::path::Path;

pub fn load_key(path: &Path) -> Option<[u8; 32]> {
    let b = std::fs::read(path).ok()?;
    b.try_into().ok()
}

pub fn load_or_create_key(path: &Path) -> [u8; 32] {
    if let Some(k) = load_key(path) {
        return k;
    }
    let mut k = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut k);
    if let Some(p) = path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let _ = std::fs::write(path, k);
    k
}

pub fn encrypt(key: &[u8; 32], plain: &[u8]) -> String {
    let cipher = Aes256Gcm::new_from_slice(key).expect("key 32");
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    let ct = cipher.encrypt(Nonce::from_slice(&nonce), plain).expect("aes-gcm");
    let mut out = nonce.to_vec();
    out.extend_from_slice(&ct);
    B64.encode(out)
}

pub fn decrypt(key: &[u8; 32], b64: &str) -> Option<Vec<u8>> {
    let raw = B64.decode(b64.trim()).ok()?;
    if raw.len() < 12 + 16 {
        return None;
    }
    let cipher = Aes256Gcm::new_from_slice(key).ok()?;
    cipher.decrypt(Nonce::from_slice(&raw[..12]), &raw[12..]).ok()
}

pub fn b64_decode_str(s: &str) -> Option<String> {
    String::from_utf8(B64.decode(s.trim()).ok()?).ok()
}
