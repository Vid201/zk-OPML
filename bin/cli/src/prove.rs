use std::{fs::File, path::PathBuf};
use tracing::info;
use tract_onnx::prelude::{tvec, DatumExt, InferenceFact, SimpleState, Tensor};
use zkopml_ml::{data::DataFile, onnx::load_onnx_model};

#[derive(clap::Args, Debug, Clone)]
pub struct ProveArgs {
    #[arg(long, short, help = "Verbosity level (0-4)", action = clap::ArgAction::Count)]
    pub v: u8,

    /// Path to the model file (ONNX)
    #[clap(long)]
    pub model_path: String,

    /// Path to the input data
    #[clap(long)]
    pub input_data_path: String,

    /// Input shape of the model
    #[clap(long, value_delimiter = ',')]
    pub input_shape: Vec<usize>,

    /// Output shape of the model
    #[clap(long, value_delimiter = ',')]
    pub output_shape: Vec<usize>,
}

pub async fn prove(args: ProveArgs) -> anyhow::Result<()> {
    // Read the input data
    info!("Reading the input data from {}", args.input_data_path);
    let path = PathBuf::from(&args.input_data_path);
    let reader = File::open(&path)?;
    let data_file: DataFile = serde_json::from_reader(reader)?;
    let input_data: Vec<f32> = data_file.input_data.into_iter().flat_map(|v| v).collect();
    info!("Input data: {:?}", input_data);

    // Load the model and perform the inference
    info!("Reading the model file from {}", args.model_path);
    let mut file = std::fs::File::open(args.model_path.clone())?;
    let input_fact: InferenceFact = f32::fact(args.input_shape.as_slice()).into();
    let model = load_onnx_model(&mut file, input_fact)?;
    let input = Tensor::from_shape(args.input_shape.as_slice(), input_data.as_ref())?;
    // let result = model/.inner.run(tvec!(input.clone().into()))?;
    let mut simple_state = SimpleState::new(model.inner)?;
    simple_state.set_inputs(tvec!(input.clone().into()))?;
    simple_state.compute_one(0)?;
    simple_state.compute_one(89999)?;
    // info!("Inference result: {:?}", result);

    Ok(())
}
