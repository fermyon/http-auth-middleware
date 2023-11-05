# http-auth-middleware

This repo is an example of how to compose a middleware component with a business logic component.

## Repo structure

The `github-oauth/` directory contains an API for using GitHub oauth in an application. It consists of

1. The `authorize` handler which kicks off the github oauth flow allowing a user to give permissions to a GitHub app
2. The `callback` handler which GitHub uses as the redirect url in the oauth flow. The callback handler is responsible for taking a code from the URL param and exchanging it for authentication token from GitHub for the user.
3. The `authenticate` handler which validates a given access token in an incoming request with the GitHub user API.
4. The `login` handler which returns a login button.

The `example/` directory contains a Spin application which consists of one http handler which returns an HTTP response contains `Hello, Fermyon!` in the body. In the `spin.toml` file, the component build instructions point to a `build.sh` script which builds the example component and composes it with the github-oauth component.


## Demo instructions

### Pre-requisites

- Install [cargo component](https://github.com/bytecodealliance/cargo-component):

```bash
cargo install --git https://github.com/bytecodealliance/cargo-component cargo-component
```

- Install a fork of [wasm-tools](https://github.com/dicej/wasm-tools/tree/wasm-compose-resource-imports): 

```bash
cargo install --git https://github.com/dicej/wasm-tools --branch wasm-compose-resource-imports wasm-tools --locked
```

- Install latest [Spin](https://github.com/fermyon/spin)

- Create an OAuth App in your [GitHub Developer Settings](https://github.com/settings/developers). Set the callback URL to `http://127.0.0.1:3000/login/callback`. Accept defaults and input dummy values for the rest of the fields.
    - Save the Client ID
    - Generate a new Client Secret and save that as well

### Build the components and run the demo

```bash

# Build the middleware
cargo component build --manifest-path github-oauth/Cargo.toml --release

# Build and run the example
spin up --build -f example -e CLIENT_ID=<YOUR_GITHUB_APP_CLIENT_ID> -e CLIENT_SECRET=<YOUR_GITHUB_APP_CLIENT_SECRET>

# Open http://127.0.0.1:3000/login in a browser
```

### Running with Wasmtime

This component can be universally run by runtimes that support WASI preview 2's [HTTP proxy
world](https://github.com/WebAssembly/wasi-http/blob/main/wit/proxy.wit). For example, it can be
served directly by Wasmtime, the runtime embedded in Spin. First, ensure you have installed the
[Wasmtime CLI](https://github.com/bytecodealliance/wasmtime/releases) with at least version
`v14.0.3`. We will use the `wasmtime serve` subcommand which serves requests to/from a WASI HTTP
component.

Unfortunately, `wasmtime serve` does not currently support setting environment variables in
component, so we cannot pass environment variables at runtime as we did with Spin. Instead, set the
`CLIENT_ID` and `CLIENT_SECRET` oauth app secrets generated in the [prerequisites](#pre-requisites)
step as environment variables and build the oauth component with the `compile-time-secrets` feature
flag. The flag ensures the environment variables are set in the component at compile time so they
are no longer needed from the WebAssembly runtime.

```bash
export CLIENT_ID=<YOUR_GITHUB_APP_CLIENT_ID> 
export CLIENT_SECRET=<YOUR_GITHUB_APP_CLIENT_SECRET>
cargo component build --manifest-path github-oauth/Cargo.toml --release --features compile-time-secrets
# Compose the auth component with the business logic component using wasm-tools
cd example && ./build.sh
# Serve the component on the expected host and port
wasmtime serve service.wasm --addr 127.0.0.1:3000
```

### Configuring the callback URL

Instead of using the default callback URL of `http://127.0.0.1:3000/login/callback`, you can configure the URL in an environment variable that is resolved at build time. This is useful in the case that the component is not running locally, rather in a hosted environment such as Fermyon Cloud.

```sh
export AUTH_CALLBACK_URL=https://http-auth-app.fermyon.app/login/callback
export CLIENT_ID=<YOUR_GITHUB_APP_CLIENT_ID> 
export CLIENT_SECRET=<YOUR_GITHUB_APP_CLIENT_SECRET>
cargo component build --manifest-path github-oauth/Cargo.toml --release --features compile-time-secrets
spin deploy -f example 
```
