set fallback := true
set dotenv-load

verbosity := "" # "-v"
eth_rpc := "ws://127.0.0.1:8546"
deployer_address := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
owner_address := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
user_address := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
sp1_verifier_smart_contract := "0x61EEd5eE968506eB27320FD776Fe14E4842b1990"
challenge_window := "5000"
response_window := "10"

# default recipe to display help information
default:
	@just --list

build:
	cargo build --profile release-client-lto

format:
	cargo fmt --all

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
	FOUNDRY_PROFILE=deploy forge script ./script/deploy/SP1VerifierGatewayPlonk.s.sol:SP1VerifierGatewayScript --private-key {{deployer_address}} --multi --broadcast && \
	FOUNDRY_PROFILE=deploy forge script ./script/deploy/v4.0.0-rc.3/SP1VerifierPlonk.s.sol:SP1VerifierScript --private-key {{deployer_address}} --multi --broadcast

deploy-smart-contracts:
	./target/release-client-lto/zkopml-cli deploy \
	--eth-node-address {{eth_rpc}} \
	--deployer-key {{deployer_address}} \
	--owner-key {{owner_address}} \
	--sp1-verifier-address {{sp1_verifier_smart_contract}} \
	--challenge-window {{challenge_window}} \
	--response-window {{response_window}} \
	{{verbosity}}

register:
	./target/release-client-lto/zkopml-cli register \
	--eth-node-address {{eth_rpc}} \
	--model-registry-address ${MODEL_REGISTRY_SMART_CONTRACT} \
	--user-key {{user_address}} \
	--model-path ${MODEL_PATH} \
	{{verbosity}}

request model_id:
	./target/release-client-lto/zkopml-cli request \
	--eth-node-address {{eth_rpc}} \
	--model-registry-address ${MODEL_REGISTRY_SMART_CONTRACT} \
	--model-path ${MODEL_PATH} \
	--input-data-path ${INPUT_DATA_PATH} \
	--user-key {{user_address}} \
	--model-id {{model_id}} \
	{{verbosity}}

submit model_id:
	./target/release-client-lto/zkopml-cli submit \
	--eth-node-address {{eth_rpc}} \
	--model-registry-address ${MODEL_REGISTRY_SMART_CONTRACT} \
	--fault-proof-address ${FDG_SMART_CONTRACT} \
	--user-key {{user_address}} \
	--model-id {{model_id}} \
	--model-path ${MODEL_PATH} \
	{{verbosity}}

submit-defect model_id operator_index:
	./target/release-client-lto/zkopml-cli submit \
	--eth-node-address {{eth_rpc}} \
	--model-registry-address ${MODEL_REGISTRY_SMART_CONTRACT} \
	--fault-proof-address ${FDG_SMART_CONTRACT} \
	--user-key {{user_address}} \
	--model-id {{model_id}} \
	--model-path ${MODEL_PATH} \
	--defect \
	{{verbosity}}

verify model_id:
	SP1_PROVER=network NETWORK_RPC_URL=${NETWORK_RPC_URL} NETWORK_PRIVATE_KEY=${NETWORK_PRIVATE_KEY} \
	./target/release-client-lto/zkopml-cli verify \
	--eth-node-address {{eth_rpc}} \
	--model-registry-address ${MODEL_REGISTRY_SMART_CONTRACT} \
	--fault-proof-address ${FDG_SMART_CONTRACT} \
	--user-key {{user_address}} \
	--model-id {{model_id}} \
	--model-path ${MODEL_PATH} \
	{{verbosity}}

prove-local:
	./target/release-client-lto/zkopml-cli prove \
	--model-path ${MODEL_PATH} \
	--sp1-prover cpu \
	{{verbosity}}

prove-network operator_index:
	SP1_PROVER=network NETWORK_RPC_URL=${NETWORK_RPC_URL} NETWORK_PRIVATE_KEY=${NETWORK_PRIVATE_KEY} \
	./target/release-client-lto/zkopml-cli prove \
	--model-path ${MODEL_PATH} \
	--operator-index {{operator_index}} \
	--sp1-prover network \
	{{verbosity}}
