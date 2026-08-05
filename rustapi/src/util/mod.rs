pub mod http;
pub mod ip;
pub mod time;
pub mod ua;

pub use http::*;
pub use ip::*;
pub use time::*;
pub use ua::*;

use md5::{Digest, Md5};

pub fn md5_hex(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn rand_password(num: usize) -> (String, String, String) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars_a: &[u8] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{};:,.<>?";
    let chars_b: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let chars_c: &[u8] = b"0123456789abcdef";
    let a: String = (0..num)
        .map(|_| chars_a[rng.gen_range(0..chars_a.len())] as char)
        .collect();
    let b: String = (0..num)
        .map(|_| chars_b[rng.gen_range(0..chars_b.len())] as char)
        .collect();
    let c: String = (0..num)
        .map(|_| chars_c[rng.gen_range(0..chars_c.len())] as char)
        .collect();
    (a, b, c)
}
