use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;

use axum::extract::DefaultBodyLimit;
use bank_http_adapter::{
    AuthentikOidcAdapter, AuthentikOidcAdapterBuildError, AuthentikOidcConfiguration,
    AuthentikOidcConfigurationError,
};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Semaphore};
use tokio::task::JoinHandle;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use crate::configuration::BankUserNodeConfiguration;
use crate::session::BankUserSession;

mod routes;

pub struct BankUserNodeBinding {
    listener: TcpListener,
    local_address: SocketAddr,
}

pub struct BankUserNode {
    local_address: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    server_task: Option<JoinHandle<io::Result<()>>>,
}

#[derive(Debug)]
pub enum BankUserNodeInstallError {
    OidcConfiguration(AuthentikOidcConfigurationError),
    OidcDiscovery(AuthentikOidcAdapterBuildError),
    BankServerEndpoint(url::ParseError),
}

impl BankUserNodeBinding {
    pub async fn bind_local() -> io::Result<Self> {
        let listener =
            TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)).await?;
        let local_address = listener.local_addr()?;
        Ok(Self {
            listener,
            local_address,
        })
    }

    pub const fn local_address(&self) -> SocketAddr {
        self.local_address
    }

    pub async fn install(
        self,
        configuration: BankUserNodeConfiguration,
    ) -> Result<BankUserNode, BankUserNodeInstallError> {
        let oidc_configuration = self
            .oidc_configuration(&configuration)
            .map_err(BankUserNodeInstallError::OidcConfiguration)?;
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + configuration.maximum_deadline),
            cancellation.token(),
        );
        let oidc = AuthentikOidcAdapter::discover(oidc_configuration, &scope)
            .await
            .map_err(BankUserNodeInstallError::OidcDiscovery)?;
        self.install_with_adapter(configuration, oidc)
    }

    pub(crate) fn oidc_configuration(
        &self,
        configuration: &BankUserNodeConfiguration,
    ) -> Result<AuthentikOidcConfiguration, AuthentikOidcConfigurationError> {
        self.oidc_configuration_with_redirect(
            configuration,
            format!("http://{}/oidc/callback", self.local_address),
        )
    }

    pub(crate) fn oidc_configuration_with_redirect(
        &self,
        configuration: &BankUserNodeConfiguration,
        redirect_url: String,
    ) -> Result<AuthentikOidcConfiguration, AuthentikOidcConfigurationError> {
        AuthentikOidcConfiguration::builder()
            .issuer(configuration.issuer.clone())
            .client_id(configuration.client_id.clone())
            .client_secret(configuration.client_secret.clone())
            .redirect_url(redirect_url)
            .introspection_url(configuration.introspection_url.clone())
            .revocation_url(configuration.revocation_url.clone())
            .build()
    }

    pub(crate) fn install_with_adapter(
        self,
        configuration: BankUserNodeConfiguration,
        oidc: AuthentikOidcAdapter,
    ) -> Result<BankUserNode, BankUserNodeInstallError> {
        let session = Arc::new(
            BankUserSession::new(oidc, &configuration)
                .map_err(BankUserNodeInstallError::BankServerEndpoint)?,
        );
        Ok(start(
            self,
            session,
            configuration.maximum_body_bytes,
            configuration.maximum_request_concurrency.get(),
            configuration.maximum_live_streams.get(),
        ))
    }
}

impl BankUserNode {
    pub const fn local_address(&self) -> SocketAddr {
        self.local_address
    }

    pub async fn shutdown(mut self) -> io::Result<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        match self.server_task.take() {
            Some(task) => task.await.map_err(io::Error::other)?,
            None => Ok(()),
        }
    }
}

impl Drop for BankUserNode {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(task) = self.server_task.take() {
            task.abort();
        }
    }
}

impl std::fmt::Display for BankUserNodeInstallError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OidcConfiguration(error) => error.fmt(formatter),
            Self::OidcDiscovery(error) => error.fmt(formatter),
            Self::BankServerEndpoint(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for BankUserNodeInstallError {}

#[derive(Clone)]
struct UserNodeState {
    session: Arc<BankUserSession>,
    requests: Arc<Semaphore>,
    live_streams: Arc<Semaphore>,
}

fn start(
    binding: BankUserNodeBinding,
    session: Arc<BankUserSession>,
    maximum_body_bytes: usize,
    maximum_request_concurrency: usize,
    maximum_live_streams: usize,
) -> BankUserNode {
    let state = UserNodeState {
        session,
        requests: Arc::new(Semaphore::new(maximum_request_concurrency)),
        live_streams: Arc::new(Semaphore::new(maximum_live_streams)),
    };
    let router = routes::router()
        .layer(DefaultBodyLimit::max(maximum_body_bytes))
        .with_state(state);
    let (shutdown, shutdown_receiver) = oneshot::channel();
    let local_address = binding.local_address;
    let server_task = tokio::spawn(async move {
        axum::serve(binding.listener, router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_receiver.await;
            })
            .await
    });
    BankUserNode {
        local_address,
        shutdown: Some(shutdown),
        server_task: Some(server_task),
    }
}
