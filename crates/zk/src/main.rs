#![no_main]
sp1_zkvm::entrypoint!(main);

use candle_core::Tensor;
use candle_onnx::{eval::simple_eval_one, onnx::NodeProto};
use rs_merkle::{algorithms::Sha256, MerkleProof};
use sha2::Digest;
use std::collections::HashMap;
use zkopml_ml::{data::tensor_hash, utils::hash_string};

pub fn main() {
    // read merkle tree/proof data
    let merkle_root = sp1_zkvm::io::read::<[u8; 32]>();
    let leaf_indices = sp1_zkvm::io::read::<Vec<usize>>();
    let leaf_hashes = sp1_zkvm::io::read::<Vec<[u8; 32]>>();
    let total_leaves = sp1_zkvm::io::read::<usize>();
    let merkle_proof = sp1_zkvm::io::read::<Vec<u8>>();

    // read onnx data
    let mut inputs = sp1_zkvm::io::read::<HashMap<String, Tensor>>();
    let mut inputs_hashes = sp1_zkvm::io::read::<HashMap<String, [u8; 32]>>();
    let node = sp1_zkvm::io::read::<NodeProto>();

    // commit to certain values (public values)
    sp1_zkvm::io::commit(&merkle_root);
    sp1_zkvm::io::commit(&leaf_indices);

    let mut input_entries = inputs_hashes.iter().collect::<Vec<_>>();
    input_entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut hasher = sha2::Sha256::new();
    hasher.update(serde_json::to_string(&input_entries).unwrap().as_bytes()); // TODO: figure out how to more efficiently hash a tensor
    let hash: [u8; 32] = hasher.finalize().into();
    sp1_zkvm::io::commit(&hash);

    // verify merkle proof
    let proof = MerkleProof::<Sha256>::try_from(merkle_proof).expect("Invalid merkle proof");
    assert!(proof.verify(merkle_root, &leaf_indices, &leaf_hashes, total_leaves));

    // verify correct hash for ONNX operator
    // TODO: support multiple operators
    let mut node_str = format!("{:?}", node);
    let mut weights_str = String::new();
    for input in node.input.iter() {
        for (name, tensor) in inputs.iter() {
            if input != "input" && input == name && !input.contains("output") {
                weights_str.push_str(serde_json::to_string(&tensor).unwrap().as_str());
                break;
            } else {
                // verify that we are working with correct input/output values
                let hash = tensor_hash(tensor);
                assert!(hash == inputs_hashes.get(name).unwrap().clone());
            }
        }
    }
    node_str.push_str(&weights_str);
    let hash = hash_string(&node_str);
    let node_hash = {
        use rs_merkle::Hasher;
        Sha256::hash(hash.as_slice())
    };
    assert!(node_hash == leaf_hashes[0]);

    // perform execution of one ONNX operator
    // TODO: execute multiple operators
    simple_eval_one(&node, &mut inputs).expect("Execution error");

    // add output values to inputs_hashes
    for (name, tensor) in inputs.iter() {
        if !inputs_hashes.contains_key(name) {
            let hash = tensor_hash(tensor);
            inputs_hashes.insert(name.clone(), hash);
        }
    }

    let mut input_entries = inputs_hashes.iter().collect::<Vec<_>>();
    input_entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut hasher = sha2::Sha256::new();
    hasher.update(serde_json::to_string(&input_entries).unwrap().as_bytes()); // TODO: figure out how to more efficiently hash a tensor
    let hash: [u8; 32] = hasher.finalize().into();
    sp1_zkvm::io::commit(&hash);
}
