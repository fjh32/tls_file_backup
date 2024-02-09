use tokio::io::{split,  AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use std::error::Error;
use std::fs::File;
use std::net::SocketAddr;
use std::sync::Arc;


const MAX_FRAME_SIZE: u32 = 2048;

// do enum instead with ClientConnection and ServerConnection

#[derive(Debug)]
pub struct ClientConnection {
    pub tls_reader: ReadHalf<TlsStream<TcpStream>>,
    pub tls_writer: WriteHalf<TlsStream<TcpStream>>
}

#[derive(Debug)]
pub struct ServerConnection {
    pub tls_stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>
}

impl ServerConnection {
    pub fn new(tls_stream: tokio_rustls::server::TlsStream<TcpStream>) -> ServerConnection {
        ServerConnection { tls_stream: tls_stream }
    }

    pub async fn shutdown_tls_conn(&mut self) -> Result<(), Box<dyn Error>> {
        self.tls_stream.shutdown().await?;
        Ok(())
    }
    
    pub async fn read_message_into_string(&mut self) -> Result<String, Box<dyn Error>> {
        let data = self.read_message_into_vec().await?;
        let string = String::from_utf8(data)?;
        Ok(string)
    }

    pub async fn read_message_into_vec(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf:Vec<u8> = Vec::new();
        let n = self.tls_stream.read_buf(&mut buf).await?;
        Ok(buf)
    }

    pub async fn write_message_from_string(&mut self, data: String ) -> Result<(), Box<dyn Error>> {
        self.tls_stream.write_all(data.as_bytes()).await?;
        self.tls_stream.flush().await?;
        Ok(())
    }
}

impl ClientConnection {
    pub fn new(tls_stream: TlsStream<TcpStream>) -> ClientConnection {
        // if I need any other info, like connection cipher, get now
        let (reader, writer) = split(tls_stream);
        ClientConnection {
            tls_reader: reader,
            tls_writer: writer
        }
    }

    // pub fn close(&self) -> Result<(), Box<dyn Error>> {
    //     self.tls_reader.close();
    //     Ok(())
    // }

    pub async fn read_message_into_string(&mut self) -> Result<String, Box<dyn Error>> {
        let data = self.read_message_into_vec().await?;
        let string = String::from_utf8(data)?;
        Ok(string)
    }

    pub async fn read_message_into_vec(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf:Vec<u8> = Vec::new();
        let n = self.tls_reader.read_buf(&mut buf).await?;
        Ok(buf)
    }

    pub async fn write_message_from_string(&mut self, data: String ) -> Result<(), Box<dyn Error>> {
        self.tls_writer.write_all(data.as_bytes()).await?;
        self.tls_writer.flush().await?;
        Ok(())
    }

    pub fn read_into_file(&self, filename: File) -> Result<Vec<u8>, Box<dyn Error>>{
        let mut ret_bytes = Vec::new();

        let mut buf: [u8;4096] = [0;4096];


        Ok( ret_bytes)
    }
}