// use std::{fmt::Display, path::PathBuf, time::SystemTime, error::Error};
// use digest::{Digest, generic_array::GenericArray};
// use sha2::Sha256;
// use crate::hashing::*;

// use crate::merkle::*;

// pub trait DiffHistory {
//     fn get_ancestor(&self) -> Option<&Self>;
//     fn fold_history(&self) -> &Self;

//     fn is_full_node(&self) -> bool {
//         self.get_ancestor().is_none()
//     }

//     fn is_incremental_node(&self) -> bool {
//         self.get_ancestor().is_some()
//     }

//     fn get_ancestor_n(&self, n: usize) -> Option<&Self> {
//         if n == 0 {
//             Some(self)
//         } else {
//             match self.get_ancestor() {
//                 Some(ancestor) => ancestor.get_ancestor_n(n - 1),
//                 None => None
//             }
//         }
//     }
// }

// #[derive(Debug, PartialEq, Eq)]
// enum DiffNodeTextLines {
//     Line {
//         line_num: LineNum,
//         content: String,
//         hash: MerkleHash
//     },
//     LineRange {
//         start: LineNum,
//         end: LineNum,
//         hash: MerkleHash,
//         children: Vec<DiffNodeTextLines>
//     },
//     NoChange(Box<DiffNodeTextLines>),
//     Added(Box<DiffNodeTextLines>),
//     Modified {
//         old: Box<DiffNodeTextLines>,
//         new: Box<DiffNodeTextLines>
//     }
// }

// impl DiffNodeTextLines {
//     fn get_start_line_num(&self) -> LineNum {
//         match self {
//             DiffNodeTextLines::Line { line_num, .. } => *line_num,
//             DiffNodeTextLines::LineRange { start, .. } => *start,
//             DiffNodeTextLines::NoChange(node) => node.get_start_line_num(),
//             DiffNodeTextLines::Added(node) => node.get_start_line_num(),
//             DiffNodeTextLines::Modified { new, .. } => new.get_start_line_num()
//         }
//     }

//     fn get_end_line_num(&self) -> LineNum {
//         match self {
//             DiffNodeTextLines::Line { line_num, .. } => *line_num,
//             DiffNodeTextLines::LineRange { end, .. } => *end,
//             DiffNodeTextLines::NoChange(node) => node.get_end_line_num(),
//             DiffNodeTextLines::Added(node) => node.get_end_line_num(),
//             DiffNodeTextLines::Modified { new, .. } => new.get_end_line_num()
//         }
//     }

//     fn get_hash(&self) -> &MerkleHash {
//         match self {
//             DiffNodeTextLines::Line { hash, .. } => hash,
//             DiffNodeTextLines::LineRange { hash, .. } => hash,
//             DiffNodeTextLines::NoChange(node) => node.get_hash(),
//             DiffNodeTextLines::Added(node) => node.get_hash(),
//             DiffNodeTextLines::Modified { new, .. } => new.get_hash()
//         }
//     }

//     fn get_children(&self) -> Vec<&dyn MerkleNode> {
//         match self {
//             DiffNodeTextLines::Line { .. } => Vec::new(),
//             DiffNodeTextLines::LineRange { children, .. } => children.iter().map(|c| c as &dyn MerkleNode).collect(),
//             DiffNodeTextLines::NoChange(node) => node.get_children(),
//             DiffNodeTextLines::Added(node) => node.get_children(),
//             DiffNodeTextLines::Modified { new, .. } => new.get_children()
//         }
//     }

//     fn get_ancestor(&self) -> Option<&Self> {
//         match self {
//             DiffNodeTextLines::Line { .. } => None,
//             DiffNodeTextLines::LineRange { .. } => None,
//             DiffNodeTextLines::NoChange(node) => node.get_ancestor(),
//             DiffNodeTextLines::Added(_) => None,
//             DiffNodeTextLines::Modified { old, .. } => Some(old)
//         }
//     }

//     fn fold_history(&self) -> &Self {
//         match self {
//             DiffNodeTextLines::Line { .. } => self,
//             DiffNodeTextLines::LineRange { .. } => self,
//             DiffNodeTextLines::NoChange(node) => node.fold_history(),
//             DiffNodeTextLines::Added(node) => node.fold_history(),
//             DiffNodeTextLines::Modified { new, .. } => new.fold_history()
//         }
//     }
// }

// impl MerkleNode for DiffNodeTextLines {
//     fn get_hash(&self) -> &MerkleHash {
//         self.get_hash()
//     }

//     fn get_children(&self) -> Vec<&dyn MerkleNode> {
//         self.get_children()
//     }
// }

// impl DiffHistory for DiffNodeTextLines {
//     fn get_ancestor(&self) -> Option<&Self> {
//         self.get_ancestor()
//     }

//     fn fold_history(&self) -> &Self {
//         self.fold_history()
//     }
// }

// #[derive(Debug, PartialEq, Eq)]
// enum DiffNodeFsItem {
//     Symlink {
//         path: PathBuf,
//         ref_path: PathBuf,
//         hash: MerkleHash,
//         size: usize,
//         modified: SystemTime
//     },
//     BinaryFile {
//         path: PathBuf,
//         hash: MerkleHash,
//         size: usize,
//         modified: SystemTime
//     },
//     TextFile {
//         path: PathBuf,
//         hash: MerkleHash,
//         size: usize,
//         modified: SystemTime,
//         location_node: DiffNodeTextLines
//     },
//     Dir {
//         path: PathBuf,
//         hash: MerkleHash,
//         children: Vec<DiffNodeFsItem>
//     },
//     NoChange(Box<DiffNodeFsItem>),
//     Added(Box<DiffNodeFsItem>),
//     Modified {
//         old: Box<DiffNodeFsItem>,
//         new: Box<DiffNodeFsItem>
//     }
// }

// impl DiffNodeFsItem {
//     fn get_hash(&self) -> &MerkleHash {
//         match self {
//             DiffNodeFsItem::Symlink { hash, .. } => hash,
//             DiffNodeFsItem::BinaryFile { hash, .. } => hash,
//             DiffNodeFsItem::TextFile { hash, .. } => hash,
//             DiffNodeFsItem::Dir { hash, .. } => hash,
//             DiffNodeFsItem::NoChange(fs_item) => fs_item.get_hash(),
//             DiffNodeFsItem::Added(fs_item) => fs_item.get_hash(),
//             DiffNodeFsItem::Modified { new, .. } => new.get_hash()
//         }
//     }

//     fn get_path(&self) -> &PathBuf {
//         match self {
//             DiffNodeFsItem::Symlink { path, .. } => path,
//             DiffNodeFsItem::BinaryFile { path, .. } => path,
//             DiffNodeFsItem::TextFile { path, .. } => path,
//             DiffNodeFsItem::Dir { path, .. } => path,
//             DiffNodeFsItem::NoChange(fs_item) => fs_item.get_path(),
//             DiffNodeFsItem::Added(fs_item) => fs_item.get_path(),
//             DiffNodeFsItem::Modified { new, .. } => new.get_path()
//         }
//     }

//     fn get_modified(&self) -> &SystemTime {
//         match self {
//             DiffNodeFsItem::Symlink { modified, .. } => modified,
//             DiffNodeFsItem::BinaryFile { modified, .. } => modified,
//             DiffNodeFsItem::TextFile { modified, .. } => modified,
//             DiffNodeFsItem::Dir { children, .. } => {
//                 children.iter().fold(&SystemTime::UNIX_EPOCH, |acc, child| {
//                     let child_modified = child.get_modified();
//                     if child_modified > acc {
//                         child_modified
//                     } else {
//                         acc
//                     }
//                 })
//             },
//             DiffNodeFsItem::NoChange(fs_item) => fs_item.get_modified(),
//             DiffNodeFsItem::Added(fs_item) => fs_item.get_modified(),
//             DiffNodeFsItem::Modified { new, .. } => new.get_modified()
//         }
//     }

//     fn get_size(&self) -> usize {
//         match self {
//             DiffNodeFsItem::Symlink { size, .. } => *size,
//             DiffNodeFsItem::BinaryFile { size, .. } => *size,
//             DiffNodeFsItem::TextFile { size, .. } => *size,
//             DiffNodeFsItem::Dir { children, .. } => {
//                 children.iter().fold(0, |acc, child| {
//                     acc + child.get_size()
//                 })
//             },
//             DiffNodeFsItem::NoChange(fs_item) => fs_item.get_size(),
//             DiffNodeFsItem::Added(fs_item) => fs_item.get_size(),
//             DiffNodeFsItem::Modified { new, .. } => new.get_size()
//         }
//     }

//     fn get_ancestor(&self) -> Option<&Self> {
//         match self {
//             DiffNodeFsItem::Symlink { .. } => None,
//             DiffNodeFsItem::BinaryFile { .. } => None,
//             DiffNodeFsItem::TextFile { .. } => None,
//             DiffNodeFsItem::Dir { .. } => None,
//             DiffNodeFsItem::NoChange(fs_item) => fs_item.get_ancestor(),
//             DiffNodeFsItem::Added(_) => None,
//             DiffNodeFsItem::Modified { old, .. } => Some(old)
//         }
//     }

//     fn fold_history(&self) -> &Self {
//         match self {
//             DiffNodeFsItem::Symlink { .. } => self,
//             DiffNodeFsItem::BinaryFile { .. } => self,
//             DiffNodeFsItem::TextFile { .. } => self,
//             DiffNodeFsItem::Dir { .. } => self,
//             DiffNodeFsItem::NoChange(fs_item) => fs_item.fold_history(),
//             DiffNodeFsItem::Added(fs_item) => fs_item.fold_history(),
//             DiffNodeFsItem::Modified { new, .. } => new.fold_history()
//         }
//     }
// }

// impl MerkleNode for DiffNodeFsItem {
//     fn get_hash(&self) -> &MerkleHash {
//         self.get_hash()
//     }

//     fn get_children(&self) -> Vec<&dyn MerkleNode> {
//         match self {
//             DiffNodeFsItem::Symlink { .. } => Vec::new(),
//             DiffNodeFsItem::BinaryFile { .. } => Vec::new(),
//             DiffNodeFsItem::TextFile { location_node, .. } => vec![location_node as &dyn MerkleNode],
//             DiffNodeFsItem::Dir { children, .. } => children.iter().map(|c| c as &dyn MerkleNode).collect(),
//             DiffNodeFsItem::NoChange(fs_item) => fs_item.get_children(),
//             DiffNodeFsItem::Added(fs_item) => fs_item.get_children(),
//             DiffNodeFsItem::Modified { new, .. } => new.get_children()
//         }
//     }
// }

// impl DiffHistory for DiffNodeFsItem {
//     fn get_ancestor(&self) -> Option<&Self> {
//         self.get_ancestor()
//     }

//     fn fold_history(&self) -> &Self {
//         self.fold_history()
//     }
// }

// #[derive(Debug, PartialEq, Eq)]
// enum DiffNode {
//     FsItem(DiffNodeFsItem),
//     TextLines(DiffNodeTextLines)
// }

// impl DiffNode {
//     fn get_hash(&self) -> &MerkleHash {
//         match self {
//             DiffNode::FsItem(fs_item) => fs_item.get_hash(),
//             DiffNode::TextLines(text_file_line_location) => text_file_line_location.get_hash()
//         }
//     }

//     fn get_children(&self) -> Vec<&dyn MerkleNode> {
//         match self {
//             DiffNode::FsItem(fs_item) => fs_item.get_children(),
//             DiffNode::TextLines(text_file_line_location) => text_file_line_location.get_children()
//         }
//     }

//     // fn get_ancestor(&self) -> Option<&Self> {
//     //     match self {
//     //         DiffNode::FsItem(fs_item) => fs_item.get_ancestor().map(|x| &Self::FsItem(x)),
//     //         DiffNode::TextLines(text_file_line_location) => text_file_line_location.get_ancestor()
//     //     }
//     // }

//     // fn fold_history(&self) -> &Self {
//     //     match self {
//     //         DiffNode::FsItem(fs_item) => fs_item.fold_history(),
//     //         DiffNode::TextLines(text_file_line_location) => text_file_line_location.fold_history()
//     //     }
//     // }
// }

// impl MerkleNode for DiffNode {
//     fn get_hash(&self) -> &MerkleHash {
//         self.get_hash()
//     }

//     fn get_children(&self) -> Vec<&dyn MerkleNode> {
//         self.get_children()
//     }
// }