# zk-OPML

This repository contains the source code for the article "zk-OPML: Using Zero-Knowledge Proofs to Optimize OPML." zk-OPML is a novel method for achieving verifiability of machine learning model inference, leveraging crypto-economic and cryptographic principles. The repository includes everything needed to set up a local environment and reproduce the experiments presented in the article.

## Method Description

zk-OPML builds on the principles of Optimistic Machine Learning (OPML) and Zero-Knowledge Machine Learning (ZKML), combining the strengths of both approaches to achieve greater efficiency, scalability, and performance. By leveraging the interactive verification of OPML together with the cryptographic guarantees of ZKML, the protocol introduces a hybrid model that balances practical feasibility with strong security assurances.

At a high level, the protocol works as follows: a user submits a machine learning inference request through a smart contract call, and submitters respond by providing inference results. Challengers monitor these results and can raise a dispute if they believe an output is incorrect. Once a challenge is opened, the Fault Dispute Game (FDG) is played between the submitter and challenger over the sequence of ONNX operators that define the ML model. The binary search process narrows the disagreement to a specific ONNX operator (or group of operators). At this point, a zero-knowledge proof (ZKP) is generated for the disputed operator’s execution and verified on-chain, ensuring that the computation was performed correctly.

## Project Structure

The project is structured in the following way:

- `contracts`: smart contracts for the ML model registry, fault disoute game (FDG), and library for SP1 ZKVM on-chain verifier
- `crates`: common code in Rust, SP1 ZKVM program
- `bin`: source code for binaries, CLI
- `testdata`: test ML models and data for development and testing
- `notebooks`: Jupyter Notebooks for ZKML/EZKL experiments

## Prerequisites

1. [Rust v1.88.0](https://www.rust-lang.org/tools/install)
2. [Python 3.10.12](https://www.python.org/)
3. [just](https://just.systems/man/en/)
4. [Docker](https://www.docker.com/)
5. [SP1 ZKVM](https://docs.succinct.xyz/docs/sp1/introduction)
6. [Foundry](https://getfoundry.sh/)
7. [Samply](https://github.com/mstange/samply)

## Usage

First, build the code:

```bash
just build
```

Before starting the zk-OPML, we need to set a few environment variables. Even though testing is performed on the local Ethereum devnet, the SP1 ZKVM proof generation requests are sent to the [Succinct Prover Network](https://docs.succinct.xyz/docs/sp1/prover-network/quickstart). For more information on how to use the prover network, check the provided link. You need to configure your `.env` file with the following variables (check `.env.example`):

```bash
# ML model
MODEL_PATH=<path-to-the-testing-ml-model> # check the folder testdata for all ML models that were tested; this model will be used everywhere
INPUT_DATA_PATH=<path-to-the-input-data>

# SP1 verifier smart contracts (variables needed to deploy local SP1 smart contract verifier)
CHAINS=DEV
RPC_DEV=http://127.0.0.1:8545

# Smart contracts
MODEL_REGISTRY_SMART_CONTRACT=<model-registry-smart-contract-address> # check the output of just deploy-smart-contracts
FDG_SMART_CONTRACT=<fdg-smart-contract-address> # check the output of just deploy-smart-contracts

# SP1 prover network
NETWORK_PRIVATE_KEY=<private-key-for-succinct-prover-network>
NETWORK_RPC_URL=<rpc-url-for-succinct-prover-network>
```

Then we need to setup the development environment (local Ethereum network and IPFS) and deploy all needed smart contracts:

```bash
just setup-network # ethereum, IPFS
just deploy-create2 # create2 smart contract
just deploy-sp1-verifier # SP1 on-chain verifier
just deploy-smart-contracts # ML model registry, FDG smart contract
```

After everything is set up, we can first start with registering the ML model to the model registry smart contract:

```bash
just register
```

To test the fault proof game, you need to open three terminal windows and run all three participating entities in the following order:

| **Actor**        | **Command**                                   | **Command Parameters**           | **Description**                                         |
|:-----------------|:----------------------------------------------|:---------------------------------|:--------------------------------------------------------|
| Submitter        | `just submit 0` or `just submit-defect 0 2`   | `0`: `model id`, `2`: `defect operator` | Responds with ML inference                              |
| Verifier         | `just verify 0`                               | `0`: `model id`                    | Verifies the ML inference and can create a challenge     |
| User/Requester   | `just request 0`                              | `0`: `model id`                    | Requests ML inference                                   |

> **Notes:**  
> The `model id` is an incremental counter assigned to each registered model. The first registered model receives `model id` 0, the next one 1, and so on.
> The `defect operator` refers to the ONNX operator index where the submitter intentionally corrupts the inference (for testing purposes), allowing the verifier to create a successfull challenge.

![Terminal Example](assets/terminal.png)

To shutdown the development environment:

```bash
just shutdown-network
```

## ML Models

The following ML models are available in the `testdata` directory:

- `testdata/xg_boost`: XGBoost
- `testdata/lenet_5`: LeNet
- `testdata/2d_conv`: Neural network with 2-dimensional convolution and complex ONNX operators
- `testdata/mobilenet`: MobileNet

To use a specific model, set the variable `MODEL_PATH` in the `.env` to the location of the model's ONNX file.

## Results

| Model Name |  ONNX operators |  Number of ONNX operators | Number of parameters | Size (MB)  | zk-OPML time | EZKL proving time (witness generation and proving) |
|----------|----------|----------|----------|----------|----------|----------|
|   XGBoost  | 15 x Constant, 11 x Reshape, 10 x Gather, 5 x Add, 5 x Cast, 5 x Less, 4 x GatherElements, 4 x Mul, 1 x ReduceSum, 1 x Softmax |  62  |   3,420  |   0.03  |  120 s + 360 s + 118 s = 598 s  |  0.04 s + 18.71 s = 18.75 s  |
|   LeNet  |  3 x Gemm, 2 x Add, 2 x AveragePool, 2 x Conv, 2 x Mul, 1 x Flatten | 12  |   61,706  |   0.24  |  120 s + 250 s + 134 s = 494 s  |   0.40 s + 35.97 s = 36.37 s  |
|  2d conv   | 12 x Relu, 10 x Conv, 5 x MaxPool, 3 x Gemm, 1 x Flatten |  31  |   54,584  |   0.21  |   120 s + 300 s + 146 s = 566 s  |   2.11 s + 253.92 s = 256.03 s  |
|   MobileNet  |  54 x Conv, 53 x BatchNormalization, 36 x Relu, 10 x Add, 1 x GlobalAveragePool, 1 x Reshape |  155  |   3,539,138  |  13.6  |  120 s + 480 s + 515 s = 1115 s  | 351.53 s + / = /  |

> **Note:** The time for zk-OPML was calculated as: zk-OPML time = challenge creation window + 2 × log₂(number of ONNX operators) × response window + SP1 ZKVM proving

> **Note:** EZKL ZK proving was not possible for MobileNet due to very high RAM requirements.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Article

TODO

## Acknowledgments

- [Alloy](https://github.com/alloy-rs/alloy) - Ethereum Rust library
- [Foundry](https://github.com/foundry-rs/foundry) - Ethereum smart contract development and testing
- [SP1](https://github.com/succinctlabs/sp1) - Zero-knowledge virtual machine (ZKVM)
- [OPML](https://github.com/ora-io/opml) - Optimistic machine learning (OPML)
- [EZKL](https://github.com/zkonduit/ezkl) - Zero-knowledge machine learning (ZKML)
- [Candle ONNX](https://github.com/huggingface/candle/tree/main/crates/candle-onnx) - ONNX runtime for Rust

## Author's Contact

- Twitter: [@vidkersic](https://twitter.com/vidkersic)
- Email: [vid.kersic@yahoo.com](mailto:vid.kersic@yahoo.com)
