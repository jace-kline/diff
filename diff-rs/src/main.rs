use std::io;
use sha2::{Digest, Sha256};

mod hashing;
mod merkle;

use hashing::*;

#[test]
fn test_hash_file() {
    let path = "/home/jacekline/dev/projects/dif/test.txt";
    let hash = hash_file::<Sha256>(path).expect("Failed to hash file");
    println!("{}", hash_to_hex_string(&hash));
}

#[test]
fn test_hash_file_lines() {
    let path = "/home/jacekline/dev/projects/dif/test.txt";
    let line_hashes = hash_file_lines::<Sha256>(path).expect("Failed to hash file");

    for hash in line_hashes {
        println!("{}", hash_to_hex_string(&hash));
    }
}

#[test]
fn test() {
    let mut hasher = Sha256::new();
    hasher.update(b"hello");
    let hash: [u8; 32] = hasher.finalize().into();
    println!("{:?}", hash_to_hex_string(&hash));
}

fn main() {
    println!("Hello, world!");
}
