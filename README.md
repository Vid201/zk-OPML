# zkopML

Zero-knowledge Optimistic Machine Learning

## Project description

The aim of this project is to combine zkML and opML approaches for better efficiency and performance than relying solely on a single approach.

High-level idea: ML outputs are optimistically accepted as correct, but anyone can challenge the outcome if they think the output is wrong. ML model is made of the sequence of ONNX operators. opML is used to find the dispute point (which ONNX operation is wrong), whereas zkML is used to verify the execution of this ONNX operator (or several ONNX operators, based on the predefined parameter N).

## Project structure

The project has the following folders:

- `contracts`: smart contract for fault proof game and zkVM proof verification
- `crates`: common code and libs
- `bin`: binary files and cli

## Prerequisites

1. [rust](https://www.rust-lang.org/tools/install)
2. [just](https://just.systems/man/en/)
3. [docker](https://www.docker.com/)
4. [kurtosis](https://www.kurtosis.com/)

## Devnet usage

## License

## Paper

## Acknowledgments

## Contact
