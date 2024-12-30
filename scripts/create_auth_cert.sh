#!/bin/bash
openssl req -new -nodes -out pki/mtls_auth.csr -newkey rsa:4096 -keyout pki/mtls_auth.key -subj "/CN=Auth/C=AT/ST=Vienna/L=Vienna/O=MyOrg"
openssl x509 -req -in pki/auth.csr -extfile pki/v3.ext -CA pki/mtls_ca.crt -CAkey pki/mtls_ca.key -CAcreateserial -out pki/mtls_auth.crt -days 730 -sha256