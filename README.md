# http-auth-middleware

```
spin -V
spin 2.0.0-pre0 (f05a6ea4 2023-10-23)

spin build -f demo
cargo component build --release
wasm-tools compose target/wasm32-wasi/release/http_auth_middleware.wasm -d demo/target/wasm32-wasi/release/http_handler.wasm
```