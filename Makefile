.PHONY: compose
compose:
	wasm-tools compose target/wasm32-wasi/release/http_auth_middleware.wasm -c compose.yml -o service.wasm

.PHONY: build-components
build-components: build-http-component build-auth-component

.PHONY: build-http-component
build-http-component:
	cd demo && cargo component build --release

.PHONY: build-auth-component
build-auth-component:
	cargo component build --release