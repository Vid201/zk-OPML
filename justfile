set fallback := true

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

deploy verbosity="-vv" eth_rpc="http://127.0.0.1:32002" deployer="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" owner="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80":
	./target/release/zkopml-cli deploy \
		--eth-node-address {{eth_rpc}} \
		--deployer-key {{deployer}} \
		--owner-key {{owner}} \
		{{verbosity}}

register verbosity="-vv" eth_rpc="http://127.0.0.1:32002" user="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" model="./testdata/nanoGPT/network.onnx":
	./target/release/zkopml-cli register \
		--eth-node-address {{eth_rpc}} \
		--user-key {{user}} \
		--model-path {{model}} \
		{{verbosity}}
