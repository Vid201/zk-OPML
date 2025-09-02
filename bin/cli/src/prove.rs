use alloy::hex::ToHexExt;
use candle_core::Tensor;
use candle_onnx::eval::{get_tensor, simple_eval_one};
use sp1_sdk::{Prover, ProverClient, SP1Stdin, include_elf, network::FulfillmentStrategy};
use std::collections::HashMap;
use tracing::info;
use zkopml_ml::{
    data::{extract_input_data, tensor_hash},
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

    /// Path to the input data file (JSON)
    #[clap(long)]
    pub input_data_path: String,

    /// Index of the ONNX operator to prove
    /// If not provided, the prover will prove all operators
    #[clap(long)]
    pub operator_index: Option<usize>,

    /// Type of SP1 prover
    /// - `cpu`: Use the local/cpu SP1 prover
    ///   - Note: When proving with cpu, this will not actually generate all proofs,
    ///     but will only output the number of cycles for each operator.
    /// - `network`: Use the network SP1 prover
    #[clap(long, default_value = "cpu")]
    pub sp1_prover: SP1Prover,
}

const ELF: &[u8] = include_elf!("zkopml-zk");

pub async fn prove(args: ProveArgs) -> anyhow::Result<()> {
    // Load the model and perform the inference
    info!("Reading the model file from {}", args.model_path);
    let model_path = args.model_path.clone();
    let model = load_onnx_model(&model_path)?;
    info!("Number of ONNX operators: {}", model.num_operators());

    // Create merkle tree from ONNX operators
    info!("Creating a Merkle tree from the model operators.");
    let mut nodes = model.graph().unwrap().node;
    let mut nodes_indices: Vec<usize> = (0..nodes.len()).collect();
    let merkle_tree = ModelMerkleTree::new(nodes.clone(), model.graph().unwrap());
    info!("Merkle root hash: {:?}", merkle_tree.root().encode_hex());

    if let Some(operator_index) = args.operator_index {
        nodes = vec![nodes[operator_index].clone()];
        nodes_indices = vec![operator_index];
    }

    for (node, node_index) in nodes.iter().zip(nodes_indices.iter()) {
        // Create SP1 proof of execution
        let mut stdin = SP1Stdin::new();
        stdin.write(&merkle_tree.root());

        let leaf_indices = vec![*node_index];
        stdin.write(&leaf_indices);

        let leaf_hashes = merkle_tree.leaves_hashes(leaf_indices.clone());
        stdin.write(&leaf_hashes);

        let total_leaves = merkle_tree.total_leaves();
        stdin.write(&total_leaves);

        let merkle_proof: Vec<u8> = merkle_tree.prove(leaf_indices).to_bytes();
        stdin.write(&merkle_proof);

        let mut inputs: HashMap<String, Tensor> = HashMap::new();
        for t in model.graph().clone().unwrap().initializer.iter() {
            let tensor = get_tensor(t, t.name.as_str())?;
            inputs.insert(t.name.to_string(), tensor);
        }
        let input_data =
            extract_input_data(&std::fs::read_to_string(args.input_data_path.clone())?)?;
        model.prepare_inputs(&mut inputs, input_data)?;
        for j in 0..*node_index {
            let node = model.get_node(j).unwrap();
            simple_eval_one(&node, &mut inputs)?;
        }
        let mut input_hashes = HashMap::new();
        for (name, tensor) in inputs.iter() {
            let hash = tensor_hash(tensor);
            input_hashes.insert(name.clone(), hash);
        }
        inputs.retain(|k: &String, _| node.input.contains(k));
        let mut inputs_raw: HashMap<String, Tensor> = HashMap::new();
        for (name, tensor) in inputs.iter() {
            let mut init = false;
            for t in model.graph().unwrap().initializer.iter() {
                if name == &t.name {
                    let mut input_name = name.clone();
                    input_name.push_str("graph_initializer");
                    inputs_raw.insert(input_name, tensor.clone());
                    init = true;
                    break;
                }
            }
            if !init {
                inputs_raw.insert(name.clone(), tensor.clone());
            }
        }
        stdin.write(&inputs_raw);
        stdin.write(&input_hashes);

        stdin.write(&node);

        if args.sp1_prover == SP1Prover::Cpu {
            info!(
                "Using the local/cpu SP1 prover. This will not actually generate all proofs, but will only output the number of cycles for each operator."
            );
            let client = ProverClient::builder().cpu().build();
            info!(
                "Executing the SP1 program. Proving ONNX operator: {:?}",
                node
            );

            let (mut public_values, report) = client.execute(ELF, &stdin).run().unwrap();
            info!(
                "Executed program with {} cycles",
                report.total_instruction_count()
            );

            info!("Raw public values: {:?}", public_values.raw());

            let merkle_root_ret = public_values.read::<MerkleTreeHash>();
            let leaf_indices_ret = public_values.read::<Vec<usize>>();
            let inputs_hash = public_values.read::<[u8; 32]>();
            let outputs_hash = public_values.read::<[u8; 32]>();

            info!("Returned public values:");
            info!("Merkle root: {:?}", merkle_root_ret.encode_hex());
            info!("Leaf indices: {:?}", leaf_indices_ret);
            info!("Inputs hash: {:?}", inputs_hash.encode_hex());
            info!("Outputs hash: {:?}", outputs_hash.encode_hex());

            // let (_, vk) = client.setup(ELF);
            // info!("Generated keys (setup), vk: {:?}", vk.bytes32());
        } else {
            info!("Using the network SP1 prover.");
            let client = ProverClient::builder().network().build();
            info!(
                "Executing the SP1 program. Proving ONNX operator: {:?}",
                node
            );

            let (mut public_values, report) = client.execute(ELF, &stdin).run().unwrap();
            info!(
                "Executed program with {} cycles",
                report.total_instruction_count()
            );

            info!("Raw public values: {:?}", public_values.raw());

            let merkle_root_ret = public_values.read::<MerkleTreeHash>();
            let leaf_indices_ret = public_values.read::<Vec<usize>>();
            let inputs_hash = public_values.read::<[u8; 32]>();
            let outputs_hash = public_values.read::<[u8; 32]>();

            info!("Returned public values:");
            info!("Merkle root: {:?}", merkle_root_ret.encode_hex());
            info!("Leaf indices: {:?}", leaf_indices_ret);
            info!("Inputs hash: {:?}", inputs_hash.encode_hex());
            info!("Outputs hash: {:?}", outputs_hash.encode_hex());

            let (pk, vk) = client.setup(ELF);
            info!("Generated keys (setup)");

            let program_hash = client.register_program(&vk, ELF).await?;
            info!("Registered program with hash: {:?}", program_hash);

            let proof = client
                .prove(&pk, &stdin)
                .strategy(FulfillmentStrategy::Hosted)
                .plonk()
                .run()
                .unwrap();
            info!("Generated proof");

            let proof_bytes = proof.bytes();
            info!("Proof: 0x{}", proof_bytes.encode_hex());

            client.verify(&proof, &vk).expect("verification failed");

            info!("Verified proof");
        }
    }

    Ok(())
}
