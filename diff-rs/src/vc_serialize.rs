use std::collections::HashMap;
use std::fmt::Write;
use std::io::{Read, Write as IoWrite, BufReader};
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::{fmt::Display, path::PathBuf, time::SystemTime, error::Error};
use digest::{Digest, generic_array::GenericArray};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use serde::{Deserialize, Serialize};

use crate::hashing::*;
use crate::merkle::*;
use crate::vc::*;

pub fn serialize_json<S>(obj: S) -> Result<String, serde_json::Error>
where S: Serialize {
    serde_json::to_string_pretty(&obj)
}

pub fn deserialize_json<D>(json: &str) -> Result<D, serde_json::Error>
where D: DeserializeOwned {
    serde_json::from_str(json)
}

pub fn serialize_json_to_file<S, P>(obj: S, path: P) -> Result<(), Box<dyn Error>>
where S: Serialize, P: AsRef<Path> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)?;
    let json = serialize_json(obj)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn deserialize_json_from_file<D, P>(path: P) -> Result<D, Box<dyn Error>>
where D: DeserializeOwned, P: AsRef<Path> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let obj = serde_json::from_reader(reader)?;
    Ok(obj)
}

pub trait SerializeDeserializeJson : Serialize + DeserializeOwned {
    fn serialize_json(&self) -> Result<String, serde_json::Error> {
        serialize_json(self)
    }

    fn serialize_json_to_file<P>(&self, path: P) -> Result<(), Box<dyn Error>>
    where P: AsRef<Path> {
        serialize_json_to_file(self, path)
    }

    fn deserialize_json(json: &str) -> Result<Self, serde_json::Error> {
        deserialize_json(json)
    }

    fn deserialize_json_from_file<P>(path: P) -> Result<Self, Box<dyn Error>>
    where P: AsRef<Path> {
        deserialize_json_from_file(path)
    }
}

pub trait SaveLoadObjectJson : SerializeDeserializeJson + VcHashId {
    fn save_object_json<P>(&self, parent_path: P) -> Result<(), Box<dyn Error>>
    where P: AsRef<Path> {
        let path = PathBuf::from(parent_path.as_ref()).join(self.get_hash_str());
        self.serialize_json_to_file(path)
    }

    fn load_object_json<P>(parent_path: P, hashstr: &str) -> Result<Self, Box<dyn Error>>
    where P: AsRef<Path> {
        let path = PathBuf::from(parent_path.as_ref()).join(hashstr);
        Self::deserialize_json_from_file(path)
    }
}

#[derive(Debug, PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
pub struct BlobStub {
    pub hashstr: VcHashString
}
impl SerializeDeserializeJson for BlobStub {}
impl VcHashId for BlobStub {
    fn get_hash_bytes(&self) -> VcHash {
        hex_string_to_hash::<VcHasher>(&self.hashstr).unwrap()
    }

    fn get_hash_str(&self) -> VcHashString {
        self.hashstr.clone()
    }
}
impl SaveLoadObjectJson for BlobStub {}

#[derive(Debug, PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
pub struct TreeStub {
    pub listings: HashMap<Name, (FsObjectType, VcHashString)>,
    pub hashstr: VcHashString
}
impl SerializeDeserializeJson for TreeStub {}
impl VcHashId for TreeStub {
    fn get_hash_bytes(&self) -> VcHash {
        hex_string_to_hash::<VcHasher>(&self.hashstr).unwrap()
    }

    fn get_hash_str(&self) -> VcHashString {
        self.hashstr.clone()
    }
}
impl SaveLoadObjectJson for TreeStub {}

#[derive(Debug, PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
pub struct CommitStub {
    pub tree_hashstr: VcHashString,
    pub hashstr: VcHashString,
    pub parent_hashstr: Option<VcHashString>,
    pub author: String,
    pub message: String,
    pub timestamp: SystemTime
}
impl SerializeDeserializeJson for CommitStub {}
impl VcHashId for CommitStub {
    fn get_hash_bytes(&self) -> VcHash {
        hex_string_to_hash::<VcHasher>(&self.hashstr).unwrap()
    }

    fn get_hash_str(&self) -> VcHashString {
        self.hashstr.clone()
    }
}
impl SaveLoadObjectJson for CommitStub {}

#[derive(Debug, PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
pub enum FsObjectType {
    Blob,
    Tree
}
impl SerializeDeserializeJson for FsObjectType {}

#[derive(Debug, PartialEq, Clone)]
pub enum FsObjectStub {
    BlobStub(BlobStub),
    TreeStub(TreeStub)
}
impl Serialize for FsObjectStub {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        match self {
            FsObjectStub::BlobStub(blob_stub) => blob_stub.serialize(serializer),
            FsObjectStub::TreeStub(tree_stub) => tree_stub.serialize(serializer)
        }
    }
}
impl<'a> Deserialize<'a> for FsObjectStub {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'a> {
        Ok(FsObjectStub::TreeStub(TreeStub::deserialize(deserializer)?))
    }
}
impl SerializeDeserializeJson for FsObjectStub {}
impl VcHashId for FsObjectStub {
    fn get_hash_bytes(&self) -> VcHash {
        match self {
            FsObjectStub::BlobStub(blob_stub) => blob_stub.get_hash_bytes(),
            FsObjectStub::TreeStub(tree_stub) => tree_stub.get_hash_bytes()
        }
    }

    fn get_hash_str(&self) -> VcHashString {
        match self {
            FsObjectStub::BlobStub(blob_stub) => blob_stub.get_hash_str(),
            FsObjectStub::TreeStub(tree_stub) => tree_stub.get_hash_str()
        }
    }
}
impl SaveLoadObjectJson for FsObjectStub {}

#[derive(Debug, PartialEq, Clone)]
pub enum VcObjectStub {
    FsObjectStub(FsObjectStub),
    CommitStub(CommitStub)
}
impl Serialize for VcObjectStub {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        match self {
            VcObjectStub::FsObjectStub(fs_object_stub) => fs_object_stub.serialize(serializer),
            VcObjectStub::CommitStub(commit_stub) => commit_stub.serialize(serializer)
        }
    }
}
impl<'a> Deserialize<'a> for VcObjectStub {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'a> {
        // CommitStub::deserialize(deserializer)
        //     .map(|commit_stub| VcObjectStub::CommitStub(commit_stub))
        //     .or_else(|_| {
        //         let x = FsObjectStub::deserialize(deserializer)?;
        //         Ok(VcObjectStub::FsObjectStub(x))
        //     })
        // try to parse as commit stub, if that fails, try to parse as fs object stub
        
    }
}
impl VcHashId for VcObjectStub {
    fn get_hash_bytes(&self) -> VcHash {
        match self {
            VcObjectStub::FsObjectStub(fs_object_stub) => fs_object_stub.get_hash_bytes(),
            VcObjectStub::CommitStub(commit_stub) => commit_stub.get_hash_bytes()
        }
    }

    fn get_hash_str(&self) -> VcHashString {
        match self {
            VcObjectStub::FsObjectStub(fs_object_stub) => fs_object_stub.get_hash_str(),
            VcObjectStub::CommitStub(commit_stub) => commit_stub.get_hash_str()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum HeadRefStub {
    Tag(Name),
    Head(Name),
    Commit(VcHashString)
}

impl BlobStub {
    pub fn from_blob(blob: &Blob) -> Self {
        Self {
            hashstr: hash_to_hex_string(&blob.hash)
        }
    } 
}

impl TreeStub {
    pub fn from_tree(tree: &Tree) -> Self {
        let listings = tree.listings.iter().map(|(name, fs_object)| {
            let (fs_object_type, hashstr) = match fs_object {
                FsObject::Blob(blob) => (FsObjectType::Blob, hash_to_hex_string(&blob.hash)),
                FsObject::Tree(tree) => (FsObjectType::Tree, hash_to_hex_string(&tree.hash))
            };
            (name.clone(), (fs_object_type, hashstr))
        }).collect::<HashMap<_, _>>();

        Self {
            listings,
            hashstr: hash_to_hex_string(&tree.hash)
        }
    }
}

impl CommitStub {
    pub fn from_commit(commit: &Commit) -> Self {
        let parent_hashstr = match &commit.parent {
            Some(parent) => Some(hash_to_hex_string(&parent.hash)),
            None => None
        };
        Self {
            tree_hashstr: hash_to_hex_string(&commit.tree.hash),
            hashstr: hash_to_hex_string(&commit.hash),
            parent_hashstr,
            author: commit.author.clone(),
            message: commit.message.clone(),
            timestamp: commit.timestamp
        }
    }
}

#[test]
fn test_blob_stub_serialize_json() {
    let blob = Blob::new(b"hello");
    let blob_stub = BlobStub::from_blob(&blob);
    let json = blob_stub.serialize_json().unwrap();
    let blob_stub2 = BlobStub::deserialize_json(&json).unwrap();
    print!("{}", json);
    assert_eq!(blob_stub, blob_stub2);
}

#[test]
fn test_tree_stub_serialize_json() {
    let mut tree = Tree::new(HashMap::new());
    tree.listings.insert("hello".to_string(), FsObject::Blob(Blob::new(b"hello")));
    let tree_stub = TreeStub::from_tree(&tree);
    let json = tree_stub.serialize_json().unwrap();
    let tree_stub2 = TreeStub::deserialize_json(&json).unwrap();
    print!("{}", json);
    assert_eq!(tree_stub, tree_stub2);
}
