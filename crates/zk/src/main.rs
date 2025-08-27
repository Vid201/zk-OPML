#![no_main]
sp1_zkvm::entrypoint!(main);

use candle_core::Tensor;
use candle_onnx::{eval::simple_eval_one, onnx::NodeProto};
use rs_merkle::{MerkleProof, algorithms::Sha256};
use std::collections::HashMap;
use zkopml_ml::{data::tensor_hash, utils::hash_buffer};

pub fn main() {
    // println!("cycle-tracker-start: inputs");

    // read merkle tree/proof data
    let merkle_root = sp1_zkvm::io::read::<[u8; 32]>();
    let leaf_indices = sp1_zkvm::io::read::<Vec<usize>>();
    let leaf_hashes = sp1_zkvm::io::read::<Vec<[u8; 32]>>();
    let total_leaves = sp1_zkvm::io::read::<usize>();
    let merkle_proof = sp1_zkvm::io::read::<Vec<u8>>();

    // read onnx data
    let inputs_raw = sp1_zkvm::io::read::<HashMap<String, Tensor>>();
    let mut inputs_hashes = sp1_zkvm::io::read::<HashMap<String, [u8; 32]>>();
    let node = sp1_zkvm::io::read::<NodeProto>();

    // println!("cycle-tracker-end: inputs");

    // println!("cycle-tracker-start: merkle commit");

    // commit to certain values (public values)
    sp1_zkvm::io::commit(&merkle_root);
    sp1_zkvm::io::commit(&leaf_indices);

    // println!("cycle-tracker-end: merkle commit");

    // println!("cycle-tracker-start: commit to input data hash");

    let mut input_entries = inputs_hashes.iter().collect::<Vec<_>>();
    input_entries.sort_by(|a, b| a.0.cmp(b.0));
    let hash = hash_buffer(serde_json::to_string(&input_entries).unwrap().as_bytes());
    sp1_zkvm::io::commit(&hash);

    // println!("cycle-tracker-end: commit to input data hash");

    // println!("cycle-tracker-start: verify merkle proof");

    // verify merkle proof
    let proof = MerkleProof::<Sha256>::try_from(merkle_proof).expect("Invalid merkle proof");
    assert!(proof.verify(merkle_root, &leaf_indices, &leaf_hashes, total_leaves));

    // println!("cycle-tracker-end: verify merkle proof");

    // println!("cycle-tracker-start: verify onnx operator");

    // verify correct hash for ONNX operator
    // TODO: we could precompute all graph initializers beforehand in production (when commiting in the registry to the model) and just verfiy ZK proofs here
    let mut inputs: HashMap<String, Tensor> = HashMap::new();
    let mut node_inputs = Vec::new();

    let mut node_buffer = Vec::new();
    node_buffer.extend_from_slice(&serde_json::to_vec(&node).unwrap());

    for (name, tensor) in inputs_raw.iter() {
        let mut input_name = String::new();
        let hash = tensor_hash(tensor);
        if name.ends_with("graph_initializer") {
            input_name.push_str(name.replace("graph_initializer", "").as_str());
            node_inputs.push(input_name.clone());
        } else {
            input_name.push_str(name.as_str());
        }
        assert!(hash == inputs_hashes.get(&input_name).unwrap().clone());
        inputs.insert(input_name, tensor.clone());
    }

    node_inputs.sort();

    for node_input in node_inputs {
        node_buffer.extend_from_slice(inputs_hashes.get(&node_input).unwrap());
    }

    let node_hash = hash_buffer(&node_buffer);
    assert!(node_hash == leaf_hashes[0]);

    // println!("cycle-tracker-end: verify onnx operator");

    // println!("cycle-tracker-start: onnx execution");

    // perform execution of one ONNX operator
    // TODO: execute multiple operators
    simple_eval_one(&node, &mut inputs).expect("Execution error");

    // println!("cycle-tracker-end: onnx execution");

    // println!("cycle-tracker-start: commit to output data hash");

    // add output values to inputs_hashes
    for (name, tensor) in inputs.iter() {
        if !inputs_hashes.contains_key(name) {
            let hash = tensor_hash(tensor);
            inputs_hashes.insert(name.clone(), hash);
        }
    }

    let mut input_entries = inputs_hashes.iter().collect::<Vec<_>>();
    input_entries.sort_by(|a, b| a.0.cmp(b.0));
    let hash = hash_buffer(serde_json::to_string(&input_entries).unwrap().as_bytes());
    sp1_zkvm::io::commit(&hash);

    // println!("cycle-tracker-end: commit to output data hash");
}
