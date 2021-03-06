use std::io::Write;
use std::io;
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::seq::SliceRandom;

type AesCbc = Cbc<Aes256, Pkcs7>;
use ipfs_api::IpfsClient;
use ipfs_api::TryFromUri;
use http::Uri;
use std::io::Cursor;
use base64;
use iota_client::{Client};
const BASE_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

fn encrypt(filename: &str, password: &str) -> String {
    let buf = std::fs::read(filename).expect("Failed to load the file");
    let enc_data = encrypt_data(password, &buf);
    return enc_data;
}


pub async fn send_to_ipfs(filename: &str, password: &str) -> String {   
    let uri = "https://ipfs.infura.io:5001/api/v0".parse::<Uri>().unwrap();
    let client = IpfsClient::build_with_base_uri(uri);
    let enc_data = encrypt(filename, password);
    return client.add(Cursor::new(enc_data)).await.expect("Failed to add file to IPFS").hash;
}

pub async fn send_ipfs_hash(ipfs_hash: &str, index: &str) {
    println!("start IOTA");
    let iota = Client::builder()
        .with_node("https://chrysalis-nodes.iota.org").expect("IOTA node is unavailable.")
        .finish()
        .await.expect("IOTA node is unavailable.");
    let _ = iota
        .message()
        .with_index(index)
        .with_data(ipfs_hash.as_bytes().to_vec())
        .finish()
        .await.expect("Failed to send message to IOTA.");
    println!("IOTA done");
}

fn gen_ascii_chars(size: usize) -> String {
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        BASE_STR.as_bytes()
            .choose_multiple(&mut rng, size)
            .cloned()
            .collect()
    ).unwrap()
}

fn encrypt_data(key: &str, data: &[u8]) -> String {
    let iv_str = gen_ascii_chars(16);
    let iv = iv_str.as_bytes();
    let cipher = AesCbc::new_var(key.as_bytes(), iv).unwrap();
    let ciphertext = cipher.encrypt_vec(data);
    let mut buffer = bytebuffer::ByteBuffer::from_bytes(iv);
    buffer.write_bytes(&ciphertext);
    base64::encode(buffer.to_bytes())
}

pub fn decrypt(filename: &str, password: &str) {
    let content = std::fs::read_to_string(filename).expect("Failed to load encrypted file.");
    let buf = decrypt_data(password, &content);
    let mut writer = io::BufWriter::new(io::stdout());
    writer.write(&buf).unwrap();
}
fn decrypt_data(key: &str, data: &str) -> Vec<u8> {
    let bytes = base64::decode(data).unwrap();
    let cipher = AesCbc::new_var(key.as_bytes(), &bytes[0..16]).unwrap();
    cipher.decrypt_vec(&bytes[16..]).unwrap()
}
