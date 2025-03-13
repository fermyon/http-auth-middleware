mkdir -p p3

cargo +1.90 build --release --manifest-path github-oauth/Cargo.toml --target wasm32-wasip2

cargo +1.90 build --release --manifest-path example-app/Cargo.toml --target wasm32-wasip2

wasm-tools compose --skip-validation ./target/wasm32-wasip2/release/github_oauth.wasm -c ./compose.yaml -o ./p3/authed_app.wasm
