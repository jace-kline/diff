use std::collections::HashMap;
use std::fmt::Write;
use std::io::{Read, Write as IoWrite};
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::{fmt::Display, path::PathBuf, time::SystemTime, error::Error};
use digest::{Digest, generic_array::GenericArray};
use sha2::Sha256;

use crate::hashing::*;
use crate::merkle::*;
use crate::util::*;

pub type VcHasher = Sha256;
pub type VcHash = DigestByteArray<VcHasher>;
pub type VcHashString = String;
pub type Name = String;

pub trait VcHashId {
    fn get_hash_bytes(&self) -> VcHash;
    fn get_hash_str(&self) -> VcHashString;
}

#[derive(Debug)]
pub struct Blob {
    pub data: Box<[u8]>,
    pub hash: VcHash
}

#[derive(Debug)]
pub struct Tree {
    pub listings: HashMap<Name, FsObject>,
    pub hash: VcHash
}

#[derive(Debug)]
pub struct Commit<'a> {
    pub tree: Tree,
    pub hash: VcHash,
    pub parent: Option<&'a Commit<'a>>,
    pub author: String,
    pub message: String,
    pub timestamp: SystemTime
}

#[derive(Debug)]
pub enum FsObject {
    Blob(Blob),
    Tree(Tree)
}

#[derive(Debug)]
pub enum VcObject<'a> {
    FsObject(FsObject),
    Commit(Commit<'a>)
}

#[derive(Debug)]
pub enum HeadRef<'a> {
    Tag(Name),
    Head(Name),
    Commit(&'a Commit<'a>)
}

#[derive(Debug)]
pub struct Index<'a> {
    pub root_path: &'a Path,
    pub tags: HashMap<&'a str, &'a Commit<'a>>,
    pub heads: HashMap<&'a str, &'a Commit<'a>>,
    pub objects: HashMap<VcHash, VcObject<'a>>,
    pub head: HeadRef<'a>
}

impl Blob {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.into(),
            hash: hash::<VcHasher>(data)
        }
    }

    pub fn new_owned(data: Box<[u8]>) -> Self {
        let hash = hash::<VcHasher>(&data);
        Self {
            data,
            hash
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_data_as_string(&self) -> String {
        String::from_utf8_lossy(&self.get_data()).to_string()
    }

    pub fn get_data_as_lines(&self) -> Vec<String> {
        self.get_data_as_string().lines().map(|s| s.to_string()).collect()
    }

    pub fn from_file<P>(path: P) -> Result<Self, std::io::Error>
    where P: AsRef<Path>
    {
        let mut f = File::open(path)?;
        let mut buf: Vec<u8> = Vec::new();
        let _filesize = f.read_to_end(&mut buf)?;
        Ok(Self::new(&buf))
    }

    pub fn to_file<P>(&self, parent_path: P) -> Result<(), std::io::Error>
    where P: AsRef<Path>
    {
        let path = PathBuf::from(parent_path.as_ref()).join(hash_to_hex_string(&self.get_hash()));
        if Path::exists(&path) { // assume if file exists with same hash name, it contains the same data
            return Ok(())
        }
        let mut f = File::create(path)?;
        f.write_all(&self.get_data())?;
        Ok(())
    }
}


impl MerkleNode<VcHasher> for Blob {
    fn get_hash(&self) -> VcHash {
        hash::<VcHasher>(&self.data)
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode<VcHasher>> {
        Vec::new()
    }
}

impl VcHashId for Blob {
    fn get_hash_bytes(&self) -> VcHash {
        self.hash.clone()
    }

    fn get_hash_str(&self) -> VcHashString {
        hash_to_hex_string(&self.hash)
    }
}

#[test]
fn test_blob_to_file_file_saved() {
    use tempfile;

    let mut tempdir = tempfile::tempdir().expect("Failed to create temp dir");
    let parent_path = tempdir.path().to_str().expect("Failed to convert temp dir path to string");
    let blob = Blob::new(b"hello world");
    blob.to_file(parent_path).expect("Failed to write blob to file");
    let path = PathBuf::from(parent_path).join(hash_to_hex_string(&blob.get_hash()));
    assert!(Path::exists(path.as_path()));
}

#[test]
fn test_blob_to_file_from_file() {
    use tempfile;

    let mut tempdir = tempfile::tempdir().expect("Failed to create temp dir");
    let parent_path = tempdir.path().to_str().expect("Failed to convert temp dir path to string");
    let blob = Blob::new(b"hello world");
    blob.to_file(parent_path).expect("Failed to write blob to file");
    let blob2 = Blob::from_file(PathBuf::from(parent_path).join(hash_to_hex_string(&blob.get_hash()))).expect("Failed to read blob from file");
    assert_eq!(blob.get_hash(), blob2.get_hash());
    assert_eq!(blob.get_data(), blob2.get_data());
}

impl Tree {
    pub fn new(listings: HashMap<Name, FsObject>) -> Self {
        let child_hashes = listings.values().map(|c| c.get_hash()).collect::<Vec<VcHash>>();
        let hash = combine_hashes::<VcHasher>(&child_hashes);
        Self {
            listings,
            hash
        }
    }

    pub fn to_file<P>(&self, parent_path: P) -> Result<(), Box<dyn Error>>
    where P: AsRef<Path>
    {
        let path = parent_path.as_ref().join(hash_to_hex_string(&self.get_hash()));
        if Path::exists(&path) { // assume if file exists with same hash name, it contains the same data
            return Ok(())
        }
        todo!();
    }

    pub fn from_file<P>(path: P) -> Result<Self, Box<dyn Error>>
    where P: AsRef<Path>
    {
        todo!();
    }
}

impl MerkleNode<VcHasher> for Tree {
    fn get_hash(&self) -> VcHash {
        self.hash.clone()
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode<VcHasher>> {
        self.listings.values().map(|c| c as &dyn MerkleNode<VcHasher>).collect()
    }
}

impl VcHashId for Tree {
    fn get_hash_bytes(&self) -> VcHash {
        self.hash.clone()
    }

    fn get_hash_str(&self) -> VcHashString {
        hash_to_hex_string(&self.hash)
    }
}

impl<'a> Commit<'a> {
    pub fn new(tree: Tree, parent: Option<&'a Commit<'a>>, author: String, message: String) -> Self {
        let timestamp = SystemTime::now();
        let mut hasher = VcHasher::new();
        hasher.update(&tree.get_hash());
        hasher.update(&parent.as_ref().map(|p| p.get_hash()).unwrap_or([0; 32].into()));
        hasher.update(&author.as_bytes());
        hasher.update(&message.as_bytes());
        hasher.update(&timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_le_bytes());
        let hash = hasher.finalize().into();

        Self {
            tree,
            hash,
            parent,
            author,
            message,
            timestamp
        }
    }
}

impl<'a> MerkleNode<VcHasher> for Commit<'a> {
    fn get_hash(&self) -> VcHash {
        self.hash.clone()
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode<VcHasher>> {
        vec![&self.tree]
    }
}

impl VcHashId for Commit<'_> {
    fn get_hash_bytes(&self) -> VcHash {
        self.hash.clone()
    }

    fn get_hash_str(&self) -> VcHashString {
        hash_to_hex_string(&self.hash)
    }
}

impl FsObject {
    pub fn from_file<P>(path: P) -> Result<Self, Box<dyn Error>>
    where P: AsRef<Path>
    {
        let err = std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to parse object file");
        let path = path.as_ref();
        if !Path::exists(path) {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "File does not exist")));
        }
        Blob::from_file(path).map(|b| Self::Blob(b))
            .or_else(|_| Tree::from_file(path).map(|t| Self::Tree(t)))
    }

    pub fn to_file<P>(&self, parent_path: P) -> Result<(), Box<dyn Error>>
    where P: AsRef<Path>
    {
        match self {
            FsObject::Blob(b) => {
                b.to_file(parent_path)?;
            },
            FsObject::Tree(t) => {
                t.to_file(parent_path)?;
            }
        }
        Ok(())
    }
}

impl MerkleNode<VcHasher> for FsObject {
    fn get_hash(&self) -> VcHash {
        match self {
            FsObject::Blob(b) => b.get_hash(),
            FsObject::Tree(t) => t.get_hash()
        }
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode<VcHasher>> {
        match self {
            FsObject::Blob(b) => b.get_children(),
            FsObject::Tree(t) => t.get_children()
        }
    }
}

impl VcHashId for FsObject {
    fn get_hash_bytes(&self) -> VcHash {
        match self {
            FsObject::Blob(b) => b.get_hash(),
            FsObject::Tree(t) => t.get_hash()
        }
    }

    fn get_hash_str(&self) -> VcHashString {
        hash_to_hex_string(&self.get_hash_bytes())
    }
}

impl<'a> MerkleNode<VcHasher> for VcObject<'a> {
    fn get_hash(&self) -> VcHash {
        match self {
            VcObject::FsObject(f) => f.get_hash(),
            VcObject::Commit(c) => c.get_hash()
        }
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode<VcHasher>> {
        match self {
            VcObject::FsObject(f) => f.get_children(),
            VcObject::Commit(c) => c.get_children()
        }
    }
}

impl VcHashId for VcObject<'_> {
    fn get_hash_bytes(&self) -> VcHash {
        match self {
            VcObject::FsObject(f) => f.get_hash(),
            VcObject::Commit(c) => c.get_hash()
        }
    }

    fn get_hash_str(&self) -> VcHashString {
        hash_to_hex_string(&self.get_hash_bytes())
    }
}

