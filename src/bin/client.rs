use std::fs::File;
use std::io::{self, Read};
use std::io::BufReader;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use rustls::pki_types::{CertificateDer, UnixTime};
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio::io::{copy, split, stdin as tokio_stdin, stdout as tokio_stdout, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified};

use file_backup_service::common;
use file_backup_service::connection;


// const HOST_ADDR: &str = "127.0.0.1:4545";
// const HOST_IP: &str = "127.0.0.1";
const HOST_ADDR: &str = "192.168.1.110:4545";
const HOST_IP: &str = "192.168.1.110";
const CERT: &str = "/home/frank/certs/ripplein.space-dev.pem";


#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = HOST_ADDR
        .to_string()
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;

    
    let mut root_cert_store = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
        // println!("Got a cert");
        root_cert_store.add(cert).unwrap();
    }


    // USE this to include CA crt file with which to accept anyone's cert the CA has signed
    // println!("OPENING CERT FILE {}", CERT);
    // let mut pem = BufReader::new(File::open(CERT)?);
    // for cert in rustls_pemfile::certs(&mut pem) {
    //     let cert = match cert {
    //         Ok(cert) => {println!("Got a cert"); cert },
    //         Err(_) => {println!("Err occurred "); break; }
    //     };

    //     root_cert_store.add(cert).unwrap();
    // }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let tls_connector = TlsConnector::from(Arc::new(config));
    let sock_stream = TcpStream::connect(&addr).await?;
    let ip_addr = ServerName::try_from(HOST_IP).unwrap();
    let mut tls_stream = match tls_connector.connect(ip_addr, sock_stream).await {
        Ok(tls) => tls,
        Err(_e) => {
            println!("{}", _e);
            panic!("FAILED TO CONNECT") 
        }
    };


    // let mut conn = file_backup_service::connection::ClientConnection::new(tls_stream);


    match file_backup_service::connection::write_message_from_string(&mut tls_stream, String::from("HELLO SERVER FROM CLIENTv2")).await {
        Ok(_) => println!("sending msg to server"),
        Err(_) => {
            panic!("failed msg to server")
        }
    };

    let string = match file_backup_service::connection::read_into_string(&mut tls_stream).await {
        Ok(string) => string,
        Err(_) => {
            panic!("failed read to server")
        }
    };

    println!("Received this from server: {}", string);

    println!("ENDED");
    Ok(())
}