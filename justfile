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
verifier := "0x61EEd5eE968506eB27320FD776Fe14E4842b1990"
registry := "0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9"
fault_proof := "0xdc64a140aa3e981100a9beca4e685f962f0cf6c9"
model_id := "0"
operator_index := "0"
challenge_window := "96000"
response_window := "100"

# default recipe to display help information
default:
	@just --list

build:
	cargo build --release

format:
	cargo fmt --all

# kurtosis-up:
#	kurtosis run github.com/ethpandaops/ethereum-package --args-file kurtosis.yaml > kurtosis.log

# kurtosis-down:
# 	kurtosis clean -a

# ipfs-up:
# 	docker compose up -d

# ipfs-down:
#	docker compose down

setup-network:
	docker compose up -d && \
	sleep 5 && \
	eth_accounts_response=$(curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_accounts","params":[],"id":1}' http://127.0.0.1:8545) && \
	account=$(echo "$eth_accounts_response" | jq -r '.result[0]') && \
	curl -s -X POST -H "Content-Type: application/json" --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_sendTransaction\",\"params\":[{\"from\": \"$account\", \"to\": \"0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266\", \"value\": \"0x56BC75E2D63100000\"}],\"id\":1}" http://127.0.0.1:8545

shutdown-network:
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
		--sp1-verifier-address {{verifier}} \
		--challenge-window {{challenge_window}} \
		--response-window {{response_window}} \
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
		--fault-proof-address {{fault_proof}} \
		--user-key {{user}} \
		--model-id {{model_id}} \
		--model-path {{model}} \
		--input-shape {{input_shape}} \
		--output-shape {{output_shape}} \
		{{verbosity}}

submit-defect:
	./target/release/zkopml-cli submit \
		--eth-node-address {{eth_rpc}} \
		--model-registry-address {{registry}} \
		--fault-proof-address {{fault_proof}} \
		--user-key {{user}} \
		--model-id {{model_id}} \
		--model-path {{model}} \
		--input-shape {{input_shape}} \
		--output-shape {{output_shape}} \
		--defect \
		{{verbosity}}

verify:
	SP1_PROVER=network NETWORK_RPC_URL=${NETWORK_RPC_URL} NETWORK_PRIVATE_KEY=${NETWORK_PRIVATE_KEY} \
	./target/release/zkopml-cli verify \
		--eth-node-address {{eth_rpc}} \
		--model-registry-address {{registry}} \
		--fault-proof-address {{fault_proof}} \
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
