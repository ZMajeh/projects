use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aead::Aead;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use zeroize::Zeroizing;
use js_sys::Uint8Array;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

type HmacSha256 = Hmac<Sha256>;

const PBKDF2_ITERATIONS: u32 = 100_000;

fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32, out_len: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(out_len);
    let mut block: u32 = 1;
    while result.len() < out_len {
        let mut mac = <HmacSha256 as Mac>::new_from_slice(password).unwrap();
        mac.update(salt);
        mac.update(&block.to_be_bytes());
            let d = mac.finalize().into_bytes();
            let mut u = d.as_slice().to_vec();
            let mut t = d.as_slice().to_vec();
            for _ in 1..iterations {
                let mut mac = <HmacSha256 as Mac>::new_from_slice(password).unwrap();
                mac.update(&u);
                u = mac.finalize().into_bytes().as_slice().to_vec();
            for i in 0..t.len() {
                t[i] ^= u[i];
            }
        }
        result.extend_from_slice(&t);
        block += 1;
    }
    result.truncate(out_len);
    result
}

fn derive_key(password: &str, salt: &[u8]) -> Zeroizing<[u8; 32]> {
    let derived = pbkdf2_hmac_sha256(password.as_bytes(), salt, PBKDF2_ITERATIONS, 32);
    Zeroizing::new(derived.try_into().unwrap())
}

pub fn encrypt(plaintext: &str, password: &str) -> Result<String, String> {
    let mut salt = [0u8; 16];
    get_random(&mut salt)?;

    let mut nonce_bytes = [0u8; 12];
    get_random(&mut nonce_bytes)?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&*key)
        .map_err(|e| format!("key setup failed: {e}"))?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("encryption failed: {e}"))?;

    let mut bundle = Vec::new();
    bundle.extend_from_slice(&salt);
    bundle.extend_from_slice(&nonce_bytes);
    bundle.extend_from_slice(&ciphertext);

    Ok(B64.encode(&bundle))
}

pub fn decrypt(b64: &str, password: &str) -> Result<String, String> {
    let bundle = B64.decode(b64).map_err(|e| format!("base64 decode failed: {e}"))?;
    if bundle.len() < 16 + 12 {
        return Err("ciphertext too short".to_string());
    }

    let salt = &bundle[0..16];
    let nonce_bytes = &bundle[16..28];
    let ciphertext = &bundle[28..];

    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new_from_slice(&*key)
        .map_err(|e| format!("key setup failed: {e}"))?;

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|e| format!("decryption failed — wrong password?: {e}"))?;

    String::from_utf8(plaintext).map_err(|e| format!("utf8 error: {e}"))
}

fn get_random(buf: &mut [u8]) -> Result<(), String> {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return Err("no window".to_string()),
    };
    let crypto = window.crypto().map_err(|_| "no crypto".to_string())?;
    let arr = Uint8Array::new_with_length(buf.len() as u32);
    crypto
        .get_random_values_with_array_buffer_view(&arr)
        .map_err(|e| format!("random failed: {e:?}"))?;
    buf.copy_from_slice(&arr.to_vec());
    Ok(())
}
