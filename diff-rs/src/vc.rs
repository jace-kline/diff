use std::collections::HashMap;
use std::io::Read;
use std::{fmt::Display, path::PathBuf, time::SystemTime, error::Error};
use digest::{Digest, generic_array::GenericArray};
use sha2::Sha256;

use crate::hashing::*;
use crate::merkle::*;

type VcHasher = Sha256;
type VcHash = DigestByteArray<VcHasher>;
type Name = String;

#[derive(Debug)]
pub struct Blob {
    pub data: Box<[u8]>,
    pub hash: VcHash
}

#[derive(Debug)]
pub struct BlobStub {
    pub hash: VcHash
}

#[derive(Debug)]
pub struct Tree {
    pub listings: HashMap<Name, FsObject>,
    pub hash: VcHash
}

#[derive(Debug)]
pub enum FsObject {
    Blob(Blob),
    BlobStub(BlobStub),
    Tree(Tree)
}

#[derive(Debug)]
pub struct Commit {
    pub tree: Tree,
    pub hash: VcHash,
    pub parent: Option<Box<Commit>>,
    pub author: String,
    pub message: String,
    pub timestamp: SystemTime
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

    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let mut f = std::fs::File::open(path)?;
        let mut buf: Vec<u8> = Vec::new();
        let _filesize = f.read_to_end(&mut buf)?;
        Ok(Self::new(&buf))
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
}

impl MerkleNode<VcHasher> for Blob {
    fn get_hash(&self) -> VcHash {
        hash::<VcHasher>(&self.data)
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode<VcHasher>> {
        Vec::new()
    }
}

impl BlobStub {
    pub fn new(hash: VcHash) -> Self {
        Self {
            hash
        }
    }
}

impl MerkleNode<VcHasher> for BlobStub {
    fn get_hash(&self) -> VcHash {
        self.hash.clone()
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode<VcHasher>> {
        Vec::new()
    }
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
}

impl MerkleNode<VcHasher> for Tree {
    fn get_hash(&self) -> VcHash {
        self.hash.clone()
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode<VcHasher>> {
        self.listings.values().map(|c| c as &dyn MerkleNode<VcHasher>).collect()
    }
}

impl MerkleNode<VcHasher> for FsObject {
    fn get_hash(&self) -> VcHash {
        match self {
            FsObject::Blob(b) => b.get_hash(),
            FsObject::BlobStub(b) => b.get_hash(),
            FsObject::Tree(t) => t.get_hash()
        }
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode<VcHasher>> {
        match self {
            FsObject::Blob(b) => b.get_children(),
            FsObject::BlobStub(b) => b.get_children(),
            FsObject::Tree(t) => t.get_children()
        }
    }
}

