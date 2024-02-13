use tokio::io::{split, AsyncRead, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{path::Path, io::{Read, BufReader}};
use std::fs::OpenOptions;
use log::{debug, error, log_enabled, info, Level};

const BUFFER_SIZE: usize = 1024 * 15; // 15KB

pub struct Connection<IO: AsyncRead + AsyncWriteExt + Unpin> { // these generic type bounds will make it easy to unit test
    pub stream: IO
}

impl<IO:AsyncRead + AsyncWriteExt + Unpin> Connection<IO> {
    pub fn new(iostream: IO) -> Connection<IO> { // put better constraints on the input type here
        Connection{ stream: iostream }
    }



    pub async fn shutdown_tls_conn(&mut self) -> Result<(), Box<dyn Error>> {
        self.stream.shutdown().await?;
        Ok(())
    }


    pub async fn read_into_vec(&mut self) -> Result<Vec<u8>, Box<dyn Error>>{

        let mut buf:Vec<u8> = Vec::new();
        let n = self.stream.read_buf(&mut buf).await?;
        Ok(buf)
    }

    pub async fn read_into_string(&mut self) -> Result<String, Box<dyn Error>>{

            let data = self.read_into_vec().await?;
            let string = String::from_utf8(data)?;
            Ok(string)

    }

    pub async fn write_message_from_string(&mut self, data: String ) -> Result<(), Box<dyn Error>> {
        self.stream.write_all(data.as_bytes()).await?;
        self.stream.flush().await?;
        Ok(())
    }


    // read from file, write to stream
    pub async fn write_from_file(&mut self, filename: String) -> Result<usize, Box<dyn Error>> {
        let mut file = File::open(filename)?;
        let mut buf: [u8;BUFFER_SIZE] = [0;BUFFER_SIZE];
        // let mut buf = Vec::with_capacity(BUFFER_SIZE);
        while let Ok(n) = file.read(&mut buf) {
            // debug!("Client read {} bytes from file and sending to server",n);
            if n <= 0 {
                break;
            }
            let bufdata = &buf[0..n];
            self.stream.write(bufdata).await?;
            self.stream.flush().await?;
            
        }

        Ok(0usize)
    }

    // read from stream, write to file
    pub async fn read_to_file(&mut self, filename: String) -> Result<(), Box<dyn Error>>{
        let mut file = OpenOptions::new().read(true).write(true).create(true).open(&filename)?;
        let mut buf: [u8;BUFFER_SIZE] = [0;BUFFER_SIZE]; // Rust won't allow dynamic sized regular arrays
        // let mut buf = Vec::with_capacity(BUFFER_SIZE);
        while let Ok(n) = self.stream.read(&mut buf).await {
            // debug!("Read from tls stream and writing {} bytes to {}", n, &filename);
            if n <= 0 {
                break;
            }
            let bufdata = &buf[0..n];
            file.write(bufdata)?;
            
        }


        // Ok( ret_bytes)
        Ok(())
    }
}
