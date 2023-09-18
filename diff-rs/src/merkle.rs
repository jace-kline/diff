use std::{path::PathBuf, time::SystemTime};
use digest::{Digest, generic_array::GenericArray};
use crate::hashing;

// D: The hash type
// T: The context to attach to this node
pub struct MerkleNode<D, T>
where D: Digest
{
    hash: hashing::DigestByteArray<D>,
    children: Option<Vec<MerkleNode<D, T>>>,
    ctx: T
}

impl<D, T> MerkleNode<D, T>
where D: Digest
{
    // pub fn compute_derived_hash<D: Digest>(&self, children: &[HashType]) -> HashType {
    //     hashing::hash()
    // }

    pub fn new_leaf(&self, hash: hashing::DigestByteArray<D>, ctx: T) -> Self {
        Self {
            hash,
            children: None,
            ctx
        }
    }

    pub fn new_derived(&self, children: Vec<MerkleNode<D, T>>, ctx: T) -> Self {
        let child_hashes: Vec<hashing::DigestByteArray<D>> = children.iter().map(|c| c.hash.clone()).collect();
        let hash = hashing::combine_hashes::<D>(&child_hashes);
        Self {
            hash,
            children: Some(children),
            ctx
        }
    }
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


