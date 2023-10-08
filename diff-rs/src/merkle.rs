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

    fn compute_hash(&self) -> DigestByteArray<D> {
        if self.is_leaf() {
            self.get_hash().clone()
        } else {
            let child_hashes = 
                &self.get_child_hashes().iter()
                .map(|c| c.clone())
                .collect::<Vec<DigestByteArray<D>>>();
            combine_hashes::<D>(child_hashes)
        }
    }

    fn compute_hash_recursive(&self) -> DigestByteArray<D> {
        if self.is_leaf() {
            self.get_hash().clone()
        } else {
            let child_hashes = 
                &self.get_children().iter()
                .map(|&c| c.compute_hash_recursive())
                .collect::<Vec<DigestByteArray<D>>>();
            combine_hashes::<D>(child_hashes)
        }
    }

    fn verify_hash(&self) -> bool {
        self.compute_hash() == self.get_hash()
    }

    fn verify_hash_recursive(&self) -> bool {
        self.compute_hash_recursive() == self.get_hash()
    }

    fn verify_hash_recursive_with(&self, hash: &DigestByteArray<D>) -> bool {
        self.compute_hash_recursive() == *hash
    }
    
}