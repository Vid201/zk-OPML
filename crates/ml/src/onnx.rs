use tract_onnx::prelude::{Framework, Graph, InferenceModelExt, TypedFact, TypedOp};

#[derive(Debug)]
pub struct Model {
    pub graph: Graph<TypedFact, Box<dyn TypedOp>>,
}

pub fn load_onnx_model(reader: &mut dyn std::io::Read) -> anyhow::Result<Model> {
    let model = tract_onnx::onnx().model_for_read(reader)?;
    Ok(Model {
        graph: model.into_typed()?,
    })
}
