//! A bound listener cannot accept requests until independent custody has reached the host.
use super::*;
/// Prepared transport with no serving task. Extract the independent continuation before activation.
/// A prepared listener cannot use running-server shutdown.
/// ```compile_fail,E0599
/// use bank_http_adapter::BankHttpPreparedListener;
/// async fn not_running(prepared: BankHttpPreparedListener) { prepared.shutdown().await; }
/// ```
/// Activation is the valid counterpart; it creates no principal or native operation authority.
/// ```
/// use bank_http_adapter::{BankHttpPreparedListener, BankHttpServer};
/// fn activate(prepared: BankHttpPreparedListener) -> BankHttpServer { prepared.activate() }
/// ```
#[must_use = "retain independent mutation custody before activating the listener"]
pub struct BankHttpPreparedListener {
    pub(super) server: BankHttpServer,
    pub(super) listener: TcpListener,
    pub(super) router: axum::Router,
}
impl BankHttpPreparedListener {
    pub fn local_address(&self) -> SocketAddr {
        self.server.local_address()
    }
    pub fn activate(self) -> BankHttpServer {
        let Self {
            mut server,
            listener,
            router,
        } = self;
        let (shutdown, receiver) = oneshot::channel();
        server.shutdown = Some(shutdown);
        server.server_task = Some(tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = receiver.await;
                })
                .await
        }));
        server
    }
}
