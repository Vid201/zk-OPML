#![no_main]
sp1_zkvm::entrypoint!(main);

use candle_core::Tensor;
use candle_onnx::{eval::simple_eval_one, onnx::NodeProto};
use rs_merkle::{algorithms::Sha256, MerkleProof};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};
use zkopml_ml::utils::hash_string;

pub fn main() {
    // read merkle tree/proof data
    let merkle_root = sp1_zkvm::io::read::<[u8; 32]>();
    let leaf_indices = sp1_zkvm::io::read::<Vec<usize>>();
    let leaf_hashes = sp1_zkvm::io::read::<Vec<[u8; 32]>>();
    let total_leaves = sp1_zkvm::io::read::<usize>();
    let merkle_proof = sp1_zkvm::io::read::<Vec<u8>>();

    // read onnx data
    let mut inputs = sp1_zkvm::io::read::<HashMap<String, Tensor>>();
    let node = sp1_zkvm::io::read::<NodeProto>();

    // commit to certain values (public values)
    sp1_zkvm::io::commit(&merkle_root);
    sp1_zkvm::io::commit(&leaf_indices);

    let mut input_entries = inputs.iter().collect::<Vec<_>>();
    input_entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut hasher = DefaultHasher::new();
    input_entries.hash(&mut hasher); // TODO: figure out how to more efficiently hash a tensor
    sp1_zkvm::io::commit(&hasher.finish());

    // verify merkle proof
    let proof = MerkleProof::<Sha256>::try_from(merkle_proof).expect("Invalid merkle proof");
    assert!(proof.verify(merkle_root, &leaf_indices, &leaf_hashes, total_leaves));

    // verify correct hash for ONNX operator
    // TODO: support multiple operators
    let node_str = format!("{:?}", node);
    let hash = hash_string(&node_str);
    let node_hash = {
        use rs_merkle::Hasher;
        Sha256::hash(hash.to_string().as_bytes())
    };
    assert!(node_hash == leaf_hashes[0]);

    // perform execution of one ONNX operator
    // TODO: execute multiple operators
    simple_eval_one(&node, &mut inputs).expect("Execution error");

    let mut input_entries = inputs.iter().collect::<Vec<_>>();
    input_entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut hasher = DefaultHasher::new();
    input_entries.hash(&mut hasher); // TODO: figure out how to more efficiently hash a tensor
    sp1_zkvm::io::commit(&hasher.finish());
}
