use std::fmt::{self, Debug};
use std::io::{Read, Write};
use std::net::TcpStream;

pub trait Stream: Read + Write + Unpin {}
impl Debug for Box<dyn Stream> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Boxed Stream").finish()
    }
}

impl Stream for TcpStream {}
