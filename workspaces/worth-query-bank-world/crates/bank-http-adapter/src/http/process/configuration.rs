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

#[derive(Debug)]
pub enum BankHttpProcessConfigurationError {
    Oidc(AuthentikOidcConfigurationError),
    World,
    Identity(AuthentikBankIdentityBuildError),
    ExternalRail(bank_server::BankExternalEffectTransportDenial),
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
            .build(&oidc)
            .map_err(|_| BankHttpProcessConfigurationError::World)?;
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(300)),
            cancellation.token(),
        );
        let identity = install_identity(oidc, world, self.cold_certification, &scope).await?;
        if let Some(address) = external_rail {
            identity
                .runtime()
                .install_external_effect_transport(Arc::new(
                    BankProcessRailTransport::connected_to(address),
                ))
                .map_err(BankHttpProcessConfigurationError::ExternalRail)?;
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
    cold_certification: Option<BankHttpProcessColdCertification>,
    scope: &WorthQueryRequestScope,
) -> Result<AuthentikBankIdentity, BankHttpProcessConfigurationError> {
    if let Some(cold) = cold_certification {
        return install_cold_identity(oidc, world, &cold.trusted_root_pem, scope).await;
    }
    AuthentikBankIdentity::install_world(oidc, world, scope)
        .await
        .map_err(BankHttpProcessConfigurationError::Identity)
}

async fn install_cold_identity(
    oidc: AuthentikOidcConfiguration,
    world: bank_server::BankWorldSeed,
    trusted_root_pem: &str,
    scope: &WorthQueryRequestScope,
) -> Result<AuthentikBankIdentity, BankHttpProcessConfigurationError> {
    crate::cold_certification_for_test::install_world_for_test(
        oidc,
        world,
        Some(trusted_root_pem.as_bytes()),
        scope,
    )
    .await
    .map_err(BankHttpProcessConfigurationError::Identity)
}

impl std::fmt::Display for BankHttpProcessConfigurationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Oidc(error) => error.fmt(formatter),
            Self::World => formatter.write_str("invalid Bank HTTP process world"),
            Self::Identity(error) => error.fmt(formatter),
            Self::ExternalRail(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for BankHttpProcessConfigurationError {}
