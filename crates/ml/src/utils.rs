use candle_onnx::{
    eval::get_tensor,
    onnx::{GraphProto, NodeProto},
};
use sha2::{Digest, Sha256};

use crate::data::tensor_hash;

pub fn hash_buffer(buffer: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(buffer);
    hasher.finalize().into()
}

pub fn node_hash(node: &NodeProto, graph: &GraphProto) -> [u8; 32] {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&serde_json::to_vec(&node).unwrap());

    let mut node_inputs = node.input.clone();
    node_inputs.sort();

    for input in node_inputs.iter() {
        for t in graph.initializer.iter() {
            if input == &t.name {
                let tensor = get_tensor(t, t.name.as_str()).unwrap();
                buffer.extend_from_slice(&tensor_hash(&tensor));
                break;
            }
        }
    }

    hash_buffer(&buffer)
}
