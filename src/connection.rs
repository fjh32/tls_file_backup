use async_compression::tokio::bufread::GzipEncoder;
use log::info;
use std::io;
use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::Command;

const BUFFER_SIZE: usize = 1024 * 15;

pub struct Connection<IO> {
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

    pub async fn compress_and_send(&mut self, dir_or_filename: String) -> Result<u64, io::Error> {
        let path = Path::new(&dir_or_filename);
        if path.is_dir() {
            self.compress_and_send_dir(dir_or_filename).await
        } else if path.is_file() {
            self.compress_and_send_file(dir_or_filename).await
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Not accepting anything except directories or files",
            ))
        }
    }

    // this function needs to do this: $ tar cf - /src_dir" | gzip | tls send ==> TLS Server receives a .tar.gz as a stream
    async fn compress_and_send_dir(&mut self, dirname: String) -> Result<u64, io::Error> {
        let mut tarcmd = Command::new("tar")
            .arg("-cf")
            .arg("-")
            .arg(&dirname)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let proc_out = tarcmd.stdout.as_mut().unwrap();
        let proc_out_reader = BufReader::new(proc_out);
        let mut encoder = GzipEncoder::new(proc_out_reader);

        self.write_reader_to_stream(&mut encoder).await
    }

    async fn compress_and_send_file(&mut self, filename: String) -> Result<u64, io::Error> {
        let mut encoder = GzipEncoder::new(BufReader::new(File::open(&filename).await?));
        self.write_reader_to_stream(&mut encoder).await
    }

    /// Read from file, write to self.stream.
    pub async fn write_from_file(&mut self, filename: String) -> Result<u64, io::Error> {
        let mut file = File::open(&filename).await?;
        self.write_reader_to_stream(&mut file).await
    }

    // underlying buffered write to self.stream
    async fn write_reader_to_stream<T>(&mut self, reader: &mut T) -> Result<u64, io::Error>
    where
        T: AsyncRead + Unpin,
    {
        let start = Instant::now();
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut total_bytes = 0u64;
        while let Ok(n) = reader.read(&mut buf).await {
            if n == 0 {
                break;
            }
            total_bytes += n as u64;
            let bufdata = &buf[0..n];
            let _n = self.stream.write(bufdata).await?;
            // TLS Stream needs flushing
            self.stream.flush().await?; // this line right here is why we can't simply do a tokio::copy(file,stream)
        }
        info!(
            "Sending {} bytes to server took {:?}",
            total_bytes,
            start.elapsed()
        );
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
        let mut elapsed = duration.as_secs();
        if elapsed == 0 {
            elapsed = 1u64;
        }
        let bps = total_bytes / elapsed;
        info!(
            "Receiving {} bytes for {} from client took {:?}. Network transfer speed: {} bytes per second.",
            total_bytes, filename, duration, bps
        );
        Ok(total_bytes)
    }
}
