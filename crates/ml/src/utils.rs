use candle_onnx::{
    eval::get_tensor,
    onnx::{GraphProto, NodeProto},
};
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

pub fn node_hash(node: &NodeProto, graph: &GraphProto) -> u64 {
    let mut node_str = format!("{:?}", node);
    let mut weights_str = String::new();
    for input in node.input.iter() {
        for t in graph.initializer.iter() {
            if input == &t.name {
                let tensor = get_tensor(t, t.name.as_str()).unwrap();
                weights_str.push_str(serde_json::to_string(&tensor).unwrap().as_str());
                break;
            }
        }
    }
    node_str.push_str(&weights_str);
    hash_string(&node_str)
}
