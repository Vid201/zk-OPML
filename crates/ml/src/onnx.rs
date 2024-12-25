use tract_onnx::prelude::{
    Framework, Graph, InferenceFact, InferenceModelExt, SimplePlan, TypedFact, TypedOp,
};

#[derive(Debug)]
pub struct Model {
    pub inner: SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
}

impl Model {
    pub fn graph(&self) -> &Graph<TypedFact, Box<dyn TypedOp>> {
        self.inner.model()
    }
}

pub fn load_onnx_model(
    reader: &mut dyn std::io::Read,
    input_fact: InferenceFact,
) -> anyhow::Result<Model> {
    let model = tract_onnx::onnx()
        .model_for_read(reader)?
        .with_input_fact(0, input_fact)?
        .into_optimized()?
        .into_runnable()?;
    Ok(Model { inner: model })
}
