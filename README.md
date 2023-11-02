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

- Create a [GitHub App](https://github.com/settings/apps/new). The callback URL should be `http://127.0.0.1:3000/login/callback`. Accept defaults and input dummy values for the rest of the fields.
    - Save the Client ID
    - Generate a new Client Secret and save that as well

- In `example/spin.toml`: replace `<YOUR GITHUB CLIENT ID>` with the Client ID from your GitHub APP and `<YOUR GITHUB CLIENT SECRET>` with the Client Secret from your GitHub App

### Build the components and run the demo

```

# Build the middleware
cargo component build --manifest-path github-oauth/Cargo.toml --release

# Build and run the example
spin up --build -f example

# Open http://127.0.0.1:3000/login in a browser
```
