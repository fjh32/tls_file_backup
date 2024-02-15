use std::io;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;
use rustls::pki_types::{CertificateDer, UnixTime};
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use log::{debug, error, log_enabled, info, Level};
use clap::Parser;
use std::fs::metadata;


use file_backup_service::common;
use file_backup_service::connection;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ClientArgs {
    #[arg(short, long, default_value = "ripplein.space")]
    ip: String,
    #[arg(short, long, default_value_t = 4545)]
    port: i32,
    #[arg(short, long)]
    file: String,
}


#[tokio::main]
async fn main() -> io::Result<()> {
    common::setup_logger();

    let args = ClientArgs::parse();
    let host = common::make_address_str(&args.ip, &args.port);
    info!("Connecting to {}", host);

    let addr = host
        .to_string()
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;

    
    let mut root_cert_store = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
        root_cert_store.add(cert).unwrap();
    }


    // USE this to include CA crt file with which to accept anyone's cert the CA has signed
    // very useful to distribute client with CA cert
    // println!("OPENING CERT FILE {}", CERT);
    // let mut pem = BufReader::new(File::open(CERT)?);
    // for cert in rustls_pemfile::certs(&mut pem) {
    //     let cert = match cert {
    //         Ok(cert) => {println!("Got a cert"); cert },
    //         Err(_) => {println!("Err occurred "); break; }
    //     };

    //     root_cert_store.add(cert).unwrap();
    // }

    let ip_addr = ServerName::try_from(args.ip).unwrap();
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let tls_connector = TlsConnector::from(Arc::new(config));

    let sock_stream = TcpStream::connect(&addr).await?;
    let tls_stream = tls_connector.connect(ip_addr, sock_stream).await?;
    let mut conn = connection::Connection::new(tls_stream);
    info!("TLS connection established with {}", host);


    // let filepath = Path::new(&args.file);
    // let filename = filepath.file_name().
    // let filename_message_to_send = format!("filename:{}:filename", args.file);
    conn.write_message_from_string(filename_message_to_send).await?;

    let string = match conn.read_into_string().await {
        Ok(string) => string,
        Err(_) => {
            panic!("failed read to server")
        }
    };

    info!("Received this from server: {}", string);

    match conn.write_from_file(args.file).await {
        Ok(_) => info!("sending file to server"),
        Err(_) => {
            panic!("failed msg to server")
        }
    }

    info!("ENDED");
    Ok(())
}