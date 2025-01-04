#![no_main]
sp1_zkvm::entrypoint!(main);

use candle_core::Tensor;
use candle_onnx::{eval::simple_eval_one, onnx::NodeProto};
use std::collections::HashMap;

pub fn main() {
    // read merkle tree/proof data
    let merkle_root = sp1_zkvm::io::read::<[u8; 32]>();
    let merkle_proof = sp1_zkvm::io::read::<Vec<u8>>();
    let operator_index = sp1_zkvm::io::read::<u32>();

    // read onnx data
    let mut inputs = sp1_zkvm::io::read::<HashMap<String, Tensor>>();
    let node = sp1_zkvm::io::read::<NodeProto>();

    // commit to certain values (public values)
    sp1_zkvm::io::commit(&merkle_root);
    sp1_zkvm::io::commit(&operator_index);
    sp1_zkvm::io::commit(&inputs);

    // TODO: verify merkle proof

    // perform execution of one ONNX operator
    simple_eval_one(&node, &mut inputs).expect("Execution error");

    sp1_zkvm::io::commit(&inputs);
}
