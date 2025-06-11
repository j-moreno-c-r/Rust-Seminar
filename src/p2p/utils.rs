
use sha2::{Digest, Sha256};

pub const MAGIC: [u8; 4] = [0xF9, 0xBE, 0xB4, 0xD9];

pub fn sha256d(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    second.into()
}
