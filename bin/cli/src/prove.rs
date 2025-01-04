use candle_core::Tensor;
use candle_onnx::eval::simple_eval_one;
use sp1_sdk::{ProverClient, SP1Stdin};
use std::{collections::HashMap, fs::File, path::PathBuf};
use tracing::info;
use zkopml_ml::{
    data::DataFile,
    merkle::{MerkleTreeHash, ModelMerkleTree},
    onnx::load_onnx_model,
};

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

    let mut inputs: HashMap<String, Tensor> = HashMap::new();
    model.prepare_inputs(&mut inputs, input_data.clone(), args.input_shape.clone())?;
    let result = model.inference(&mut inputs)?;
    info!("Inference result: {:?}", result["output"].to_string());

    // Create merkle tree from ONNX operators
    info!("Creating a Merkle tree from the model operators.");
    let nodes = model.graph().unwrap().node;
    let merkle_tree = ModelMerkleTree::new(nodes);
    info!("Merkle root hash: {}", merkle_tree.root_hash());

    // Create SP1 proof of execution
    let mut stdin = SP1Stdin::new();
    stdin.write(&merkle_tree.root());
    let leaf_indices = vec![2_usize];
    stdin.write(&leaf_indices);
    let leaf_hashes = merkle_tree.leaves_hashes(leaf_indices.clone());
    stdin.write(&leaf_hashes);
    let total_leaves = merkle_tree.total_leaves();
    stdin.write(&total_leaves);
    let merkle_proof: Vec<u8> = merkle_tree.prove(leaf_indices).to_bytes();
    stdin.write(&merkle_proof);
    let mut inputs: HashMap<String, Tensor> = HashMap::new();
    model.prepare_inputs(&mut inputs, input_data, args.input_shape)?;
    for i in 0..2 {
        let node = model.get_node(i).unwrap();
        simple_eval_one(&node, &mut inputs)?;
    }
    stdin.write(&inputs);
    stdin.write(&model.get_node(2).unwrap());

    let client = ProverClient::new();

    let (mut public_values, report) = client.execute(ELF, stdin.clone()).run().unwrap();
    info!(
        "executed program with {} cycles",
        report.total_instruction_count()
    );

    let merkle_root_ret = public_values.read::<MerkleTreeHash>();
    let leaf_indices_ret = public_values.read::<Vec<usize>>();
    let inputs_ret = public_values.read::<HashMap<String, Tensor>>();
    let outputs_ret = public_values.read::<HashMap<String, Tensor>>();

    info!("Returned public values:");
    info!("Merkle root: {:?}", merkle_root_ret);
    info!("Leaf indices: {:?}", leaf_indices_ret);
    info!("Inputs: {:?}", inputs_ret);
    info!("Outputs: {:?}", outputs_ret);

    // let (pk, vk) = client.setup(ELF);
    // info!("generated keys (setup)");

    // let proof = client.prove(&pk, stdin).run().unwrap();
    // info!("generated proof");

    // info!("a: {}", a);
    // info!("b: {}", b);

    // client.verify(&proof, &vk).expect("verification failed");

    // info!("verified proof");

    Ok(())
}
