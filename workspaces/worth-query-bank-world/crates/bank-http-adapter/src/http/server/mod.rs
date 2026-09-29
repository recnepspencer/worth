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
mod inbound_completion;
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
pub use inbound_completion::BankRailCompletionServerInstallation;
use live_executor::BankHttpLiveExecutor;
use queue::BankHttpExecutionQueue;
use routes::BankHttpRouteState;

pub struct BankHttpServer {
    local_address: SocketAddr,
    rail_completion: Option<Arc<dyn inbound_completion::BankRailCompletionRoute>>,
    rail_maintenance_wake: Option<Arc<tokio::sync::Notify>>,
    shutdown: Option<oneshot::Sender<()>>,
    live_shutdown: watch::Sender<bool>,
    server_task: Option<JoinHandle<io::Result<()>>>,
    dispatcher_task: Option<JoinHandle<()>>,
    continuation_task: Option<JoinHandle<()>>,
    elevation_task: Option<JoinHandle<()>>,
    recovery_task: Option<JoinHandle<()>>,
    rail_maintenance_task: Option<JoinHandle<io::Result<()>>>,
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
        bind_application_to_listener(Arc::new(identity), self.listener, self.configuration, None)
    }

    pub fn install_with_rail_completion(
        self,
        identity: AuthentikBankIdentity,
        rail: BankRailCompletionServerInstallation,
    ) -> io::Result<BankHttpServer> {
        bind_application_to_listener(
            Arc::new(identity),
            self.listener,
            self.configuration,
            Some(rail),
        )
    }

    /// Lets a host retain the same runtime for independent owner inspection.
    pub fn install_shared_with_rail_completion(
        self,
        identity: Arc<AuthentikBankIdentity>,
        rail: BankRailCompletionServerInstallation,
    ) -> io::Result<BankHttpServer> {
        bind_application_to_listener(identity, self.listener, self.configuration, Some(rail))
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

    pub async fn bind_with_rail_completion(
        identity: AuthentikBankIdentity,
        configuration: BankHttpServerConfiguration,
        rail: BankRailCompletionServerInstallation,
    ) -> io::Result<Self> {
        BankHttpServerBinding::bind(configuration)
            .await?
            .install_with_rail_completion(identity, rail)
    }

    pub const fn local_address(&self) -> SocketAddr {
        self.local_address
    }

    /// Inspect a terminal through this server's exact installed rail route.
    pub fn observe_rail_completion(
        &self,
        correlation_token: [u8; 32],
    ) -> Option<worth_query_host::facade::primary_graph::WorthQueryInboundTerminalObservation> {
        self.rail_completion
            .as_ref()?
            .observe_terminal(correlation_token)
    }

    /// Cue one bounded owner continuation after an external recovery or
    /// capacity change affecting the installed rail completion route.
    pub fn continue_rail_completion(&self) {
        if let Some(wake) = &self.rail_maintenance_wake {
            wake.notify_one();
        }
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
        if let Some(task) = self.rail_maintenance_task.take() {
            retain_first_error(
                &mut first_error,
                task.await
                    .map_err(io::Error::other)
                    .and_then(|result| result),
            );
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
        if let Some(task) = self.rail_maintenance_task.take() {
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
    bind_application_to_listener(application, listener, configuration, None)
}

fn bind_application_to_listener<A>(
    application: Arc<A>,
    listener: TcpListener,
    configuration: BankHttpServerConfiguration,
    rail: Option<BankRailCompletionServerInstallation>,
) -> io::Result<BankHttpServer>
where
    A: BankHttpApplicationAuthenticator,
{
    let local_address = listener.local_addr()?;
    let rail = rail
        .map(|rail| inbound_completion::install(Arc::clone(&application), rail))
        .transpose()?;
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
    let rail_maintenance_wake = Arc::new(tokio::sync::Notify::new());
    let rail_maintenance_task = rail.as_ref().map(|route| {
        inbound_completion::start_maintenance(
            Arc::clone(route),
            live_shutdown.subscribe(),
            Arc::clone(&rail_maintenance_wake),
            configuration.maximum_deadline(),
        )
    });
    if rail.is_some() {
        rail_maintenance_wake.notify_one();
    }
    let state = BankHttpRouteState::new(
        queue,
        live,
        continuations,
        recovery,
        elevation,
        rail.clone(),
        std::sync::Arc::new(tokio::sync::Semaphore::new(
            configuration.maximum_concurrency().get(),
        )),
        Arc::clone(&rail_maintenance_wake),
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
        rail_completion: rail,
        rail_maintenance_wake: rail_maintenance_task
            .as_ref()
            .map(|_| rail_maintenance_wake),
        shutdown: Some(shutdown),
        live_shutdown,
        server_task: Some(server_task),
        dispatcher_task: Some(dispatcher_task),
        continuation_task: Some(continuation_task),
        recovery_task: Some(recovery_task),
        rail_maintenance_task,
        elevation_task: Some(elevation_task),
        live_thread: Some(live_thread),
    })
}
