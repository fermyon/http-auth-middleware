cargo component build --release
wasm-tools compose ../github-oauth/target/wasm32-wasi/release/github_oauth.wasm -d target/wasm32-wasi/release/example.wasm -o service.wasm