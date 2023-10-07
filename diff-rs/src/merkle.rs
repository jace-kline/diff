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
type LineNum = u32;
type MerkleHash = DigestByteArray<Sha256>;

pub trait MerkleNode {
    fn get_hash(&self) -> &MerkleHash;

    fn get_children(&self) -> Vec<&dyn MerkleNode>;

    fn is_leaf(&self) -> bool {
        self.get_children().is_empty()
    }

    fn get_child_hashes(&self) -> Vec<&MerkleHash> {
        self.get_children().iter().map(|c| c.get_hash()).collect()
    }

    fn compute_hash(&self) -> MerkleHash {
        if self.is_leaf() {
            self.get_hash().clone()
        } else {
            let child_hashes = 
                &self.get_child_hashes().iter()
                .map(|&c| c.clone())
                .collect::<Vec<MerkleHash>>();
            combine_hashes::<Sha256>(child_hashes)
        }
    }

    fn compute_hash_recursive(&self) -> MerkleHash {
        if self.is_leaf() {
            self.get_hash().clone()
        } else {
            let child_hashes = 
                &self.get_children().iter()
                .map(|&c| c.compute_hash_recursive())
                .collect::<Vec<MerkleHash>>();
            combine_hashes::<Sha256>(child_hashes)
        }
    }

    fn verify_hash(&self) -> bool {
        self.compute_hash() == *self.get_hash()
    }

    fn verify_hash_recursive(&self) -> bool {
        self.compute_hash_recursive() == *self.get_hash()
    }

    fn verify_hash_recursive_with(&self, hash: &MerkleHash) -> bool {
        self.compute_hash_recursive() == *hash
    }
    
}

#[derive(Debug, PartialEq, Eq)]
enum DiffNodeTextFileLocation {
    Line {
        line_num: LineNum,
        hash: MerkleHash
    },
    LineRange {
        start: LineNum,
        end: LineNum,
        hash: MerkleHash,
        children: Vec<DiffNodeTextFileLocation>
    }
}

impl DiffNodeTextFileLocation {
    fn get_start_line_num(&self) -> LineNum {
        match self {
            DiffNodeTextFileLocation::Line { line_num, .. } => *line_num,
            DiffNodeTextFileLocation::LineRange { start, .. } => *start
        }
    }

    fn get_end_line_num(&self) -> LineNum {
        match self {
            DiffNodeTextFileLocation::Line { line_num, .. } => *line_num,
            DiffNodeTextFileLocation::LineRange { end, .. } => *end
        }
    }

    fn get_hash(&self) -> &MerkleHash {
        match self {
            DiffNodeTextFileLocation::Line { hash, .. } => hash,
            DiffNodeTextFileLocation::LineRange { hash, .. } => hash
        }
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode> {
        match self {
            DiffNodeTextFileLocation::Line { .. } => Vec::new(),
            DiffNodeTextFileLocation::LineRange { children, .. } => children.iter().map(|c| c as &dyn MerkleNode).collect()
        }
    }
}

impl MerkleNode for DiffNodeTextFileLocation {
    fn get_hash(&self) -> &MerkleHash {
        self.get_hash()
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode> {
        self.get_children()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum DiffNodeFsItem {
    Symlink {
        path: PathBuf,
        ref_path: PathBuf,
        hash: MerkleHash,
        size: usize,
        modified: SystemTime
    },
    BinaryFile {
        path: PathBuf,
        hash: MerkleHash,
        size: usize,
        modified: SystemTime
    },
    TextFile {
        path: PathBuf,
        hash: MerkleHash,
        size: usize,
        modified: SystemTime,
        location_node: DiffNodeTextFileLocation
    },
    Dir {
        path: PathBuf,
        hash: MerkleHash,
        children: Vec<DiffNodeFsItem>
    }
}

impl DiffNodeFsItem {
    fn get_hash(&self) -> &MerkleHash {
        match self {
            DiffNodeFsItem::Symlink { hash, .. } => hash,
            DiffNodeFsItem::BinaryFile { hash, .. } => hash,
            DiffNodeFsItem::TextFile { hash, .. } => hash,
            DiffNodeFsItem::Dir { hash, .. } => hash
        }
    }

    fn get_path(&self) -> &PathBuf {
        match self {
            DiffNodeFsItem::Symlink { path, .. } => path,
            DiffNodeFsItem::BinaryFile { path, .. } => path,
            DiffNodeFsItem::TextFile { path, .. } => path,
            DiffNodeFsItem::Dir { path, .. } => path
        }
    }

    fn get_modified(&self) -> &SystemTime {
        match self {
            DiffNodeFsItem::Symlink { modified, .. } => modified,
            DiffNodeFsItem::BinaryFile { modified, .. } => modified,
            DiffNodeFsItem::TextFile { modified, .. } => modified,
            DiffNodeFsItem::Dir { children, .. } => {
                children.iter().fold(&SystemTime::UNIX_EPOCH, |acc, child| {
                    let child_modified = child.get_modified();
                    if child_modified > acc {
                        child_modified
                    } else {
                        acc
                    }
                })
            }
        }
    }

    fn get_size(&self) -> usize {
        match self {
            DiffNodeFsItem::Symlink { size, .. } => *size,
            DiffNodeFsItem::BinaryFile { size, .. } => *size,
            DiffNodeFsItem::TextFile { size, .. } => *size,
            DiffNodeFsItem::Dir { children, .. } => {
                children.iter().fold(0, |acc, child| {
                    acc + child.get_size()
                })
            }
        }
    }
}

impl MerkleNode for DiffNodeFsItem {
    fn get_hash(&self) -> &MerkleHash {
        self.get_hash()
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode> {
        match self {
            DiffNodeFsItem::Symlink { .. } => Vec::new(),
            DiffNodeFsItem::BinaryFile { .. } => Vec::new(),
            DiffNodeFsItem::TextFile { location_node, .. } => vec![location_node as &dyn MerkleNode],
            DiffNodeFsItem::Dir { children, .. } => children.iter().map(|c| c as &dyn MerkleNode).collect()
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum DiffNode {
    FsItem(DiffNodeFsItem),
    TextFileLineLocation(DiffNodeTextFileLocation)
}

impl DiffNode {
    fn get_hash(&self) -> &MerkleHash {
        match self {
            DiffNode::FsItem(fs_item) => fs_item.get_hash(),
            DiffNode::TextFileLineLocation(text_file_line_location) => text_file_line_location.get_hash()
        }
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode> {
        match self {
            DiffNode::FsItem(fs_item) => fs_item.get_children(),
            DiffNode::TextFileLineLocation(text_file_line_location) => text_file_line_location.get_children()
        }
    }
}

impl MerkleNode for DiffNode {
    fn get_hash(&self) -> &MerkleHash {
        self.get_hash()
    }

    fn get_children(&self) -> Vec<&dyn MerkleNode> {
        self.get_children()
    }
}