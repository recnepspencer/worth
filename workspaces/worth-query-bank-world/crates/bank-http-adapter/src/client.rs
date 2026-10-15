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

pub(crate) fn cold_certification_http_client_for_test(
    trusted_root_pem: Option<&[u8]>,
) -> Result<reqwest::Client, AuthentikOidcAdapterBuildError> {
    let pem = trusted_root_pem
        .ok_or(AuthentikOidcAdapterBuildError::MissingColdCertificationTrustedRoot)?;
    let mut roots = reqwest::Certificate::from_pem_bundle(pem)
        .map_err(|_| AuthentikOidcAdapterBuildError::InvalidColdCertificationTrustedRoot)?;
    if roots.len() != 1 {
        return Err(AuthentikOidcAdapterBuildError::InvalidColdCertificationTrustedRoot);
    }
    let root = roots.pop().expect("one named root certificate");
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .add_root_certificate(root)
        .build()
        .map_err(|_| AuthentikOidcAdapterBuildError::InvalidColdCertificationTrustedRoot)
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
            super::cold_certification_http_client_for_test(None),
            Err(super::AuthentikOidcAdapterBuildError::MissingColdCertificationTrustedRoot)
        ));
    }
    #[test]
    fn valid_named_cold_trusted_root_constructs_validating_client() {
        assert!(
            super::cold_certification_http_client_for_test(Some(include_bytes!(
                "client/trusted_root.pem"
            )))
            .is_ok()
        );
    }
    #[test]
    fn malformed_cold_trusted_root_is_refused() {
        let invalid_der = b"-----BEGIN CERTIFICATE-----\nAQID\n-----END CERTIFICATE-----\n";
        assert!(matches!(
            super::cold_certification_http_client_for_test(Some(invalid_der)),
            Err(super::AuthentikOidcAdapterBuildError::InvalidColdCertificationTrustedRoot)
        ));
        assert!(matches!(
            super::cold_certification_http_client_for_test(Some(b"not a certificate")),
            Err(super::AuthentikOidcAdapterBuildError::InvalidColdCertificationTrustedRoot)
        ));
    }
}
