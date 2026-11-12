//! Bounded HTTP byte capture preserves the real rail signature while deliberately losing its ACK.
use axum::{body::Bytes, extract::State, http::StatusCode, routing::post, Router};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::mpsc, task::JoinHandle};
pub struct CallbackCapture {
    pub address: std::net::SocketAddr,
    receiver: mpsc::Receiver<Vec<u8>>,
    task: JoinHandle<()>,
}
impl CallbackCapture {
    #[expect(
        clippy::disallowed_methods,
        reason = "async-runtime async-task=1: Court captures actual external callback bytes on its existing transport event loop; it creates no computation pool."
    )]
    pub fn bind(runtime: &tokio::runtime::Runtime) -> Self {
        runtime.block_on(async {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let (tx, receiver) = mpsc::channel(8);
            let router = Router::new()
                .route("/completion", post(capture))
                .layer(axum::extract::DefaultBodyLimit::max(4096))
                .with_state(Arc::new(tx));
            let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
            Self {
                address,
                receiver,
                task,
            }
        })
    }
    pub fn receive(&mut self, runtime: &tokio::runtime::Runtime) -> Vec<u8> {
        runtime.block_on(async {
            tokio::time::timeout(std::time::Duration::from_secs(10), self.receiver.recv())
                .await
                .expect("external rail must actually send signed callback")
                .expect("capture remains open")
        })
    }
    pub fn close(self, runtime: &tokio::runtime::Runtime) {
        self.task.abort();
        let result = runtime.block_on(self.task);
        assert!(result.is_ok() || result.is_err_and(|cause| cause.is_cancelled()));
    }
}
async fn capture(State(sender): State<Arc<mpsc::Sender<Vec<u8>>>>, body: Bytes) -> StatusCode {
    assert!(body.len() <= 4096, "rail respects its installed wire bound");
    sender
        .try_send(body.to_vec())
        .expect("the capture's finite window must hold this journey");
    StatusCode::SERVICE_UNAVAILABLE
}
