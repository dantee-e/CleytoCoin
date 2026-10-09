mod check_exists;
mod get_data;
mod new_node;

use std::time::Duration;

use async_std::{
    future::timeout,
    io::{ReadExt, WriteExt},
    net::TcpStream,
};
pub use check_exists::{
    notify_new_block, notify_new_node, notify_new_transaction,
};
pub use get_data::{get_data_block, get_data_transaction};

use crate::{
    error_handling::{CleytoResult, CleytonError},
    node::{ConnectedNodeInfo, Message},
};

/// Max time to wait for a connected node to answer a message
const SEND_MESSAGE_TIMEOUT: Duration = Duration::from_secs(5);

fn send_error(e: impl ToString) -> CleytonError {
    CleytonError::SendMessageError(e.to_string())
}

/// Sends `message` to the `/messages` endpoint of `connected_node` and returns
/// the body of the response. Errors if the node can't be reached, takes too
/// long to answer, or answers with a status other than 200.
pub async fn send_message_to_connected_node(
    connected_node: &ConnectedNodeInfo,
    message: &Message,
) -> CleytoResult<String> {
    let body = serde_json::to_string(message)?;

    timeout(SEND_MESSAGE_TIMEOUT, async {
        let mut stream = TcpStream::connect(connected_node.address)
            .await
            .map_err(send_error)?;

        // The node only understands HTTP, and expects the content-length
        // header in lowercase
        let request = format!(
            "POST /messages HTTP/1.1\r\n\
             host: {}\r\n\
             content-type: application/json\r\n\
             accept: application/json\r\n\
             content-length: {}\r\n\r\n\
             {}",
            connected_node.address,
            body.len(),
            body
        );

        stream
            .write_all(request.as_bytes())
            .await
            .map_err(send_error)?;
        stream.flush().await.map_err(send_error)?;

        // The node closes the connection after responding
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .map_err(send_error)?;

        parse_response(&response)
    })
    .await
    .map_err(send_error)?
}

/// Returns the body of an HTTP response if its status is 200
fn parse_response(response: &[u8]) -> CleytoResult<String> {
    let response = String::from_utf8_lossy(response);
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| send_error("Malformed HTTP response"))?;

    let status_line = head.lines().next().unwrap_or_default();
    match status_line.split(' ').nth(1) {
        Some("200") => Ok(body.to_string()),
        _ => Err(send_error(format!(
            "Connected node responded with {status_line}"
        ))),
    }
}
