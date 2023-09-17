use std::{path::PathBuf, time::SystemTime};
use digest::Digest;
use super::hashing;

type HashType = u64;

pub struct MerkleNode<T> {
    hash: HashType,
    children: Option<Vec<MerkleNode<T>>>,
    ctx: T
}

impl<T> MerkleNode<T> {
    // pub fn compute_derived_hash<D: Digest>(&self, children: &[HashType]) -> HashType {
    //     hashing::hash()
    // }

    pub fn new_leaf(&self, hash: HashType, ctx: T) -> Self {
        Self {
            hash,
            children: None,
            ctx
        }
    }

    // pub fn new_derived<D: Digest>(&self, children: Vec<MerkleNode<T>>, ctx: T) -> Self {
    //     Self {
    //         hash: 
    //     }
    // }
}

enum FsItemType {
    File,
    Dir,
    Symlink
}

enum DiffMerkleCtx {
    FileLines(u32, u32),
    FsItem {
        path: PathBuf,
        item_type: FsItemType,
        modified: SystemTime,
        len: usize
    }
}


