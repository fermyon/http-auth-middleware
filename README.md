# http-auth-middleware

This repo is an example of how to compose a middleware component with a business logic component.

## Repo structure

The `github-oauth/` directory contains an API for using GitHub oauth in an application. It consists of

1. The `authorize` handler which kicks off the github oauth flow allowing a user to give permissions to a GitHub app
2. The `callback` handler which GitHub uses as the redirect url in the oauth flow. The callback handler is responsible for taking a code from the URL param and exchanging it for authentication token from GitHub for the user.
3. The `authenticate` handler which validates a given access token in an incoming request with the GitHub user API.
4. The `login` handler which returns a login button.

The `example-app/` directory contains a Spin application which consists of one http handler which returns an HTTP response contains `Hello, Fermyon!` in the body.


## Demo instructions

### Pre-requisites

- Add the `wasm32-wasip2` target
  
  ```bash
    rustup target add wasm32-wasip2
  ```

- Install latest [Spin](https://github.com/fermyon/spin)

- Create an OAuth App in your [GitHub Developer Settings](https://github.com/settings/developers). Set the callback URL to `http://127.0.0.1:3000/login/callback`. Accept defaults and input dummy values for the rest of the fields.
    - Save the Client ID
    - Generate a new Client Secret and save that as well

### Build the components and run the demo

> NOTE: The build script `build.sh` in the `Spin.toml` will build both the `example-app` and `github-oauth` projects.

```bash
# Build and run the example
spin up --build -e CLIENT_ID=<YOUR_GITHUB_APP_CLIENT_ID> -e CLIENT_SECRET=<YOUR_GITHUB_APP_CLIENT_SECRET>

# Open http://127.0.0.1:3000/ in a browser
```

### Running with Wasmtime

This component can be universally run by runtimes that support WASI preview 2's [HTTP proxy
world](https://github.com/WebAssembly/wasi-http/blob/main/wit/proxy.wit). For example, it can be
served directly by Wasmtime, the runtime embedded in Spin. First, ensure you have installed the
[Wasmtime CLI](https://github.com/bytecodealliance/wasmtime/releases) with at least version
`v21.0.1`. We will use the `wasmtime serve` subcommand which serves requests to/from a WASI HTTP
component.

- Install [wac](https://github.com/bytecodealliance/wac):

```bash
cargo install wac-cli --locked
```

```bash
export CLIENT_ID=<YOUR_GITHUB_APP_CLIENT_ID> 
export CLIENT_SECRET=<YOUR_GITHUB_APP_CLIENT_SECRET>

# Build the example-app and github-oauth component
spin build

# Compose the example-app with the github-oauth component
wac plug --plug target/wasm32-wasip2/release/example.wasm target/wasm32-wasip2/release/github_oauth.wasm -o service.wasm

# Serve the component on the expected host and port
wasmtime serve service.wasm  -S cli --addr 127.0.0.1:3000
```

### Configuring the callback URL

Instead of using the default callback URL of `http://127.0.0.1:3000/login/callback`, you can configure the URL in an environment variable that is resolved at build time. This is useful in the case that the component is not running locally, rather in a hosted environment such as Fermyon Cloud.

```sh
export AUTH_CALLBACK_URL=http://my-auth-app.fermyon.app/login/callback
export CLIENT_ID=<YOUR_GITHUB_APP_CLIENT_ID> 
export CLIENT_SECRET=<YOUR_GITHUB_APP_CLIENT_SECRET>
cargo build --target wasm32-wasip2 --manifest-path github-oauth/Cargo.toml --release --features compile-time-secrets
spin deploy
```

### Using Runtime Environment Variables

Not all WebAssembly runtimes fully support exporting the [`wasi:cli/environment`](https://github.com/WebAssembly/wasi-cli/blob/main/wit/environment.wit) interface to components. Spin, however, does support this and can load environment variables into a component's environment. Simply pass the environment variables during a `spin up`:
```sh
spin up --build -e CLIENT_ID=<YOUR_GITHUB_APP_CLIENT_ID> -e CLIENT_SECRET=<YOUR_GITHUB_APP_CLIENT_SECRET>
```

To deploy an app to Fermyon Cloud that uses environment variables, you need to [configure them in your `spin.toml`](https://developer.fermyon.com/spin/v2/writing-apps#adding-environment-variables-to-components). Update [the example application manifest](./example/spin.toml) to contain your `CLIENT_ID` and `CLIENT_SECRET` environment variables. Since we do not know the endpoint for our Fermyon Cloud application until after the first deploy, we cannot yet configure the `AUTH_CALLBACK_URL`.

```toml
[component.frontend]
# ...
environment = { CLIENT_ID = "YOUR_GITHUB_APP_CLIENT_ID", CLIENT_SECRET = "YOUR_GITHUB_APP_CLIENT_SECRET" }
```

Now deploy your application.

```sh
$ spin deploy
Uploading github-oauth2-example version 0.1.0 to Fermyon Cloud...
Deploying...
Waiting for application to become ready............. ready
Available Routes:
  example: https://github-oauth2-example-12345.fermyon.app (wildcard)
```

In the example deploy output above, the app now exists at endpoint `https://github-oauth2-example-12345.fermyon.app`. This means our callback URL should be `https://github-oauth2-example-12345.fermyon.app/login/callback`. Configure this in the `spin.toml` with another environment variable:

```toml
[component.frontend]
# ...
environment = { CLIENT_ID = "YOUR_GITHUB_APP_CLIENT_ID", CLIENT_SECRET = "YOUR_GITHUB_APP_CLIENT_SECRET", AUTH_CALLBACK_URL = "https://github-oauth2-example-<HASH>.fermyon.app/login/callback" }
```

Now, redeploy with another `spin deploy`. Be sure to update your GitHub OAuth App to update the callback URL.

This example uses environment variable to import secrets, since that is a ubiquitous interface and enables cross cloud portability of your component. If you are interested in configuring dynamic secrets that are not exposed in text in your `spin.toml` and can be updated with the `spin cloud variables` CLI, see [Spin's documentation on configuring application variables](https://developer.fermyon.com/spin/v2/variables#application-variables).