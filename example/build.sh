cargo component build --release
wasm-tools compose ../target/wasm32-wasi/release/http_auth_middleware.wasm -d target/wasm32-wasi/release/example.wasm -o service.wasm