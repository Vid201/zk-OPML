use rs_merkle::algorithms::Sha256;
use rs_merkle::{Hasher, MerkleTree};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use tract_onnx::prelude::{Node, TypedFact, TypedOp};

pub struct ModelMerkleTree {
    pub inner: MerkleTree<Sha256>,
}

impl ModelMerkleTree {
    pub fn new(nodes: Vec<Node<TypedFact, Box<dyn TypedOp>>>) -> Self {
        let leaves: Vec<[u8; 32]> = nodes
            .iter()
            .map(|node| {
                let node_str = format!("{:?}", node);
                let hash = hash_string(&node_str);
                Sha256::hash(hash.to_string().as_bytes())
            })
            .collect();

        Self {
            inner: MerkleTree::<Sha256>::from_leaves(&leaves),
        }
    }

    pub fn root(&self) -> [u8; 32] {
        self.inner.root().unwrap()
    }

    pub fn root_hash(&self) -> String {
        self.inner.root_hex().unwrap()
    }
}

pub fn hash_string(s: &str) -> u64 {
    use std::hash::Hasher;

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
