use std::fs::File;
use std::io;
use std::io::BufReader;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::path::Path;
use rustls_pemfile::ec_private_keys;
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::pki_types::CertificateDer;
use tokio_rustls::rustls::pki_types::PrivateKeyDer;
use tokio_rustls::TlsAcceptor;
use std::sync::Arc;
use log::{ error, info};
use clap::Parser;

use file_backup_service::common;
use file_backup_service::connection;


// curl --cacert certs/new/server-certificate.pem https://ripplein.space:4545/ --resolve ripplein.space:4545:127.0.0.1
fn load_certs(filename: &String) -> io::Result<Vec<CertificateDer<'static>>> {
    let file = File::open(&Path::new(filename))?;
    certs(&mut BufReader::new(file)).collect()
}

/// pkcs keys
fn load_keys(filename: &String) -> io::Result<PrivateKeyDer<'static>> {
    let file = File::open(&Path::new(filename))?;
    pkcs8_private_keys(&mut BufReader::new(file))
        .next()
        .unwrap()
        .map(Into::into)
}

/// elliptic curve keys
fn _load_ec_keys(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
    let file = File::open(&Path::new(filename))?;
    ec_private_keys(&mut BufReader::new(file))
        .next()
        .unwrap()
        .map(Into::into)
}


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ServerArgs {
    #[arg(short, long, default_value = "0.0.0.0")]
    ip: String,
    #[arg(short, long, default_value_t = 4545)]
    port: i32,
    #[arg(short, long)]
    cert: String,
    #[arg(short, long)]
    key: String,
    #[arg(short, long, default_value = "/drives/breen/backups/")]
    write_dir: String
}


#[tokio::main]
async fn main() -> io::Result<()> {
    common::setup_logger();

    let args = ServerArgs::parse();

    let my_addr_str = common::make_address_str(&args.ip, &args.port);
    info!("TLS Server running on {}. Writing incoming files to {}", my_addr_str, args.write_dir);
    

    let addr = my_addr_str
        .to_string()
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;

    let certs = load_certs(&args.cert)?;
    let key = load_keys(&args.key)?;
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    info!("Loaded keys from: {}", args.cert);


    let args = Arc::new(args);
    let tlsacceptor =  TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(&addr).await?;
    
    loop {
        let args = args.clone();
        let (socket, peer_addr) = listener.accept().await?;
        let tlsacceptor_ptr = tlsacceptor.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(socket, peer_addr, tlsacceptor_ptr, args).await {
                error!("{:?}", err);
            };
        });
    }
}


async fn handle_client(sock: TcpStream, peer_addr: SocketAddr, tls_acceptor: TlsAcceptor, server_args: Arc<ServerArgs>) -> Result<(), io::Error> {

        info!("=================== New Client ===================");
        let tls_stream = tls_acceptor.accept(sock).await?;
        let mut conn = connection::Connection::new(tls_stream);
        info!("Connected via TLS to {}", peer_addr);

        // Sequential message passing with client
        let mut filename_to_write = common::verify_filename(conn.read_into_string().await?)?;
        info!("{} wants to send us this file: {}", peer_addr, filename_to_write);
        filename_to_write = common::format_filename(&peer_addr.ip().to_string(), &filename_to_write);

        conn.write_message_from_string(String::from("OK")).await?;

        let mut absfilepath = server_args.write_dir.clone();
        absfilepath.push_str(&filename_to_write);
        info!("Reading data from {} into {}", peer_addr, absfilepath);
        conn.read_to_file(absfilepath).await?;

        info!("{} file transfer complete. Connection closed.", peer_addr);
        Ok(())
}