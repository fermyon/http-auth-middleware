localhost:
	cargo component build --release --manifest-path github-oauth/Cargo.toml --features compile-time-secrets
	cargo component build --release --manifest-path example/Cargo.toml
	wasm-tools compose github-oauth/target/wasm32-wasi/release/github_oauth.wasm -d example/target/wasm32-wasi/release/example.wasm -o example/build/cosmonic.service.wasm
	wasmtime serve service.wasm --addr 127.0.0.1:3000

cosmonic:
	@CLIENT_ID=${CLIENT_ID_COSMONIC}
	@CLIENT_SECRET=${CLIENT_SECRET_COSMONIC}
	@AUTH_CALLBACK_URL=${AUTH_CALLBACK_URL_COSMONIC}
	cargo component build --release --manifest-path github-oauth/Cargo.toml --features compile-time-secrets
	cargo component build --release --manifest-path example/Cargo.toml
	wasm-tools compose github-oauth/target/wasm32-wasi/release/github_oauth.wasm -d example/target/wasm32-wasi/release/example.wasm -o example/build/cosmonic.service.wasm
	cd example && \
	wash claims sign --http_server --http_client --name example build/cosmonic.service.wasm -d build/cosmonic.service_s.wasm && \
	cosmo launch --launch-only

fermyon:
	@CLIENT_ID=${CLIENT_ID_FERMYON}
	@CLIENT_SECRET=${CLIENT_SECRET_FERMYON}
	@AUTH_CALLBACK_URL=${AUTH_CALLBACK_URL_FERMYON}
	cargo component build --release --manifest-path github-oauth/Cargo.toml --features compile-time-secrets
	spin deploy -f example
