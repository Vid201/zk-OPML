use candle_core::Tensor;
use candle_onnx::{
    eval::simple_eval_one,
    onnx::{GraphProto, ModelProto, NodeProto},
    read_file, simple_eval,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Model {
    pub inner: ModelProto,
}

impl Model {
    pub fn graph(&self) -> Option<GraphProto> {
        self.inner.graph.clone()
    }

    pub fn get_node(&self, index: usize) -> Option<NodeProto> {
        self.inner.graph.clone().unwrap().node.get(index).cloned()
    }

    pub fn num_operators(&self) -> usize {
        self.inner.graph.clone().unwrap().node.len()
    }

    pub fn inference(
        &self,
        inputs: &mut HashMap<String, Tensor>,
    ) -> anyhow::Result<HashMap<String, Tensor>> {
        simple_eval(&self.inner, inputs).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn eval_one(
        &self,
        node: NodeProto,
        inputs: &mut HashMap<String, Tensor>,
    ) -> anyhow::Result<()> {
        simple_eval_one(&node, inputs).map_err(|e| anyhow::anyhow!(e))
    }
}

pub fn load_onnx_model(path: &String) -> anyhow::Result<Model> {
    let model = read_file(&path)?;
    Ok(Model { inner: model })
}
