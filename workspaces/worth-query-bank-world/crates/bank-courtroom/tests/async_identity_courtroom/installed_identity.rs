use std::time::{Duration, Instant};

use bank_domain::model::BankPrincipalId;
use bank_http_adapter::cold_certification_for_test;
use bank_http_adapter::{
    AuthentikBankIdentity, AuthentikBankIdentityBuildError, AuthentikOidcAdapterBuildError,
    AuthentikOidcConfiguration,
};
use bank_server::BankPrincipalSeed;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

use super::callback::CallbackReceiver;
use super::docker_world::{DockerIdentityWorld, IdentityEndpoints};
use super::fixture::IdentityFixture;

pub struct InstalledIdentityWorld {
    pub callback: CallbackReceiver,
    pub fixture: IdentityFixture,
    pub endpoints: IdentityEndpoints,
    pub scope: WorthQueryRequestScope,
    pub identity: AuthentikBankIdentity,
    docker: DockerIdentityWorld,
}

impl InstalledIdentityWorld {
    pub async fn install() -> Result<Self, String> {
        let callback = CallbackReceiver::bind()
            .await
            .map_err(|error| format!("courtroom callback listener failed: {error}"))?;
        let fixture = IdentityFixture::dynamic(callback.redirect_url());
        let docker = DockerIdentityWorld::start(&fixture).await?;
        let endpoints = docker.wait_until_ready(&fixture).await?;
        let scope = request_scope(Duration::from_secs(300));
        let configuration = configuration(&endpoints, &fixture, &callback)?;
        let seeds = principal_seeds(&endpoints, &fixture)?;
        require_ordinary_tls_rejection(configuration.clone(), &scope).await?;
        let identity =
            cold_certification_for_test::install_identity_for_test(configuration, seeds, &scope)
                .await
                .map_err(|error| format!("bank identity installation failed: {error}"))?;
        Ok(Self {
            callback,
            fixture,
            endpoints,
            scope,
            identity,
            docker,
        })
    }

    pub fn shutdown(self) -> Result<(), String> {
        let project = self.docker.project_name();
        drop(self);
        DockerIdentityWorld::require_project_absent(&project)
    }
}

async fn require_ordinary_tls_rejection(
    configuration: AuthentikOidcConfiguration,
    scope: &WorthQueryRequestScope,
) -> Result<(), String> {
    match AuthentikBankIdentity::install(configuration, std::iter::empty(), scope).await {
        Err(AuthentikBankIdentityBuildError::Adapter(
            AuthentikOidcAdapterBuildError::DiscoveryUnavailable,
        )) => Ok(()),
        Err(error) => Err(format!(
            "ordinary TLS rejection returned the wrong denial: {error}"
        )),
        Ok(_) => Err(
            "ordinary identity installation trusted the cold self-signed certificate".to_string(),
        ),
    }
}

fn principal_seeds(
    endpoints: &IdentityEndpoints,
    fixture: &IdentityFixture,
) -> Result<Vec<BankPrincipalSeed>, String> {
    fixture
        .participants()
        .iter()
        .enumerate()
        .map(|(index, participant)| {
            let principal_id = u64::try_from(index + 1)
                .ok()
                .and_then(BankPrincipalId::new)
                .ok_or_else(|| "courtroom principal inventory exceeded typed IDs".to_string())?;
            let external_identity = WorthQueryExternalPrincipalIdentity::new(
                endpoints.issuer(),
                participant.username(),
            )
            .map_err(|error| format!("dynamic external identity failed: {error}"))?;
            Ok(BankPrincipalSeed::enabled(principal_id, external_identity))
        })
        .collect()
}

pub fn request_scope(duration: Duration) -> WorthQueryRequestScope {
    let cancellation = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(Some(Instant::now() + duration), cancellation.token())
}

fn configuration(
    endpoints: &IdentityEndpoints,
    fixture: &IdentityFixture,
    callback: &CallbackReceiver,
) -> Result<AuthentikOidcConfiguration, String> {
    let configuration = AuthentikOidcConfiguration::builder()
        .issuer(endpoints.issuer())
        .client_id(fixture.client_id())
        .client_secret(fixture.client_secret())
        .redirect_url(callback.redirect_url())
        .introspection_url(endpoints.introspection_url())
        .revocation_url(endpoints.revocation_url())
        .build()
        .map_err(|error| format!("dynamic Authentik configuration failed: {error}"))?;
    if format!("{configuration:?}").contains(fixture.client_secret()) {
        return Err("adapter configuration debug disclosed the client secret".to_string());
    }
    Ok(configuration)
}
