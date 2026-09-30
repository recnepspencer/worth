//! Keep the initiating socket alive until the committed callback is captured.

use std::net::SocketAddr;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub(super) async fn post_node_holding_response(
    address: SocketAddr,
    path: &str,
    body: &serde_json::Value,
) -> TcpStream {
    let body = serde_json::to_vec(body).expect("notice body should encode");
    let mut stream = TcpStream::connect(address)
        .await
        .expect("initiating client should connect to the user node");
    let headers = format!(
        "POST {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(headers.as_bytes())
        .await
        .expect("initiating client should send headers");
    stream
        .write_all(&body)
        .await
        .expect("initiating client should send the complete notice");
    stream
}
