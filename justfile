set fallback := true
set dotenv-load

verbosity := "" # "-v"
eth_rpc := "ws://127.0.0.1:8546"
deployer := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
owner := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
user := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
model := "./testdata/variable_cnn/network.onnx"
input_data := "./testdata/variable_cnn/input.json"
input_shape := "1,1,28,28"
output_shape := "1,1,28,28"
registry := "0x5fbdb2315678afecb367f032d93f642f64180aa3"
verifier := ""
model_id := "0"
operator_index := "4"

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

deploy-create2:
	cd contracts/create2 && \
	./scripts/test.sh

deploy-sp1-verifier:
	cd contracts/foundry/lib/sp1-contracts/contracts && \
	FOUNDRY_PROFILE=deploy forge script ./script/deploy/SP1VerifierGatewayPlonk.s.sol:SP1VerifierGatewayScript --private-key {{deployer}} --multi --broadcast && \
	FOUNDRY_PROFILE=deploy forge script ./script/deploy/v4.0.0-rc.3/SP1VerifierPlonk.s.sol:SP1VerifierScript --private-key {{deployer}} --multi --broadcast

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
		--model-path {{model}} \
		--input-shape {{input_shape}} \
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
		--operator-index {{operator_index}} \
		--sp1-prover cpu \
		{{verbosity}}

prove-network:
		SP1_PROVER=network NETWORK_RPC_URL=${NETWORK_RPC_URL} NETWORK_PRIVATE_KEY=${NETWORK_PRIVATE_KEY} \
		./target/release/zkopml-cli prove \
		--model-path {{model}} \
		--input-data-path {{input_data}} \
		--input-shape {{input_shape}} \
		--output-shape {{output_shape}} \
		--operator-index {{operator_index}} \
		--sp1-prover network \
		{{verbosity}}
