use log::info;
use std::io;
use std::time::Instant;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct Connection<IO> {
    // these generic type bounds will make it easy to unit test
    pub stream: IO,
}

impl<IO: AsyncRead + AsyncWrite + Unpin> Connection<IO> {
    pub fn new(iostream: IO) -> Connection<IO> {
        Connection { stream: iostream }
    }

    pub async fn shutdown_tls_conn(&mut self) -> Result<(), io::Error> {
        self.stream.shutdown().await
    }

    pub async fn read_into_vec(&mut self) -> Result<Vec<u8>, io::Error> {
        let mut buf: Vec<u8> = Vec::new();
        let n = self.stream.read_buf(&mut buf).await?;
        buf.truncate(n);
        Ok(buf)
    }

    pub async fn read_into_string(&mut self) -> Result<String, io::Error> {
        let bytes = self.read_into_vec().await?;
        let s = String::from_utf8(bytes)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Ok(s)
    }

    pub async fn write_message_from_string(&mut self, data: String) -> Result<(), io::Error> {
        self.stream.write_all(data.as_bytes()).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Read from file, write to self.stream.
    pub async fn write_from_file(&mut self, filename: String) -> Result<u64, io::Error> {
        let mut file = File::open(&filename).await?;
        let start = Instant::now();
        // can use a bufreader to customize buffer size, default is 8k
        let total_bytes = tokio::io::copy(&mut file, &mut self.stream).await?;
        info!("Sending {} to server took {:?}", filename, start.elapsed());
        Ok(total_bytes)
    }

    /// Read from self.stream, write to file
    pub async fn read_to_file(&mut self, filename: String) -> Result<u64, io::Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&filename)
            .await?;

        let start = Instant::now();
        let total_bytes = tokio::io::copy(&mut self.stream, &mut file).await?;
        info!(
            "Receiving {} from client took {:?}",
            filename,
            start.elapsed()
        );
        Ok(total_bytes)
    }
}
