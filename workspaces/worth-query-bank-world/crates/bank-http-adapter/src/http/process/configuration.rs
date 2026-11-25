#![allow(
    clippy::result_large_err,
    reason = "A refused process configuration hands back the original Home owner without allocating on failure"
)]
use std::time::{Duration, Instant};
use std::{net::SocketAddr, sync::Arc};

use serde::Deserialize;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use crate::{
    AuthentikBankIdentity, AuthentikBankIdentityBuildError, AuthentikOidcConfiguration,
    AuthentikOidcConfigurationError,
};

use super::rail_transport::BankProcessRailTransport;
use super::BankHttpProcessWorld;

#[derive(Deserialize)]
pub struct BankHttpProcessConfiguration {
    pub home_directory: std::path::PathBuf,
    pub oidc: BankHttpProcessOidcConfiguration,
    pub world: BankHttpProcessWorld,
    #[serde(default)]
    pub cold_certification: Option<BankHttpProcessColdCertification>,
    #[serde(default)]
    pub external_rail: Option<SocketAddr>,
}

/// Named, process-local certificate trust; ordinary installation keeps system TLS roots.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpProcessColdCertification {
    pub trusted_root_pem: String,
}

#[derive(Deserialize)]
pub struct BankHttpProcessOidcConfiguration {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub introspection_url: String,
    pub revocation_url: String,
}

pub enum BankHttpProcessConfigurationError {
    Oidc(AuthentikOidcConfigurationError),
    World,
    Identity(AuthentikBankIdentityBuildError),
    Home(Box<bank_server::BankHomeOpenRefused>),
    Authentication {
        runtime: bank_server::BankIdentityRuntime,
        denial: bank_server::BankAuthenticationBoundaryBuildError,
    },
    ExternalRail {
        identity: AuthentikBankIdentity,
        denial: bank_server::BankExternalEffectTransportDenial,
    },
}

impl BankHttpProcessConfiguration {
    pub async fn install_identity(
        self,
    ) -> Result<AuthentikBankIdentity, BankHttpProcessConfigurationError> {
        let external_rail = self.external_rail;
        let oidc = self
            .oidc
            .build()
            .map_err(BankHttpProcessConfigurationError::Oidc)?;
        let world = self
            .world
            .build()
            .map_err(|_| BankHttpProcessConfigurationError::World)?;
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(300)),
            cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
        let identity = install_identity(
            oidc,
            world,
            self.home_directory,
            self.cold_certification,
            &scope,
        )
        .await?;
        if let Some(address) = external_rail
            && let Err(denial) = identity
                .runtime()
                .install_external_effect_transport(Arc::new(
                    BankProcessRailTransport::connected_to(address),
                ))
        {
            return Err(BankHttpProcessConfigurationError::ExternalRail { identity, denial });
        }
        Ok(identity)
    }
}

impl BankHttpProcessOidcConfiguration {
    fn build(self) -> Result<AuthentikOidcConfiguration, AuthentikOidcConfigurationError> {
        AuthentikOidcConfiguration::builder()
            .issuer(self.issuer)
            .client_id(self.client_id)
            .client_secret(self.client_secret)
            .redirect_url(self.redirect_url)
            .introspection_url(self.introspection_url)
            .revocation_url(self.revocation_url)
            .build()
    }
}

async fn install_identity(
    oidc: AuthentikOidcConfiguration,
    world: bank_server::BankWorldSeed,
    home_directory: std::path::PathBuf,
    cold_certification: Option<BankHttpProcessColdCertification>,
    scope: &WorthQueryRequestScope,
) -> Result<AuthentikBankIdentity, BankHttpProcessConfigurationError> {
    use BankHttpProcessConfigurationError as Error;
    let authentication = oidc
        .bank_authentication_configuration()
        .map_err(|cause| Error::Identity(AuthentikBankIdentityBuildError::Configuration(cause)))?;
    let adapter = match cold_certification {
        Some(cold) => {
            crate::cold_certification_for_test::discover_adapter_for_test(
                oidc,
                Some(cold.trusted_root_pem.as_bytes()),
                scope,
            )
            .await
        }
        None => crate::AuthentikOidcAdapter::discover(oidc, scope).await,
    }
    .map_err(|cause| Error::Identity(AuthentikBankIdentityBuildError::Adapter(cause)))?;
    let runtime = bank_server::open_bank_application_home(
        worth_query_host::facade::WorthQueryApplicationHomeLocation::Directory(home_directory),
        world,
    )
    .map_err(|cause| Error::Home(Box::new(cause)))?;
    AuthentikBankIdentity::attach_runtime(runtime, authentication, adapter)
        .map_err(|(runtime, denial)| Error::Authentication { runtime, denial })
}

impl std::fmt::Display for BankHttpProcessConfigurationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Oidc(error) => error.fmt(formatter),
            Self::World => formatter.write_str("invalid Bank HTTP process world"),
            Self::Identity(error) => error.fmt(formatter),
            Self::Home(error) => error.fmt(formatter),
            Self::Authentication { denial, .. } => denial.fmt(formatter),
            Self::ExternalRail { denial, .. } => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for BankHttpProcessConfigurationError {}

impl std::fmt::Debug for BankHttpProcessConfigurationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, formatter)
    }
}
