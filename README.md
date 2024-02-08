# Generate certs:
USE mkcert utility to generate certs and add ca root to trusted roots of system

# How certs and client-validation of certs works
certificate authority (CA) signs a certificate (Csa) with its own CRT (CaCrt), producing End entity cert Ce
Server hosting the TLS app (Sa) gets Csa & its own private key Ksa signed by CA and receives Ce.
Sa uses Ce and Ksa as its TLS encryption credentials.

Clients add CaCrt to their trusted certificates, trusting anything signed by CA.
Clients can now trust any Ce because it was signed by their already known and trusted CA bc of CaCrt.
Now, when a client initiates a connection to Sa, and Sa presents Ce, client trusts Sa because Ce was signed by CA



## Old notes
### \
`openssl req -new -newkey rsa:4096 -x509 -sha256 -days 365 -nodes -out cert.crt -keyout priv.key`
put priv.key and cert.crt in a directory at the top level of this repo named certs/



openssl ecparam -genkey -name prime256v1 -noout -out server-private-key.pem

openssl ec -in server-private-key.pem -pubout -out server-public-key.pem

openssl req -new -x509 -sha256 -key server-private-key.pem -subj "/CN=duckduckgo.com" -out server-certificate.pem


