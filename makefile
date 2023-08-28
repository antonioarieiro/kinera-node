build:
	cargo build

run-local:
	cargo run -- --dev

run:
	cargo run --release -- --dev --port=30338 --ws-port 9948 --ws-external --rpc-cors=all --rpc-methods=Unsafe --rpc-external

runzero:
	cargo run --release -- --dev --port=30334 --ws-port 9944 --ws-external --rpc-cors=all --rpc-methods=Unsafe --rpc-external

purge:
	cargo run -- purge-chain --dev
