// use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use candle_core::{Device, Tensor};
use std::{collections::HashMap, fs::File, path::PathBuf};
use tracing::info;
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

const ELF: &[u8] = include_bytes!("../../.././elf/riscv32im-succinct-zkvm-elf");

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
    let model_path = args.model_path.clone();
    let model = load_onnx_model(&model_path)?;
    info!("Number of ONNX operators: {}", model.num_operators());

    let input = Tensor::from_vec(input_data, args.input_shape, &Device::Cpu)?;
    let mut inputs: HashMap<String, Tensor> = HashMap::new();
    inputs.insert("input".to_string(), input);
    let result = model.inference(&mut inputs)?;
    info!("Inference result: {:?}", result["output"].to_string());
    println!("inputs: {:?}", inputs["/Reshape_output_0"].to_string());
    inputs.remove("/Reshape_output_0");
    println!("inputs: {:?}", inputs);

    let node = model.get_node(3).unwrap();
    println!("node: {:?}", node.output[0]);
    let _ = model.eval_one(node, &mut inputs)?;
    println!("inputs: {:?}", inputs["/Reshape_output_0"].to_string());

    // // Create SP1 proof of execution
    // let n = 1000u32;
    // let mut stdin = SP1Stdin::new();
    // stdin.write(&n);

    // let client = ProverClient::new();

    // let (_, report) = client.execute(ELF, stdin.clone()).run().unwrap();
    // println!(
    //     "executed program with {} cycles",
    //     report.total_instruction_count()
    // );

    // let (pk, vk) = client.setup(ELF);
    // let mut proof = client.prove(&pk, stdin).run().unwrap();

    // println!("generated proof");

    // let _ = proof.public_values.read::<u32>();
    // let a = proof.public_values.read::<u32>();
    // let b = proof.public_values.read::<u32>();

    // println!("a: {}", a);
    // println!("b: {}", b);

    // client.verify(&proof, &vk).expect("verification failed");

    // println!("verified proof");

    Ok(())
}
