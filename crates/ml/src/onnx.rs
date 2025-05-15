use candle_core::{Device, Tensor};
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

    pub fn prepare_inputs(&self, inputs: &mut HashMap<String, Tensor>) -> anyhow::Result<()> {
        let graph = self.inner.graph.clone().unwrap();
        let constants: std::collections::HashSet<_> =
            graph.initializer.iter().map(|i| i.name.as_str()).collect();
        for input in graph.input.iter() {
            use candle_onnx::onnx::tensor_proto::DataType;
            if constants.contains(input.name.as_str()) {
                continue;
            }

            let type_ = input.r#type.as_ref().expect("no type for input");
            let type_ = type_.value.as_ref().expect("no type.value for input");
            let value = match type_ {
                candle_onnx::onnx::type_proto::Value::TensorType(tt) => {
                    let dt = match DataType::try_from(tt.elem_type) {
                        Ok(dt) => match candle_onnx::dtype(dt) {
                            Some(dt) => dt,
                            None => {
                                anyhow::bail!(
                                    "unsupported 'value' data-type {dt:?} for {}",
                                    input.name
                                )
                            }
                        },
                        type_ => anyhow::bail!("unsupported input type {type_:?}"),
                    };
                    let shape = tt.shape.as_ref().expect("no tensortype.shape for input");
                    let dims = shape
                        .dim
                        .iter()
                        .map(|dim| match dim.value.as_ref().expect("no dim value") {
                            candle_onnx::onnx::tensor_shape_proto::dimension::Value::DimValue(
                                v,
                            ) => Ok(*v as usize),
                            candle_onnx::onnx::tensor_shape_proto::dimension::Value::DimParam(
                                d,
                            ) => {
                                // TODO: find better way to handle this for any model
                                Ok(match d.as_str() {
                                    // variable_cnn
                                    // whisper
                                    "batch_size" => 1,
                                    "decoder_sequence_length" => 2,
                                    "encoder_sequence_length / 2" => 1,
                                    // gte
                                    "sequence_length" => 32,
                                    // t5
                                    "encoder_sequence_length" => 2,
                                    _ => anyhow::bail!("unsupported dim param {d:?}"),
                                })
                            }
                        })
                        .collect::<anyhow::Result<Vec<usize>>>()?;
                    Tensor::ones(dims, dt, &Device::Cpu)?
                }
                type_ => anyhow::bail!("unsupported input type {type_:?}"),
            };
            inputs.insert(input.name.clone(), value);
        }

        Ok(())
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
