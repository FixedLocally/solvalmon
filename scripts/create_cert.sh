#!/bin/bash
if [ -z "$1" ]; then
  echo "Usage: $0 <public-ip>"
  exit 1
fi
openssl req -x509 -newkey rsa:4096 -keyout pki/tls.key -out pki/tls.crt -sha256 -days 3650 -nodes -subj "/C=XX/ST=StateName/L=CityName/O=Solana Validator/OU=CompanySectionName/CN=$1"