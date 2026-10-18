mod aftermath_routes;
mod application;
mod authenticated_owner;
mod authentication;
mod authorization_denial;
mod closure;
mod configuration;
mod continuation_executor;
mod continuation_registry;
mod continuation_routes;
mod elevation_executor;
mod elevation_registry;
mod elevation_routes;
mod estate_denial;
mod inbound_completion;
mod installation;
mod prepared_listener;
pub use prepared_listener::BankHttpPreparedListener;
mod construction_denial;
mod lifecycle;
mod live_executor;
mod live_routes;
mod mutation_application;
mod mutation_command;
mod mutation_construction;
mod mutation_custody;
mod mutation_executor;
mod mutation_refusal;
mod mutation_routes;
mod mutation_worker;
#[cfg(test)]
use installation::bind_application;
pub use installation::BankHttpServerInstallation;
pub use mutation_executor::BankHttpMutationContinuation;
mod query_denial;
mod query_publication;
mod queue;
mod recovery_executor;
mod recovery_registry;
mod recovery_routes;
mod relational_execution_denial;
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
pub use closure::{BankHttpServerClose, BankHttpServerCloseFailure};
pub use configuration::BankHttpServerConfiguration;
pub use inbound_completion::{
    BankRailCallbackServer, BankRailCallbackServerBinding, BankRailCloseAssessment,
    BankRailCompletionClose, BankRailCompletionCloseFailure, BankRailCompletionContinuation,
    BankRailCompletionServerInstallation,
};
use live_executor::BankHttpLiveExecutor;
use queue::BankHttpExecutionQueue;
use routes::BankHttpRouteState;

pub struct BankHttpServer {
    local_address: SocketAddr,
    mutations: Option<BankHttpMutationContinuation>,
    rail_completion: Option<Arc<dyn inbound_completion::BankRailCompletionRoute>>,
    rail_maintenance_wake: Option<Arc<tokio::sync::Notify>>,
    shutdown: Option<oneshot::Sender<()>>,
    live_shutdown: watch::Sender<bool>,
    server_task: Option<JoinHandle<io::Result<()>>>,
    dispatcher_task: Option<JoinHandle<()>>,
    continuation_task: Option<JoinHandle<()>>,
    recovery_task: Option<JoinHandle<()>>,
    rail_maintenance_task: Option<JoinHandle<io::Result<()>>>,
    rail_maintenance_deadline: std::time::Duration,
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

    pub fn install(
        self,
        identity: AuthentikBankIdentity,
    ) -> io::Result<BankHttpServerInstallation> {
        installation::bind_application_to_listener(
            Arc::new(identity),
            self.listener,
            self.configuration,
            None,
        )
    }

    pub fn install_with_rail_completion(
        self,
        identity: AuthentikBankIdentity,
        rail: BankRailCompletionServerInstallation,
    ) -> io::Result<BankHttpServerInstallation> {
        installation::bind_application_to_listener(
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
    ) -> io::Result<BankHttpServerInstallation> {
        installation::bind_application_to_listener(
            identity,
            self.listener,
            self.configuration,
            Some(rail),
        )
    }
}

impl BankHttpServer {
    pub async fn bind(
        identity: AuthentikBankIdentity,
        configuration: BankHttpServerConfiguration,
    ) -> io::Result<BankHttpServerInstallation> {
        BankHttpServerBinding::bind(configuration)
            .await?
            .install(identity)
    }

    pub async fn bind_with_rail_completion(
        identity: AuthentikBankIdentity,
        configuration: BankHttpServerConfiguration,
        rail: BankRailCompletionServerInstallation,
    ) -> io::Result<BankHttpServerInstallation> {
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

    /// Observe bounded work for the installed estate completion route.
    pub fn observe_estate_rail_completion_cost(
        &self,
    ) -> Option<worth_query_host::facade::primary_graph::WorthQueryInboundCostObservation> {
        self.rail_completion.as_ref()?.observe_estate_cost()
    }

    /// Cue one bounded owner continuation after an external recovery or
    /// capacity change affecting the installed rail completion route.
    pub fn continue_rail_completion(&self) {
        if let Some(wake) = &self.rail_maintenance_wake {
            wake.notify_one();
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
        if let Some(task) = self.rail_maintenance_task.take() {
            task.abort();
        }
    }
}

mod advancement_denial;

mod native_cleanup_custody;
mod native_route;

mod native_cleanup_command;

mod cleanup_description;

mod native_cleanup_handoff;
pub use native_cleanup_custody::{BankHttpCleanupRuntime, BankHttpReleasedCleanup};

mod estate_source_command;
mod estate_source_custody;

mod mutation_advancement;

mod recovery_handoff;

mod native_cleanup_delivery;
