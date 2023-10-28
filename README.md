# http-auth-middleware

```
# Build the middleware
cargo component build --release
# Build and run the example
spin up --build -f example
# ... In a new terminal ...
curl -b "token=YOUR TOKEN" localhost:3000
Hello, Fermyon!
```