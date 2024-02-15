use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use std::fs::File;
use std::io::Write;
use std::io;
use std::io::Read;
use std::fs::OpenOptions;
use log::info;
use std::time::Instant;

const BUFFER_SIZE: usize = 1024 * 15; // 15KB

pub struct Connection<IO: AsyncRead + AsyncWriteExt + Unpin> { // these generic type bounds will make it easy to unit test
    pub stream: IO
}

impl<IO:AsyncRead + AsyncWriteExt + Unpin> Connection<IO> {
    pub fn new(iostream: IO) -> Connection<IO> {
        Connection{ stream: iostream }
    }



    pub async fn shutdown_tls_conn(&mut self) -> Result<(), io::Error> {
        match self.stream.shutdown().await {
            Ok(()) => Ok(()),
            Err(err) => Err(err)
        }
    }


    pub async fn read_into_vec(&mut self) -> Result<Vec<u8>, io::Error>{
        let mut buf:Vec<u8> = Vec::new();
        match self.stream.read_buf(&mut buf).await {
            Ok(_n) => Ok(buf),
            Err(err) => Err(err)
        }
    }

    pub async fn read_into_string(&mut self) -> Result<String, io::Error>{
            let data = self.read_into_vec().await?;
            match String::from_utf8(data) {
                Ok(string) => Ok(string),
                Err(err) => Err(io::Error::new(io::ErrorKind::InvalidData, err))
            }
    }

    pub async fn write_message_from_string(&mut self, data: String ) -> Result<(), io::Error> {
        self.stream.write_all(data.as_bytes()).await?;
        self.stream.flush().await?;
        Ok(())
    }


    /// Read from file, write to self.stream.
    pub async fn write_from_file(&mut self, filename: String) -> Result<usize, io::Error> {
        let mut file = File::open(&filename)?;

        let mut buf: [u8;BUFFER_SIZE] = [0;BUFFER_SIZE];
        let start = Instant::now();
        let mut total_bytes = 0usize;

        while let Ok(n) = file.read(&mut buf) {
            total_bytes += n;
            if n <= 0 {
                break;
            }
            let bufdata = &buf[0..n];
            self.stream.write(bufdata).await?;
            self.stream.flush().await?;
        }
        info!("Sending {} to server took {:?}", filename, start.elapsed());
        Ok(total_bytes)
    }


    /// Read from self.stream, write to file
    pub async fn read_to_file(&mut self, filename: String) -> Result<usize, io::Error>{
        let mut file = OpenOptions::new().read(true).write(true).create(true).open(&filename)?;

        let mut buf: [u8;BUFFER_SIZE] = [0;BUFFER_SIZE]; // Rust won't allow dynamic sized regular arrays
        let start = Instant::now();
        let mut total_bytes = 0usize;

        while let Ok(n) = self.stream.read(&mut buf).await {
            total_bytes += n;
            if n <= 0 {
                break;
            }
            let bufdata = &buf[0..n];
            file.write(bufdata)?;
        }
        info!("Receiving {} from client took {:?}", filename, start.elapsed());
        Ok(total_bytes)
    }
}
