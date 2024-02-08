use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::sync::Arc;


const MAX_FRAME_SIZE: u32 = 2048;


#[derive(Debug)]
pub struct Connection {
    pub socket: TcpStream,
    pub addr: SocketAddr,
    pub encryption_string: Arc<String>
}

// maximum frame size, say 4096 bytes or something. max tcp packet size is 64kb
pub enum Frame {
    HeaderFrame {  
        raw_data: Vec<u8>,
        body: Vec<u8>,
        sender: String,
        recipient: SocketAddr,
        file_to_send: String,
        delimiter: String,
        max_frame_size: u32
    },
    DataFrame { 
        raw_data: Vec<u8>,
        max_frame_size: u32
    }
}

impl Frame {
    // this function should do a decrypt on bytestring, check validity, 
    // then try to parse one of the 2 Frame types out
    pub fn parse(bytestring: Vec<u8>) -> Result<Frame, std::io::Error> {


        return Ok(Frame::DataFrame { raw_data: bytestring, max_frame_size: 1 } )
    }
}


impl Connection {
    pub fn new(s: TcpStream, a: SocketAddr, e: Arc<String>) -> Connection {
        Connection {
            socket: s,
            addr: a,
            encryption_string: e
        }
    }
}