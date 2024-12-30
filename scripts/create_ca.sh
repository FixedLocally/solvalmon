#!/bin/bash
mkdir -p ./pki
openssl genrsa -aes256 -out ./pki/mtls_ca.key 4096
openssl req -x509 -new -nodes -key ./pki/mtls_ca.key -sha256 -days 1826 -out ./pki/mtls_ca.crt -subj "/C=XX/ST=StateName/L=CityName/O=Solana Validator/OU=CompanySectionName/CN=Solana Validator CA"