use log::info;
use std::io;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const BUFFER_SIZE: usize = 1024 * 15;

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
        // can use a bufreader to customize buffer size, default is 8k
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let start = Instant::now();
        let mut total_bytes = 0u64;

        while let Ok(n) = file.read(&mut buf).await {
            if n == 0 {
                break;
            }
            total_bytes += n as u64;
            let bufdata = &buf[0..n];
            let _n = self.stream.write(bufdata).await?;
            self.stream.flush().await?; // this line right here is why we can't simply do a tokio::copy(file,stream)
        }
        info!("Sending {} to server took {:?}", filename, start.elapsed());
        Ok(total_bytes)
    }

    /// Read from self.stream, write to file
    pub async fn read_to_file(&mut self, filename: String) -> Result<u64, io::Error> {
        let mut file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&filename)
            .await?;

        let start = Instant::now();
        let mut total_bytes = 0u64;
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        while let Ok(n) = self.stream.read(&mut buf).await {
            if n == 0 {
                break;
            }
            total_bytes += n as u64;
            let bufdata = &buf[0..n];
            let _n = file.write(bufdata).await?;
        }
        let duration = start.elapsed();
        let mut elapsed = duration.as_secs() as u64;
        if elapsed == 0 {
            elapsed = 1u64;
        }
        let bps = total_bytes / elapsed;
        info!(
            "Receiving {} from client took {:?}. Network transfer speed: {} bytes per second.",
            filename, duration, bps
        );
        Ok(total_bytes)
    }
}
