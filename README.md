# solvalmon
`solvalmon` is a monitoring software for the Solana validator (Agave/Jito).

## Building
```
cargo build --release
```

## Operation modes & Configuration
`solvalmon` can operate in two modes, `monitor` and `sentry`. The `monitor` mode is for validator nodes and it enables sentries to query the node's state and set identities without exposing the RPC port, as well as setting the node's identity the secondary one if internet connection is lost. The `sentry` mode polls the status from configured nodes and a configurable external RPC to determine if the node is operating properly, and failover nodes if necessary.

Example configurations are provided. Edit `config.json` for the `monitor` mode and `sentry.json` for the `sentry` mode.

## Authentication for solvalmon
`solvalmon` uses mTLS for authentication and the required files must be placed inside the `pki` directory relative to the current working directory.

In the `monitor` mode, the following files are needed:
- `mtls_ca.crt` - the mTLS CA.
- `tls.crt` - the cert that `solvalmon` uses to encrypt communications.
- `tls.key` - the private key for `tls.crt`.

`mtls_ca.key` combined with the mTLS CA can issue certs that enable access to APIs that `solvalmon` expose. Keep it securely and don't upload it to the validator node.

In the `sentry` mode, the mTLS cert's location can be configured.