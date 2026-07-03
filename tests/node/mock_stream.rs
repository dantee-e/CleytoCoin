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

pub async fn send_data(
    input_bytes: &[u8],
    state: NodeState,
) -> Result<Option<String>, Option<String>> {
    let state = Arc::new(Mutex::new(state));

    let mut contents = vec![0u8; 1024];
    contents[..input_bytes.len()].clone_from_slice(input_bytes);
    let stream = MockStream {
        read_data: contents,
        write_data: Vec::new(),
    };

    Node::handle_connection(state, Box::new(stream))
}

#[tokio::test]
async fn resolve_endpoints_test() {}
