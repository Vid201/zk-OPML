use candle_core::Tensor;
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub type Data = Vec<Vec<f32>>;

#[derive(Clone, Debug, Deserialize, Default, PartialEq)]
pub struct DataFile {
    pub input_data: Data,
    pub output_data: Option<Data>,
}

pub fn tensor_hash(tensor: &Tensor) -> [u8; 32] {
    let tensor = tensor.round_to(4).unwrap(); // round to 4 decimal places (to avoid floating point errors, TODO: should be fixed somehow different in the future)
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_string(&tensor).unwrap().as_bytes()); // TODO: figure out how to more efficiently hash a tensor
    hasher.finalize().into()
}
