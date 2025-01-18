use candle_core::Tensor;
use candle_onnx::eval::simple_eval_one;
use sha2::{Digest, Sha256};
use sp1_sdk::{include_elf, network::FulfillmentStrategy, Prover, ProverClient, SP1Stdin};
use std::{collections::HashMap, fs::File, path::PathBuf};
use tracing::info;
use zkopml_ml::{
    data::DataFile,
    merkle::{MerkleTreeHash, ModelMerkleTree},
    onnx::load_onnx_model,
};

#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Eq)]
pub enum SP1Prover {
    Cpu,
    Network,
}

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

    /// Index of the ONNX operator to prove
    #[clap(long)]
    pub operator_index: usize,

    /// Type of SP1 prover
    #[clap(long, default_value = "cpu")]
    pub sp1_prover: SP1Prover,
}

const ELF: &[u8] = include_elf!("zkopml-sp1");

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
    let merkle_tree = ModelMerkleTree::new(nodes, model.graph().unwrap());
    info!("Merkle root hash: {:?}", merkle_tree.root());

    // Create SP1 proof of execution
    let node_index = args.operator_index;
    let mut stdin = SP1Stdin::new();
    stdin.write(&merkle_tree.root());

    let leaf_indices = vec![node_index];
    stdin.write(&leaf_indices);

    let leaf_hashes = merkle_tree.leaves_hashes(leaf_indices.clone());
    stdin.write(&leaf_hashes);

    let total_leaves = merkle_tree.total_leaves();
    stdin.write(&total_leaves);

    let merkle_proof: Vec<u8> = merkle_tree.prove(leaf_indices).to_bytes();
    stdin.write(&merkle_proof);

    let mut inputs: HashMap<String, Tensor> = HashMap::new();
    model.prepare_inputs(&mut inputs, input_data, args.input_shape)?;
    for i in 0..node_index {
        let node = model.get_node(i).unwrap();
        simple_eval_one(&node, &mut inputs)?;
    }
    let node = model.get_node(node_index).unwrap();
    inputs.retain(|k, _| node.input.contains(k));
    stdin.write(&inputs);

    let mut input_hashes = HashMap::new();
    for (name, tensor) in inputs.iter() {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&tensor).unwrap().as_bytes()); // TODO: figure out how to more efficiently hash a tensor
        let hash: Vec<u8> = hasher.finalize().to_vec();
        input_hashes.insert(name.clone(), hash);
    }
    stdin.write(&input_hashes);

    stdin.write(&node);

    if args.sp1_prover == SP1Prover::Cpu {
        info!("Using the local/cpu SP1 prover.");
        let client = ProverClient::builder().cpu().build();
        info!(
            "Executing the SP1 program. Proving ONNX operator: {:?}",
            node
        );

        let (mut public_values, report) = client.execute(ELF, &stdin).run().unwrap();
        info!(
            "executed program with {} cycles",
            report.total_instruction_count()
        );

        let merkle_root_ret = public_values.read::<MerkleTreeHash>();
        let leaf_indices_ret = public_values.read::<Vec<usize>>();
        let inputs_hash = public_values.read::<Vec<u8>>();
        let outputs_hash = public_values.read::<Vec<u8>>();

        info!("Returned public values:");
        info!("Merkle root: {:?}", merkle_root_ret);
        info!("Leaf indices: {:?}", leaf_indices_ret);
        info!("Inputs hash: {:?}", inputs_hash);
        info!("Outputs hash: {:?}", outputs_hash);
    } else {
        info!("Using the network SP1 prover.");
        let client = ProverClient::builder().network().build();
        info!(
            "Executing the SP1 program. Proving ONNX operator: {:?}",
            node
        );

        let (mut public_values, report) = client.execute(ELF, &stdin).run().unwrap();
        info!(
            "executed program with {} cycles",
            report.total_instruction_count()
        );

        let merkle_root_ret = public_values.read::<MerkleTreeHash>();
        let leaf_indices_ret = public_values.read::<Vec<usize>>();
        let inputs_hash = public_values.read::<Vec<u8>>();
        let outputs_hash = public_values.read::<Vec<u8>>();

        info!("Returned public values:");
        info!("Merkle root: {:?}", merkle_root_ret);
        info!("Leaf indices: {:?}", leaf_indices_ret);
        info!("Inputs hash: {:?}", inputs_hash);
        info!("Outputs hash: {:?}", outputs_hash);

        let (pk, vk) = client.setup(ELF);
        info!("generated keys (setup)");

        let program_hash = client.register_program(&vk, ELF).await?;
        info!("registered program with hash: {:?}", program_hash);

        let proof = client
            .prove(&pk, &stdin)
            .strategy(FulfillmentStrategy::Hosted)
            .skip_simulation(true)
            .plonk()
            .run()
            .unwrap();
        info!("generated proof");

        client.verify(&proof, &vk).expect("verification failed");

        info!("verified proof");
    }

    Ok(())
}
