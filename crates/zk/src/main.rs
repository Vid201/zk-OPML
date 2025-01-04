#![no_main]
sp1_zkvm::entrypoint!(main);

use candle_core::Tensor;
use candle_onnx::{eval::simple_eval_one, onnx::NodeProto};
use rs_merkle::{algorithms::Sha256, MerkleProof};
use std::collections::HashMap;

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
    sp1_zkvm::io::commit(&inputs);

    // verify merkle proof
    let proof = MerkleProof::<Sha256>::try_from(merkle_proof).expect("Invalid merkle proof");
    assert!(proof.verify(merkle_root, &leaf_indices, &leaf_hashes, total_leaves));

    // perform execution of one ONNX operator
    // TODO: execute multiple operators
    simple_eval_one(&node, &mut inputs).expect("Execution error");

    sp1_zkvm::io::commit(&inputs);
}
