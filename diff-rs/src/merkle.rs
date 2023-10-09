use std::{fmt::Display, path::PathBuf, time::SystemTime, error::Error};
use digest::{Digest, generic_array::GenericArray};
use sha2::Sha256;
use crate::hashing::*;

#[derive(Debug)]
pub struct MerkleError {
    msg: String
}

impl MerkleError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string()
        }
    }
}

impl Display for MerkleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.msg)
    }
}

impl Error for MerkleError {}

pub trait MerkleNode<D: Digest> {
    fn get_hash(&self) -> DigestByteArray<D>;

    fn get_children(&self) -> Vec<&dyn MerkleNode<D>>;

    fn is_leaf(&self) -> bool {
        self.get_children().is_empty()
    }

    fn get_child_hashes(&self) -> Vec<DigestByteArray<D>> {
        self.get_children().iter().map(|c| c.get_hash()).collect()
    }
    
}