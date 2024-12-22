set fallback := true

registry := "0x5fbdb2315678afecb367f032d93f642f64180aa3"

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

deploy verbosity="-v" eth_rpc="http://127.0.0.1:32002" deployer="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" owner="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80":
	./target/release/zkopml-cli deploy \
		--eth-node-address {{eth_rpc}} \
		--deployer-key {{deployer}} \
		--owner-key {{owner}} \
		{{verbosity}}

register verbosity="-v" eth_rpc="http://127.0.0.1:32002" user="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" model="./testdata/nanoGPT/network.onnx":
	./target/release/zkopml-cli register \
		--eth-node-address {{eth_rpc}} \
		--model-registry-address {{registry}} \
		--user-key {{user}} \
		--model-path {{model}} \
		{{verbosity}}

request verbosity="-v" eth_rpc="http://127.0.0.1:32002" user="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" id="0" input_data="./testdata/nanoGPT/input.json":
	./target/release/zkopml-cli request \
		--eth-node-address {{eth_rpc}} \
		--model-registry-address {{registry}} \
		--user-key {{user}} \
		--model-id {{id}} \
		--input-data-path {{input_data}} \
		{{verbosity}}

submit verbosity="-v" eth_rpc="http://127.0.0.1:32002" user="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" id="0" model="./testdata/nanoGPT/network.onnx":
	./target/release/zkopml-cli submit \
		--eth-node-address {{eth_rpc}} \
		--model-registry-address {{registry}} \
		--user-key {{user}} \
		--model-id {{id}} \
		--model-path {{model}} \
		{{verbosity}}
