# Socks-5 Proxy Adapter (Unauthenticated to Authenticated)

The Firefox webdriver does not support connecting to username-password authenticated socks5 proxies. This adapter creates a local socks5 proxy that authenticates with the target & forwards all traffic.

Resources I read to create this project: [wikipedia](https://en.wikipedia.org/wiki/SOCKS).

## Usage

If you compile the project into a bin, you can run the following command to start the adapter:
```
your_bin <bind_address>:<bind_port> <remote_address>:<remote_port> <username> <password>
```

or if you just want to run the project with cargo:
```
cargo run -- <bind_address>:<bind_port> <remote_address>:<remote_port> <username> <password>
```
