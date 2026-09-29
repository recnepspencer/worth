//! A callback-only Bank host for an embedded application runtime.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use bank_server::BankIdentityRuntime;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, watch, Notify};
use tokio::task::JoinHandle;

use super::BankRailCompletionServerInstallation;
use super::{install, receive, start_maintenance, BankRailCompletionEndpointState};
use crate::http::server::configuration::BankHttpServerConfiguration;

pub struct BankRailCallbackServerBinding {
    listener: TcpListener,
    configuration: BankHttpServerConfiguration,
}

pub struct BankRailCallbackServer {
    address: SocketAddr,
    route: Arc<dyn super::BankRailCompletionRoute>,
    wake: Arc<Notify>,
    shutdown: Option<oneshot::Sender<()>>,
    maintenance_shutdown: watch::Sender<bool>,
    server_task: JoinHandle<io::Result<()>>,
    maintenance_task: JoinHandle<io::Result<()>>,
}

impl BankRailCallbackServerBinding {
    pub async fn bind(configuration: BankHttpServerConfiguration) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(configuration.bind_address()).await?,
            configuration,
        })
    }

    pub fn local_address(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Installs only the fixed rail callback. Ordinary Bank operations remain
    /// on the embedding host's independently authenticated entry surface.
    pub fn install_shared(
        self,
        runtime: Arc<BankIdentityRuntime>,
        rail: BankRailCompletionServerInstallation,
    ) -> io::Result<BankRailCallbackServer> {
        let address = self.listener.local_addr()?;
        let route = install(runtime, rail)?;
        let wake = Arc::new(Notify::new());
        let (maintenance_shutdown, maintenance_receiver) = watch::channel(false);
        let maintenance_task = start_maintenance(
            Arc::clone(&route),
            maintenance_receiver,
            Arc::clone(&wake),
            self.configuration.maximum_deadline(),
        );
        wake.notify_one();
        let state = BankRailCompletionEndpointState {
            route: Arc::clone(&route),
            slots: Arc::new(tokio::sync::Semaphore::new(
                self.configuration.maximum_concurrency().get(),
            )),
            wake: Arc::clone(&wake),
            maximum_deadline: self.configuration.maximum_deadline(),
        };
        let router = Router::new()
            .route("/v1/inbound/rail-completions", post(callback))
            .layer(DefaultBodyLimit::max(
                self.configuration.maximum_body_bytes(),
            ))
            .with_state(state);
        let (shutdown, receiver) = oneshot::channel();
        let server_task = tokio::spawn(async move {
            axum::serve(self.listener, router)
                .with_graceful_shutdown(async move {
                    let _ = receiver.await;
                })
                .await
        });
        Ok(BankRailCallbackServer {
            address,
            route,
            wake,
            shutdown: Some(shutdown),
            maintenance_shutdown,
            server_task,
            maintenance_task,
        })
    }
}

async fn callback(
    State(state): State<BankRailCompletionEndpointState>,
    envelope: Bytes,
) -> (StatusCode, Vec<u8>) {
    receive(state, envelope).await
}

impl BankRailCallbackServer {
    pub const fn local_address(&self) -> SocketAddr {
        self.address
    }

    pub fn observe_rail_completion(
        &self,
        correlation_token: [u8; 32],
    ) -> Option<worth_query_host::facade::primary_graph::WorthQueryInboundTerminalObservation> {
        self.route.observe_terminal(correlation_token)
    }

    pub fn continue_rail_completion(&self) {
        self.wake.notify_one();
    }

    pub async fn shutdown(mut self) -> io::Result<()> {
        self.maintenance_shutdown.send_replace(true);
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.server_task.await.map_err(io::Error::other)??;
        self.maintenance_task.await.map_err(io::Error::other)??;
        Ok(())
    }
}
