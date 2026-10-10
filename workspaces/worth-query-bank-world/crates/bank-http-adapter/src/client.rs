use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::reqwest;
use openidconnect::{EndpointMaybeSet, EndpointNotSet, EndpointSet};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

use crate::configuration::AuthentikOidcConfiguration;
use crate::error::AuthentikOidcAdapterBuildError;
use crate::scope::await_in_scope;

pub(crate) type AuthentikClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointSet,
    EndpointSet,
    EndpointSet,
    EndpointMaybeSet,
>;

pub(crate) fn http_client() -> Result<reqwest::Client, AuthentikOidcAdapterBuildError> {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| AuthentikOidcAdapterBuildError::HttpClient)
}

pub(crate) fn cold_certification_http_client(
) -> Result<reqwest::Client, AuthentikOidcAdapterBuildError> {
    // A cold court must hand us its named TLS root. The current court supplies
    // only issuer URLs, so fail closed until that certificate handoff exists.
    Err(AuthentikOidcAdapterBuildError::MissingColdCertificationTrustedRoot)
}

pub(crate) async fn discover_client(
    configuration: &AuthentikOidcConfiguration,
    http_client: &reqwest::Client,
    scope: &WorthQueryRequestScope,
) -> Result<AuthentikClient, AuthentikOidcAdapterBuildError> {
    let discovery = CoreProviderMetadata::discover_async(configuration.issuer.clone(), http_client);
    let metadata = await_in_scope(scope, discovery)
        .await
        .map_err(AuthentikOidcAdapterBuildError::DiscoveryInterrupted)?
        .map_err(|_| AuthentikOidcAdapterBuildError::DiscoveryUnavailable)?;
    let token_endpoint = metadata
        .token_endpoint()
        .cloned()
        .ok_or(AuthentikOidcAdapterBuildError::MissingTokenEndpoint)?;
    Ok(CoreClient::from_provider_metadata(
        metadata,
        configuration.client_id.clone(),
        Some(configuration.client_secret.clone()),
    )
    .set_token_uri(token_endpoint)
    .set_redirect_uri(configuration.redirect_url.clone())
    .set_introspection_url(configuration.introspection_url.clone())
    .set_revocation_url(configuration.revocation_url.clone()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn cold_certification_without_its_trusted_root_is_refused() {
        assert!(matches!(
            super::cold_certification_http_client(),
            Err(super::AuthentikOidcAdapterBuildError::MissingColdCertificationTrustedRoot)
        ));
    }
}
