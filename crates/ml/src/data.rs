use candle_core::Tensor;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

pub fn tensor_hash(tensor: &Tensor) -> [u8; 32] {
    let tensor = tensor.round_to(3).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_string(&tensor).unwrap().as_bytes());
    hasher.finalize().into()
}

pub fn extract_input_data(json_str: &str) -> anyhow::Result<Vec<f64>> {
    let json: Value = serde_json::from_str(json_str)?;

    let input_data = json
        .get("input_data")
        .ok_or_else(|| anyhow::anyhow!("Missing 'input_data' field in JSON"))?;

    let mut result = Vec::new();
    extract_numbers_recursive(input_data, &mut result)?;

    Ok(result)
}

fn extract_numbers_recursive(value: &Value, result: &mut Vec<f64>) -> anyhow::Result<()> {
    match value {
        Value::Number(n) => {
            let f64_val = n
                .as_f64()
                .ok_or_else(|| anyhow::anyhow!("Cannot convert number {} to f64", n))?;
            result.push(f64_val);
        }
        Value::Array(arr) => {
            for item in arr {
                extract_numbers_recursive(item, result)?;
            }
        }
        Value::Null => {}
        _ => {
            return Err(anyhow::anyhow!("Unexpected value type: {:?}", value));
        }
    }
    Ok(())
}

pub fn extract_input_data_iterative(json_str: &str) -> anyhow::Result<Vec<f64>> {
    let json: Value = serde_json::from_str(json_str)?;

    let input_data = json
        .get("input_data")
        .ok_or_else(|| anyhow::anyhow!("Missing 'input_data' field in JSON"))?;

    let mut result = Vec::new();
    let mut stack = VecDeque::new();
    stack.push_back(input_data);

    while let Some(current) = stack.pop_front() {
        match current {
            Value::Number(n) => {
                let f64_val = n
                    .as_f64()
                    .ok_or_else(|| anyhow::anyhow!("Cannot convert number {} to f64", n))?;
                result.push(f64_val);
            }
            Value::Array(arr) => {
                for item in arr.iter().rev() {
                    stack.push_front(item);
                }
            }
            Value::Null => {}
            _ => {
                return Err(anyhow::anyhow!("Unexpected value type: {:?}", current));
            }
        }
    }

    Ok(result)
}
