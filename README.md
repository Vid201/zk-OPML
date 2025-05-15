# (ZK+OP)ML

Zero-knowledge Optimistic Machine Learning (ZK+OP)ML

## Project description

The aim of this project is to combine ZKML and OPML approaches for better efficiency and performance than relying solely on a single approach.

High-level idea: ML outputs are optimistically accepted as correct, but anyone can challenge the outcome if they think the output is wrong. ML model is made of the sequence of ONNX operators. Fault Dispute Game (FDG, part of OPML) is used to find the dispute point (which ONNX operation is wrong), whereas ZKML (ZKVM) is used to verify the execution of this ONNX operator (or several ONNX operators, based on the predefined parameter N).

## Project structure

The project has the following folders:

- `contracts`: smart contracts for fault proof game and ZKVM on-chain proof verification
- `crates`: common code and libs
- `bin`: source code for binary files and CLI
- `testdata`: test models and data for development and testing

## Prerequisites

1. [rust](https://www.rust-lang.org/tools/install)
2. [just](https://just.systems/man/en/)
3. [docker](https://www.docker.com/)
4. [sp1](https://docs.succinct.xyz/docs/introduction)

## Available ML Models

The following ML models are available in the `testdata` directory:

- `variable_cnn/network.onnx`: Variable CNN model
- `t5/network.onnx`: T5 transformer model
- `gte/network.onnx`: GTE embedding model
- `whisper/network.onnx`: Whisper speech recognition model

To use a specific model, update the `model` variable in the `justfile` by uncommenting the desired model path.

## Devnet usage

To start the development network (Ethereum node and IPFS), run:

```bash
just setup-network
```

To deploy the necessary smart contracts, follow these steps:

```bash
just deploy-create2 # create2
just deploy-sp1-verifier # sp1 verifier
just deploy # ML model registry and fault proofs
```

To register an ML model to the registry, run:

```bash
just register
```

To test the fault proof game, you need to open three terminal windows and run all three actors in the following order:

1. First terminal (Submitter): Run `just submit` or `just submit-defect` to submit a correct result or defect
2. Second terminal (Verifier): Run `just verify` to verify the submitted proofs
3. Third terminal (User/Requester): Run `just request` to request ML inference

| Actor | Command | Description |
|-------|---------|-------------|
| User/Requester | `just request` | Requests ML inference from the model registry |
| Submitter | `just submit` or `just submit-defect` | Submits a proof or defect to the fault proof contract |
| Verifier | `just verify` | Verifies the submitted proof or defect |

> **Note**: Even though testing is performed on the local Ethereum devnet, the SP1 ZKVM proof generation requests are sent to the [Succinct Prover Network](https://docs.succinct.xyz/docs/network/introduction). To use this service, you need to configure your `.env` file with the following variables:
>
> ```bash
> NETWORK_RPC_URL=<your-rpc-url>
> NETWORK_PRIVATE_KEY=<your-private-key>
> ```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Paper

TODO

## Acknowledgments

- [Alloy](https://github.com/alloy-rs/alloy) - Rust Ethereum library
- [SP1](https://github.com/succinctlabs/sp1) - Zero-knowledge virtual machine
- [OPML](https://github.com/ora-io/opml) - Optimistic machine learning
- [EZKL](https://github.com/zkonduit/ezkl) - Zero-knowledge machine learning
- [Candle ONNX](https://github.com/huggingface/candle/tree/main/crates/candle-onnx) - ONNX runtime for Rust

## Contact

- Twitter: [@vidkersic](https://twitter.com/vidkersic)
- Email: [vid.kersic@yahoo.com](mailto:vid.kersic@yahoo.com)
