help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_\-.*]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

localhost: ## Build and launch component locally with wasmtime
	cargo component build --release --manifest-path github-oauth/Cargo.toml --features compile-time-secrets
	cargo component build --release --manifest-path example/Cargo.toml
	wasm-tools compose github-oauth/target/wasm32-wasi/release/github_oauth.wasm -d example/target/wasm32-wasi/release/example.wasm -o example/build/cosmonic.service.wasm
	wasmtime serve service.wasm --addr 127.0.0.1:3000

cosmonic: ## Build and launch component on Cosmonic
	@CLIENT_ID=${CLIENT_ID_COSMONIC} \
		CLIENT_SECRET=${CLIENT_SECRET_COSMONIC} \
		AUTH_CALLBACK_URL=https://auth.cosmonic.app/login/callback \
		cargo component build --release --manifest-path github-oauth/Cargo.toml --features compile-time-secrets
	cargo component build --release --manifest-path example/Cargo.toml
	mkdir -p example/build
	wasm-tools compose github-oauth/target/wasm32-wasi/release/github_oauth.wasm -d example/target/wasm32-wasi/release/example.wasm -o example/build/example.wasm
	cd example && \
	wash claims sign -v 0.1.0 --http_server --http_client --name "HTTP Auth" build/example.wasm -d build/example_s.wasm && \
	cosmo launch --launch-only

fermyon: ## Build and launch component on Fermyon Cloud
	@CLIENT_ID=${CLIENT_ID_FERMYON} \
		CLIENT_SECRET=${CLIENT_SECRET_FERMYON} \
		AUTH_CALLBACK_URL=${AUTH_CALLBACK_URL_FERMYON} \
		cargo component build --release --manifest-path github-oauth/Cargo.toml --features compile-time-secrets
	spin deploy -f example
