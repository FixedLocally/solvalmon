# PKI for solvalmon
`solvalmon` uses mTLS for authentication. For normal operation, the following files are needed inside the `pki` directory:
- `mtls_ca.crt` - the mTLS CA.
- `tls.crt` - the cert that `solvalmon` uses to encrypt communications.
- `tls.key` - the private key for `tls.crt`.

`mtls_ca.key` combined with the mTLS CA can issue certs that enable access to APIs that `solvalmon` expose. Keep it securely and don't upload it to the validator node.