use tokio::io::{split, AsyncRead, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;
use std::error::Error;
use std::fs::File;
use std::net::SocketAddr;
use std::sync::Arc;


pub async fn read_into_vec<R>(read_stream:&mut R) -> Result<Vec<u8>, Box<dyn Error>>
where
    R: AsyncRead + Unpin{

    let mut buf:Vec<u8> = Vec::new();
    let n = read_stream.read_buf(&mut buf).await?;
    Ok(buf)
}

pub async fn read_into_string<R>(read_stream:&mut R) -> Result<String, Box<dyn Error>>
where
    R: AsyncRead + Unpin{

        let data = read_into_vec(read_stream).await?;
        let string = String::from_utf8(data)?;
        Ok(string)

}

pub async fn write_message_from_string<W>(write_stream: &mut W, data: String ) -> Result<(), Box<dyn Error>> 
where
    W: AsyncWriteExt + Unpin {
    write_stream.write_all(data.as_bytes()).await?;
    write_stream.flush().await?;
    Ok(())
}

pub async fn shutdown_tls_conn<W>(write_stream: &mut W) -> Result<(), Box<dyn Error>> 
where
    W: AsyncWriteExt + Unpin {
    write_stream.shutdown().await?;
    Ok(())
}

pub fn read_into_file<R>(read_stream: &mut R, filename: File) -> Result<Vec<u8>, Box<dyn Error>>
where
    R: AsyncRead + Unpin{
    let mut ret_bytes = Vec::new();

    let mut buf: [u8;4096] = [0;4096];
    // use read_stream.read(&[u8])

    Ok( ret_bytes)
}