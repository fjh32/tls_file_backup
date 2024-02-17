# Generate certs:
USE [mkcert](https://github.com/FiloSottile/mkcert) utility to generate certs and add ca root to trusted roots of system.
In public environment, use certificates from letsencrypt on my server, 
but make sure client code works with it using `rustls_native_certs::load_native_certs()`. 
Client may need the WebPKI native cert authorities (firefox uses them, so im sure they have letencrypt CA).


# After you're sure you have working certs;
# Build
`cargo build` `cargo build --release` for release builds
# Running the exes (devel - non release)
`cargo run --bin server -- --ip "0.0.0.0" --port 4545 --cert "<absolute_path_to_cert_file>" --key "<absolute_path_to_key_file>"`

`cargo run --bin client -- --ip "<server_ip_hostname_certdomain>" --port 4545 --file <absolute_path_of_file_to_send>`

# Running release builds
Pass the same params as above to the exes



# How certs and client-validation of certs works
certificate authority (CA) signs a certificate (Csa) with its own CRT (CaCrt), producing End entity cert Ce.
Server hosting the TLS app (Sa) gets Csa & its own private key Ksa signed by CA and receives Ce.
Sa uses Ce and Ksa as its TLS encryption credentials.

Clients add CaCrt to their trusted certificates, trusting anything signed by CA.
Clients can now trust any Ce because it was signed by their already known and trusted CA bc of CaCrt.
Now, when a client initiates a connection to Sa, and Sa presents Ce, client trusts Sa because Ce was signed by CA



## Old notes
### 

run server with env variable `RUST_LOG=trace cargo run --bin server` to view all logging

`openssl req -new -newkey rsa:4096 -x509 -sha256 -days 365 -nodes -out cert.crt -keyout priv.key`
put priv.key and cert.crt in a directory at the top level of this repo named certs/



openssl ecparam -genkey -name prime256v1 -noout -out server-private-key.pem

openssl ec -in server-private-key.pem -pubout -out server-public-key.pem

openssl req -new -x509 -sha256 -key server-private-key.pem -subj "/CN=duckduckgo.com" -out server-certificate.pem

Create garbage test file: `dd if=/dev/urandom of=random.img count=1024 bs=1M` or `fallocate -l 1G example.file`

# status
[x] TLS connection between client and server (w/ trusted CA on client explained above)

[x] Streaming messages between client and server

[X] Cleanup and abstract

[ ] Unit test `file_backup_service::connection::Connection` struct

[ ] JSON configs
    - server output dir
    - file output format, eg append <datetime> to filename
    - server: overwrite existing y/n? Keep x amount of copies?

[ ] tar.gz client input if directory

[-] Logging
    - timing of file transfers

[ ] Client "basic auth"
    - b/c this is over TLS, it's safe for the client to send a password over the network for the server to verify

[ ] Custom Errors

[X] Command line args
    - client input file


