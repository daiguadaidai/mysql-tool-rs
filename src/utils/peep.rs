use aes::Aes256;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use rand::seq::SliceRandom;
use std::panic;

type AesCbc = Cbc<Aes256, Pkcs7>;

#[allow(dead_code)]
const BASE_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const KEY: &str = "76543210012345670123456776543210";

fn gen_ascii_chars(size: usize) -> String {
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        BASE_STR
            .as_bytes()
            .choose_multiple(&mut rng, size)
            .cloned()
            .collect(),
    )
    .unwrap()
}

#[allow(dead_code)]
pub fn encrypt(key: &str, data: &str) -> String {
    let iv_str = gen_ascii_chars(16);
    let iv = iv_str.as_bytes();
    let cipher = AesCbc::new_from_slices(key.as_bytes(), iv).unwrap();
    let ciphertext = cipher.encrypt_vec(data.as_bytes());
    let mut buffer = bytebuffer::ByteBuffer::from_bytes(iv);
    buffer.write_bytes(&ciphertext);
    base64::encode(buffer.to_bytes())
}

pub fn decrypt(key: &str, data: &str) -> String {
    let bytes = base64::decode(data).unwrap();
    let cipher = AesCbc::new_from_slices(key.as_bytes(), &bytes[0..16]).unwrap();
    String::from_utf8(cipher.decrypt_vec(&bytes[16..]).unwrap()).unwrap()
}

#[allow(dead_code)]
pub fn encrypt_default(data: &str) -> String {
    encrypt(KEY, data)
}

pub fn decrypt_default(data: &str) -> String {
    match panic::catch_unwind(|| decrypt(KEY, data)) {
        Ok(de) => de,
        Err(_) => data.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::peep::{decrypt_default, encrypt_default};

    #[test]
    fn test_encrypt_default() {
        let e = encrypt_default("aaa");
        println!("{}", e)
    }

    #[test]
    fn test_decrypt_default() {
        let data = "dFVEeDRocnNSaTJnV21IS1L09q59TdTLfzoy2IuSoao=";
        let d = decrypt_default(data);
        println!("{}", d)
    }

    #[test]
    fn test_decrypt_default_v2() {
        let data = "aaaa";
        let d = decrypt_default(data);
        println!("{}", d)
    }
}
