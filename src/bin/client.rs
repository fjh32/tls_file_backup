use clap::Parser;
use log::{error, info};
use rustls::RootCertStore;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::TlsConnector;

use file_backup_service::common;
use file_backup_service::connection;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ClientArgs {
    #[arg(short, long, default_value = "192.168.1.110")]
    ip: String,
    #[arg(short, long, default_value_t = 4545)]
    port: i32,
    #[arg(short, long, default_value = "./test_files")]
    file: String
}

// client needs to print output of tar
// attempt to make temp dir if it doesnt exist

// why isn't systemd client service working
// compress in memory?
#[tokio::main]
async fn main() -> io::Result<()> {
    common::setup_logger();
    let args = ClientArgs::parse();
    let host = common::make_address_str(&args.ip, &args.port);

    let path = Path::new(&args.file);
    let is_dir = path.is_dir();
    let archivename_to_tell_server = match is_dir {
        true => path.file_name().unwrap().to_str().unwrap().to_string() + ".tar.gz",
        false => path.file_name().unwrap().to_str().unwrap().to_string() + ".gz"
    };
    let absolute_path_to_archive_and_send = std::fs::canonicalize(&path)?.to_str().unwrap().to_string();


    info!("Connecting to {}", host);
    let addr = host
        .to_string()
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;

    let mut root_cert_store = rustls::RootCertStore::empty();
    let certlist = tokio::task::spawn_blocking(|| {
        rustls_native_certs::load_native_certs().expect("Could not load platform certs")
    })
    .await?;
    for cert in certlist {
        root_cert_store.add(cert).unwrap();
    }
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned()); // maybe make this async

    let ip_addr = ServerName::try_from(args.ip).unwrap();
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let tls_connector = TlsConnector::from(Arc::new(config));

    let sock_stream = TcpStream::connect(&addr).await?;
    let tls_stream = tls_connector.connect(ip_addr, sock_stream).await?;
    let mut conn = connection::Connection::new(tls_stream);
    info!("TLS connection established with {}", host);


    // sequential message passing with server

    let filename_message_to_send = format!("filename:{}:filename", archivename_to_tell_server);
    conn.write_message_from_string(filename_message_to_send)
        .await?;

    let server_response = conn.read_into_string().await?;
    if server_response != "OK" {
        let msg = "Server sent a bad response to our file request. Aborting...".to_string();
        error!("{}", msg);
        panic!("{}", msg);
    }

    info!(
        "Received ok from server. Sending {}",
        absolute_path_to_archive_and_send
    );

    if is_dir {
        conn.compress_and_send_dir(absolute_path_to_archive_and_send).await?;
    }
    else {
        conn.compress_and_send_file(absolute_path_to_archive_and_send).await?;
    }

    info!("Client done. Exiting.");
    Ok(())
}


// use this and just distribute client with .crt?
fn _add_cafile_to_root_store(roots: &mut RootCertStore, certfile: String) -> Result<(), io::Error> {
    // USE this to include CA crt file with which to accept anyone's cert the CA has signed
    // very useful to distribute client with CA cert
    println!("OPENING CERT FILE {}", certfile);
    let mut pem = BufReader::new(File::open(certfile)?);
    for cert in rustls_pemfile::certs(&mut pem) {
        let cert = match cert {
            Ok(cert) => {
                println!("Got a cert");
                cert
            }
            Err(_) => {
                println!("Err occurred ");
                break;
            }
        };
        roots.add(cert).unwrap();
    }
    Ok(())
}
