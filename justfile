set fallback := true

verbosity := "" # "-v"
eth_rpc := "ws://127.0.0.1:32003"
deployer := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
owner := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
user := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
model := "./testdata/nanoGPT/network.onnx"
input_data := "./testdata/nanoGPT/input.json"
input_shape := "1,64"
output_shape := "64,65"
registry := "0x5fbdb2315678afecb367f032d93f642f64180aa3"
model_id := "0"

# default recipe to display help information
default:
	@just --list

build:
	cargo build --release

format:
	cargo fmt --all

kurtosis-up:
	kurtosis run github.com/ethpandaops/ethereum-package --args-file kurtosis.yaml > kurtosis.log

kurtosis-down:
	kurtosis clean -a

ipfs-up:
	docker compose up -d

ipfs-down:
	docker compose down

deploy:
	./target/release/zkopml-cli deploy \
		--eth-node-address {{eth_rpc}} \
		--deployer-key {{deployer}} \
		--owner-key {{owner}} \
		{{verbosity}}

register:
	./target/release/zkopml-cli register \
		--eth-node-address {{eth_rpc}} \
		--model-registry-address {{registry}} \
		--user-key {{user}} \
		--model-path {{model}} \
		--input-shape {{input_shape}} \
		--output-shape {{output_shape}} \
		{{verbosity}}

request:
	./target/release/zkopml-cli request \
		--eth-node-address {{eth_rpc}} \
		--model-registry-address {{registry}} \
		--user-key {{user}} \
		--model-id {{model_id}} \
		--input-data-path {{input_data}} \
		{{verbosity}}

submit:
	./target/release/zkopml-cli submit \
		--eth-node-address {{eth_rpc}} \
		--model-registry-address {{registry}} \
		--user-key {{user}} \
		--model-id {{model_id}} \
		--model-path {{model}} \
		--input-shape {{input_shape}} \
		--output-shape {{output_shape}} \
		{{verbosity}}

submit-wrong:
	./target/release/zkopml-cli submit \
		--eth-node-address {{eth_rpc}} \
		--model-registry-address {{registry}} \
		--user-key {{user}} \
		--model-id {{model_id}} \
		--model-path {{model}} \
		--input-shape {{input_shape}} \
		--output-shape {{output_shape}} \
		--defect \
		{{verbosity}}

verify:
	./target/release/zkopml-cli verify \
		--eth-node-address {{eth_rpc}} \
		--model-registry-address {{registry}} \
		--user-key {{user}} \
		--model-id {{model_id}} \
		--model-path {{model}} \
		--input-shape {{input_shape}} \
		--output-shape {{output_shape}} \
		{{verbosity}}

prove:
	./target/release/zkopml-cli prove \
		--model-path {{model}} \
		--input-data-path {{input_data}} \
		--input-shape {{input_shape}} \
		--output-shape {{output_shape}} \
		-vv
