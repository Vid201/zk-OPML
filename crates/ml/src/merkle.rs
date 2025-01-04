use candle_onnx::onnx::NodeProto;
use rs_merkle::algorithms::Sha256;
use rs_merkle::{Hasher, MerkleProof, MerkleTree};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;

pub type MerkleTreeHash = [u8; 32];

pub struct ModelMerkleTree {
    pub inner: MerkleTree<Sha256>,
}

impl ModelMerkleTree {
    pub fn new(nodes: Vec<NodeProto>) -> Self {
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

    pub fn root(&self) -> MerkleTreeHash {
        self.inner.root().unwrap()
    }

    pub fn root_hash(&self) -> String {
        self.inner.root_hex().unwrap()
    }

    pub fn leaves_hashes(&self, indices: Vec<usize>) -> Vec<MerkleTreeHash> {
        let mut hashes = vec![];

        for i in indices {
            let hash = self.inner.leaves().unwrap().get(i).unwrap().clone();
            hashes.push(hash);
        }

        hashes
    }

    pub fn total_leaves(&self) -> usize {
        self.inner.leaves().unwrap().len()
    }

    pub fn prove(&self, indices: Vec<usize>) -> MerkleProof<Sha256> {
        self.inner.proof(&indices)
    }
}

pub fn hash_string(s: &str) -> u64 {
    use std::hash::Hasher;

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
