mkdir -p p3

cargo build --release --manifest-path github-oauth/Cargo.toml --target wasm32-wasip1
wasm-tools component new --skip-validation ./target/wasm32-wasip1/release/github_oauth.wasm -o ./p3/github_oauth.wasm --adapt ./adapter/wasi_snapshot_preview1.reactor.wasm

cargo build --release --manifest-path example-app/Cargo.toml --target wasm32-wasip1
wasm-tools component new --skip-validation ./target/wasm32-wasip1/release/example_app.wasm -o ./p3/example_app.wasm --adapt ./adapter/wasi_snapshot_preview1.reactor.wasm

wasm-tools compose --skip-validation ./p3/github_oauth.wasm -c ./compose.yaml -o ./p3/authed_app.wasm
