todo:
- handle validator down (status)
- use external rpc (stats)
- set identity
- nonce in auth

auth header: x-api-key
GET: admin_key.sign("{method} {path}")
POST: admin_key.sign("{method} {path} {sha256_hex(payload)}")