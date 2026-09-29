use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

pub(crate) struct CallbackProxy {
    pub(crate) address: SocketAddr,
    captured: Arc<Mutex<Vec<Vec<u8>>>>,
    first_capture: Option<oneshot::Receiver<Vec<u8>>>,
    release_first: Option<oneshot::Sender<()>>,
    shutdown: oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
}

impl CallbackProxy {
    pub(crate) async fn start(bank: SocketAddr) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test callback proxy should bind");
        let address = listener.local_addr().expect("proxy address should exist");
        let captured = Arc::new(Mutex::new(Vec::new()));
        let (shutdown, mut done) = oneshot::channel();
        let (first_captured, first_capture) = oneshot::channel();
        let (release_first, release) = oneshot::channel();
        let recordings = Arc::clone(&captured);
        let task = tokio::spawn(async move {
            let client = reqwest::Client::new();
            let mut first_captured = Some(first_captured);
            let mut release = Some(release);
            loop {
                tokio::select! {
                    _ = &mut done => break,
                    accepted = listener.accept() => {
                        let (mut connection, _) = accepted.expect("proxy should accept callback");
                        let body = read_http_body(&mut connection).await;
                        let first = {
                            let mut captured = recordings.lock().expect("recordings lock");
                            captured.push(body.clone());
                            captured.len() == 1
                        };
                        if first {
                            first_captured.take().expect("first callback signal is unique")
                                .send(body.clone()).expect("court must await first callback");
                            tokio::select! {
                                released = release.take().expect("first callback release is unique") => {
                                    released.expect("court must release first callback");
                                }
                                _ = &mut done => break,
                            }
                        }
                        let response = client
                            .post(format!("http://{bank}/v1/inbound/rail-completions"))
                            .body(body)
                            .send()
                            .await
                            .expect("proxy should forward exact callback bytes");
                        eprintln!(
                            "rail completion proxy: callback {} returned {}",
                            recordings.lock().expect("recordings lock").len(),
                            response.status()
                        );
                        if first {
                            continue; // Bank accepted custody; the rail loses this response.
                        }
                        let status = response.status();
                        let bytes = response.bytes().await.expect("Bank ACK should be readable");
                        let header = format!(
                            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            status.as_u16(), status.canonical_reason().unwrap_or("OK"), bytes.len()
                        );
                        connection.write_all(header.as_bytes()).await.expect("proxy response headers");
                        connection.write_all(&bytes).await.expect("proxy response body");
                    }
                }
            }
        });
        Self {
            address,
            captured,
            first_capture: Some(first_capture),
            release_first: Some(release_first),
            shutdown,
            task,
        }
    }

    pub(crate) fn captured(&self) -> Vec<Vec<u8>> {
        self.captured.lock().expect("recordings lock").clone()
    }

    pub(crate) async fn await_first_capture(&mut self) -> Vec<u8> {
        self.first_capture
            .take()
            .expect("first capture awaited once")
            .await
            .expect("proxy should capture first callback")
    }

    pub(crate) fn release_first(&mut self) {
        self.release_first
            .take()
            .expect("first callback released once")
            .send(())
            .expect("proxy should still await release");
    }

    pub(crate) async fn shutdown(self) {
        let _ = self.shutdown.send(());
        self.task.await.expect("proxy should stop");
    }
}

async fn read_http_body(connection: &mut TcpStream) -> Vec<u8> {
    let mut frame = Vec::new();
    let boundary = loop {
        let mut chunk = [0u8; 4096];
        let read = connection.read(&mut chunk).await.expect("callback read");
        assert!(
            read > 0 && frame.len() + read <= 8192,
            "bounded callback frame"
        );
        frame.extend_from_slice(&chunk[..read]);
        if let Some(start) = frame.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
            break start + 4;
        }
    };
    let headers = std::str::from_utf8(&frame[..boundary]).expect("HTTP headers are ASCII");
    let length: usize = headers
        .lines()
        .find_map(|line| {
            line.split_once(':').and_then(|(name, value)| {
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse().expect("valid content length"))
            })
        })
        .expect("rail sender should provide content length");
    assert!(length <= 4096, "callback envelope remains bounded");
    while frame.len() - boundary < length {
        let mut chunk = [0u8; 4096];
        let read = connection
            .read(&mut chunk)
            .await
            .expect("callback body read");
        assert!(
            read > 0 && frame.len() + read <= 8192,
            "bounded callback body"
        );
        frame.extend_from_slice(&chunk[..read]);
    }
    frame[boundary..boundary + length].to_vec()
}

pub(crate) fn correlation_token(envelope: &[u8]) -> [u8; 32] {
    let mut offset = 16; // BANK-COMPLETION1
    for _ in 0..2 {
        skip_text(envelope, &mut offset);
    } // audience, source
    offset += 8 + 32 + 8 + 8; // epoch, message identity, issue and expiry
    skip_text(envelope, &mut offset); // protocol identity
    offset += 2; // protocol version
    skip_text(envelope, &mut offset); // correlation family
    let length = u16::from_be_bytes(envelope[offset..offset + 2].try_into().unwrap()) as usize;
    offset += 2;
    assert_eq!(length, 32, "Query correlation token has fixed width");
    envelope[offset..offset + length]
        .try_into()
        .expect("captured token should fit")
}

fn skip_text(envelope: &[u8], offset: &mut usize) {
    let length = u16::from_be_bytes(envelope[*offset..*offset + 2].try_into().unwrap()) as usize;
    *offset += 2 + length;
}
