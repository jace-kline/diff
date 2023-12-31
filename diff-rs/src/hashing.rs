use std::{fs, io};
use std::io::{Read, Error};
use digest::{Digest, DynDigest, OutputSizeUser, generic_array::GenericArray};
use sha2::Sha256;
use hex;

pub type DigestByteArray<D: Digest> = GenericArray<u8, <D as OutputSizeUser>::OutputSize>;

pub fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    Digest::update(&mut hasher, data);
    hasher.finalize().into()
}

pub fn combine_hashes<D: Digest>(hashes: &[DigestByteArray<D>]) -> DigestByteArray<D> 
{
    let concatted: Vec<u8> = hashes.iter().flat_map(|h| h.into_iter()).map(|b| b.clone()).collect();
    hash::<D>(&concatted)
}

pub fn hash<D: Digest>(data: &[u8]) -> DigestByteArray<D> {
    let mut hasher = D::new();
    hasher.update(data);
    hasher.finalize()
}

pub fn hash_file<D: Digest>(path: &str) -> Result<DigestByteArray<D>, Error>
{
    let mut f = fs::File::open(path)?;

    let mut hasher = D::new();
    let mut buf: Vec<u8> = Vec::new();
    let _filesize = f.read_to_end(&mut buf)?;
    hasher.update(&buf);
    Ok(hasher.finalize())
}

pub fn hash_file_lines<D: Digest>(path: &str) -> Result<Vec<DigestByteArray<D>>, Error>
where <D as OutputSizeUser>::OutputSize: generic_array::ArrayLength
{
    let mut f = fs::File::open(path)?;

    let mut buf = String::new();
    let _filesize = f.read_to_string(&mut buf)?;

    Ok(
        buf
        .lines()
        .map(|s| hash::<D>(s.as_bytes()))
        .collect()
    )
}

pub fn hash_dyn(hasher: &mut dyn DynDigest, data: &[u8]) -> Box<[u8]> {
    hasher.update(data);
    hasher.finalize_reset()
}

pub fn hash_file_dyn(hasher: &mut dyn DynDigest, path: &str) -> Result<Box<[u8]>, Error> {
    let mut f = fs::File::open(path)?;
    let mut buf: Vec<u8> = Vec::new();
    let _filesize = f.read_to_end(&mut buf)?;
    hasher.update(&buf);
    // let _n = io::copy(&mut f, &mut hasher)?;
    Ok(hasher.finalize_reset())
}

pub fn hash_file_lines_dyn(hasher: &mut dyn DynDigest, path: &str) -> Result<Vec<Box<[u8]>>, Error> {
    let mut f = fs::File::open(path)?;
    let mut buf = String::new();
    let _filesize = f.read_to_string(&mut buf)?;

    Ok(
        buf
        .lines()
        .map(|s| hash_dyn(hasher, s.as_bytes()))
        .collect::<Vec<Box<[u8]>>>()
    )
}

pub fn select_hasher(s: &str) -> Box<dyn DynDigest> {
    match s {
        "md5" => Box::new(md5::Md5::default()),
        "sha1" => Box::new(sha1::Sha1::default()),
        "sha224" => Box::new(sha2::Sha224::default()),
        "sha256" => Box::new(sha2::Sha256::default()),
        "sha384" => Box::new(sha2::Sha384::default()),
        "sha512" => Box::new(sha2::Sha512::default()),
        _ => unimplemented!("unsupported digest: {}", s),
    }
}

pub fn hash_to_hex_string(digest: &[u8]) -> String {
    hex::encode(digest)
}

pub fn hex_string_to_hash_vec(s: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(s)
}

pub fn hex_string_to_hash<D: Digest>(s: &str) -> Result<DigestByteArray<D>, hex::FromHexError> {
    let bytes = hex_string_to_hash_vec(s)?;
    let mut arr = DigestByteArray::<D>::default();
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

#[test]
fn test_hex_string_to_hash_vec() {
    let s = "68656c6c6f";
    let bytes = hex_string_to_hash_vec(s).unwrap();
    println!("{:?}", bytes);
}

#[test]
fn test_hash_to_hex_string() {
    let hash = hash::<Sha256>(b"hello");
    println!("{}", hash_to_hex_string(&hash));
}

#[test]
fn test_hex_string_to_hash() {
    let hash = hash::<Sha256>(b"hello");
    let hex_string = hash_to_hex_string(&hash);
    let hash2 = hex_string_to_hash::<Sha256>(&hex_string).unwrap();
    assert_eq!(hash, hash2);
}

#[test]
fn hash_example() {
    let h = hash::<Sha256>(b"hello");
    println!("{}", hash_to_hex_string(&h));
}

#[test]
fn hash_dyn_example() {
    let mut hasher1 = select_hasher("md5");
    let mut hasher2 = select_hasher("sha512");

    // the `&mut *hasher` is to DerefMut the value out of the Box
    // this is equivalent to `DerefMut::deref_mut(&mut hasher)`

    // hasher can be reused due to `finalize_reset()`
    let hash1_1 = hash_dyn(&mut *hasher1, b"foo");
    let hash1_2 = hash_dyn(&mut *hasher1, b"bar");
    let hash2_1 = hash_dyn(&mut *hasher2, b"foo");

    println!("{}", hash_to_hex_string(& *hash1_1));
    println!("{}", hash_to_hex_string(& *hash1_2));
    println!("{}", hash_to_hex_string(& *hash2_1));
}

#[test]
fn combine_hashes_example() {
    let data: Vec<[u8; 4]> = vec![
        [0, 1, 2, 3],
        [0, 2, 4, 6],
        [1, 3, 5, 7]
    ];

    type D = Sha256;

    let hashes: Vec<DigestByteArray<D>> = data.iter().map(|d| hash::<D>(d)).collect();
    let combined = combine_hashes::<Sha256>(&hashes);
    for h in hashes {
        println!("{}", hash_to_hex_string(&h));
    }
    println!("combined: {}", hash_to_hex_string(&combined));
}