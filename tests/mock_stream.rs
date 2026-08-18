use cleyto_coin::node::{Node, NodeState, Stream};

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

pub struct MockStream {
    pub read_data: Vec<u8>,
    pub write_data: Vec<u8>,
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = std::cmp::min(self.read_data.len(), buf.len());
        buf[..len].copy_from_slice(&self.read_data[..len]);
        self.read_data.drain(..len);
        Ok(len)
    }
}

impl Write for MockStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_data.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Stream for MockStream {}

impl Unpin for MockStream {}

/// This is for raw data. If you want a more organized function, use
/// request(&str, &str, Option<&str>, NodeState).
pub async fn send_data(
    input_bytes: &[u8],
    state: Arc<Mutex<NodeState>>,
) -> Result<Option<String>, Option<String>> {
    let input_bytes_size = input_bytes.len();

    let mut contents = vec![0u8; input_bytes_size];
    contents[..input_bytes.len()].clone_from_slice(input_bytes);
    let stream = MockStream {
        read_data: contents,
        write_data: Vec::new(),
    };

    Node::handle_connection(state, Box::new(stream))
}

fn raw_request(method: &str, path: &str, body: Option<&str>) -> Vec<u8> {
    match body {
            Some(b) => format!(
                "{method} {path} HTTP/1.1\r\nhost: localhost\r\ncontent-length: {}\r\n\r\n{}",
                b.len(),
                b
            )
            .into_bytes(),
            None => format!("{method} {path} HTTP/1.1\r\nhost: localhost\r\n\r\n").into_bytes(),
        }
}

pub async fn request(
    method: &str,
    path: &str,
    body: Option<&str>,
    state: Arc<Mutex<NodeState>>,
) -> Result<Option<String>, Option<String>> {
    send_data(&raw_request(method, path, body), state).await
}

#[tokio::test]
async fn resolve_endpoints_test() {}
