//! Explicit cold-courtroom installation for a user node.
//!
//! The ordinary node always uses strict TLS discovery. This feature-owned path
//! changes only trust of the courtroom's ephemeral self-signed certificate;
//! the real OIDC and Bank authentication boundaries remain in force.

use std::time::Instant;

use bank_http_adapter::cold_certification_for_test;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use crate::{
    BankUserNode, BankUserNodeBinding, BankUserNodeConfiguration, BankUserNodeInstallError,
};

pub async fn install_for_test(
    binding: BankUserNodeBinding,
    configuration: BankUserNodeConfiguration,
    external_redirect_url: String,
    trusted_root_pem: Option<&[u8]>,
) -> Result<BankUserNode, BankUserNodeInstallError> {
    let oidc_configuration = binding
        .oidc_configuration_with_redirect(&configuration, external_redirect_url)
        .map_err(BankUserNodeInstallError::OidcConfiguration)?;
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + configuration.maximum_deadline),
        cancellation.token(),
    );
    let oidc = cold_certification_for_test::discover_adapter_for_test(
        oidc_configuration,
        trusted_root_pem,
        &scope,
    )
    .await
    .map_err(BankUserNodeInstallError::OidcDiscovery)?;
    binding.install_with_adapter(configuration, oidc)
}
