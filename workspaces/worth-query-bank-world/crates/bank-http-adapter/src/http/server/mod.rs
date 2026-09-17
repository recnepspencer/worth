mod aftermath_routes;
mod application;
mod authenticated_owner;
mod authentication;
mod configuration;
mod continuation_executor;
mod continuation_registry;
mod continuation_routes;
mod elevation_executor;
mod elevation_registry;
mod elevation_routes;
mod estate_denial;
mod live_executor;
mod live_routes;
mod mutation_application;
mod mutation_routes;
mod query_denial;
mod query_publication;
mod queue;
mod recovery_executor;
mod recovery_registry;
mod recovery_routes;
mod request_admission;
mod routes;

#[cfg(test)]
mod tests;

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::{oneshot, watch};
use tokio::task::JoinHandle;

use crate::AuthentikBankIdentity;

use authentication::BankHttpApplicationAuthenticator;
pub use configuration::BankHttpServerConfiguration;
use live_executor::BankHttpLiveExecutor;
use queue::BankHttpExecutionQueue;
use routes::BankHttpRouteState;

pub struct BankHttpServer {
    local_address: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    live_shutdown: watch::Sender<bool>,
    server_task: Option<JoinHandle<io::Result<()>>>,
    dispatcher_task: Option<JoinHandle<()>>,
    continuation_task: Option<JoinHandle<()>>,
    elevation_task: Option<JoinHandle<()>>,
    recovery_task: Option<JoinHandle<()>>,
    live_thread: Option<std::thread::JoinHandle<io::Result<()>>>,
}

pub struct BankHttpServerBinding {
    listener: TcpListener,
    local_address: SocketAddr,
    configuration: BankHttpServerConfiguration,
}

impl BankHttpServerBinding {
    pub async fn bind(configuration: BankHttpServerConfiguration) -> io::Result<Self> {
        let listener = TcpListener::bind(configuration.bind_address()).await?;
        let local_address = listener.local_addr()?;
        Ok(Self {
            listener,
            local_address,
            configuration,
        })
    }

    pub const fn local_address(&self) -> SocketAddr {
        self.local_address
    }

    pub fn install(self, identity: AuthentikBankIdentity) -> io::Result<BankHttpServer> {
        bind_application_to_listener(Arc::new(identity), self.listener, self.configuration)
    }
}

impl BankHttpServer {
    pub async fn bind(
        identity: AuthentikBankIdentity,
        configuration: BankHttpServerConfiguration,
    ) -> io::Result<Self> {
        BankHttpServerBinding::bind(configuration)
            .await?
            .install(identity)
    }

    pub const fn local_address(&self) -> SocketAddr {
        self.local_address
    }

    pub async fn shutdown(mut self) -> io::Result<()> {
        self.live_shutdown.send_replace(true);
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        let mut first_error = None;
        if let Some(task) = self.server_task.take() {
            retain_first_error(
                &mut first_error,
                task.await
                    .map_err(io::Error::other)
                    .and_then(|result| result),
            );
        }
        for task in [
            self.dispatcher_task.take(),
            self.continuation_task.take(),
            self.recovery_task.take(),
            self.elevation_task.take(),
        ]
        .into_iter()
        .flatten()
        {
            retain_first_error(&mut first_error, task.await.map_err(io::Error::other));
        }
        if let Some(thread) = self.live_thread.take() {
            retain_first_error(
                &mut first_error,
                tokio::task::spawn_blocking(move || {
                    thread
                        .join()
                        .map_err(|_| io::Error::other("Bank HTTP live executor panicked"))
                        .and_then(|result| result)
                })
                .await
                .map_err(io::Error::other)
                .and_then(|result| result),
            );
        }
        first_error.map_or(Ok(()), Err)
    }
}

fn retain_first_error(first: &mut Option<io::Error>, result: io::Result<()>) {
    if let Err(error) = result {
        if first.is_none() {
            *first = Some(error);
        }
    }
}

impl Drop for BankHttpServer {
    fn drop(&mut self) {
        self.live_shutdown.send_replace(true);
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(task) = self.server_task.take() {
            task.abort();
        }
        if let Some(task) = self.dispatcher_task.take() {
            task.abort();
        }
        if let Some(task) = self.continuation_task.take() {
            task.abort();
        }
        if let Some(task) = self.recovery_task.take() {
            task.abort();
        }
        if let Some(task) = self.elevation_task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
async fn bind_application<A>(
    application: Arc<A>,
    configuration: BankHttpServerConfiguration,
) -> io::Result<BankHttpServer>
where
    A: BankHttpApplicationAuthenticator,
{
    let listener = TcpListener::bind(configuration.bind_address()).await?;
    bind_application_to_listener(application, listener, configuration)
}

fn bind_application_to_listener<A>(
    application: Arc<A>,
    listener: TcpListener,
    configuration: BankHttpServerConfiguration,
) -> io::Result<BankHttpServer>
where
    A: BankHttpApplicationAuthenticator,
{
    let local_address = listener.local_addr()?;
    let (queue, dispatcher_task) = BankHttpExecutionQueue::start(
        Arc::clone(&application),
        configuration.queue_capacity().get(),
        configuration.maximum_concurrency().get(),
    );
    let (continuations, continuation_task) =
        continuation_executor::BankHttpContinuationExecutor::start(
            Arc::clone(&application),
            configuration.queue_capacity().get(),
            configuration.opaque_handle_capacity().get(),
            configuration.opaque_handle_lifetime(),
        );
    let (recovery, recovery_task) = recovery_executor::BankHttpRecoveryExecutor::start(
        Arc::clone(&application),
        configuration.queue_capacity().get(),
        configuration.opaque_handle_capacity().get(),
        configuration.opaque_handle_lifetime(),
    );
    let (elevation, elevation_task) = elevation_executor::BankHttpElevationExecutor::start(
        Arc::clone(&application),
        configuration.queue_capacity().get(),
        configuration.opaque_handle_capacity().get(),
        configuration.opaque_handle_lifetime(),
    );
    let (live, live_thread) = BankHttpLiveExecutor::start(
        application,
        configuration.queue_capacity().get(),
        configuration.stream_queue_capacity().get(),
        configuration.maximum_live_streams().get(),
    )?;
    let live_shutdown = live.shutdown_signal();
    let state = BankHttpRouteState::new(
        queue,
        live,
        continuations,
        recovery,
        elevation,
        configuration.maximum_deadline(),
    );
    let router = routes::router(state, configuration.maximum_body_bytes());
    let (shutdown, shutdown_receiver) = oneshot::channel();
    let server_task = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_receiver.await;
            })
            .await
    });
    Ok(BankHttpServer {
        local_address,
        shutdown: Some(shutdown),
        live_shutdown,
        server_task: Some(server_task),
        dispatcher_task: Some(dispatcher_task),
        continuation_task: Some(continuation_task),
        recovery_task: Some(recovery_task),
        elevation_task: Some(elevation_task),
        live_thread: Some(live_thread),
    })
}
