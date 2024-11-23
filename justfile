set fallback := true

# default recipe to display help information
default:
	@just --list

build:
	cargo build --release

run:
	./target/release/cli