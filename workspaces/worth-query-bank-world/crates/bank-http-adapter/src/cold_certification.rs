//! Cold hostile transforms for the external Docker courtroom.
//!
//! These operations never mint authenticated Query authority and never expose
//! token text. These compiled doors are sealed to the owner and named certification callers.

use openidconnect::Nonce;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

use crate::{
    AuthentikBankIdentity, AuthentikBankIdentityBuildError, AuthentikOidcAdapter,
    AuthentikOidcAdapterBuildError, AuthentikOidcConfiguration, AuthentikOidcCredential,
};

/// Discovers the same OIDC adapter as production while trusting the cold
/// courtroom's named ephemeral certificate authority.
///
/// This does not skip discovery, issuer, audience, signature, nonce, token
/// activity, or principal checks. It changes only the TLS root posture and is
/// sealed to the owner and named certification callers. Missing root material is refused.
pub async fn discover_adapter_for_test(
    configuration: AuthentikOidcConfiguration,
    trusted_root_pem: Option<&[u8]>,
    scope: &WorthQueryRequestScope,
) -> Result<AuthentikOidcAdapter, AuthentikOidcAdapterBuildError> {
    AuthentikOidcAdapter::discover_for_cold_certification_for_test(
        configuration,
        trusted_root_pem,
        scope,
    )
    .await
}

pub async fn install_identity_for_test(
    configuration: AuthentikOidcConfiguration,
    seeds: impl IntoIterator<Item = bank_server::BankPrincipalSeed>,
    trusted_root_pem: Option<&[u8]>,
    scope: &WorthQueryRequestScope,
) -> Result<AuthentikBankIdentity, AuthentikBankIdentityBuildError> {
    AuthentikBankIdentity::install_for_cold_certification_for_test(
        configuration,
        seeds,
        trusted_root_pem,
        scope,
    )
    .await
}

pub async fn install_world_for_test(
    configuration: AuthentikOidcConfiguration,
    seed: bank_server::BankWorldSeed,
    trusted_root_pem: Option<&[u8]>,
    scope: &WorthQueryRequestScope,
) -> Result<AuthentikBankIdentity, AuthentikBankIdentityBuildError> {
    AuthentikBankIdentity::install_world_for_cold_certification_for_test(
        configuration,
        seed,
        trusted_root_pem,
        scope,
    )
    .await
}

pub fn corrupt_signature_for_test(
    mut credential: AuthentikOidcCredential,
) -> Result<AuthentikOidcCredential, HostileCredentialErrorForTest> {
    let serialized = credential.id_token.to_string();
    let (signing_input, signature) = serialized
        .rsplit_once('.')
        .ok_or(HostileCredentialErrorForTest::MalformedIdToken)?;
    let first = signature
        .chars()
        .next()
        .ok_or(HostileCredentialErrorForTest::MalformedIdToken)?;
    let replacement = if first == 'A' { 'B' } else { 'A' };
    let mut corrupted_signature = signature.to_string();
    corrupted_signature.replace_range(0..first.len_utf8(), &replacement.to_string());
    credential.id_token = format!("{signing_input}.{corrupted_signature}")
        .parse()
        .map_err(|_| HostileCredentialErrorForTest::MalformedIdToken)?;
    Ok(credential)
}

pub fn mismatch_nonce_for_test(mut credential: AuthentikOidcCredential) -> AuthentikOidcCredential {
    credential.nonce = Nonce::new("cold-certification-mismatched-nonce".to_string());
    credential
}

pub fn substitute_access_token_for_test(
    mut identity_credential: AuthentikOidcCredential,
    access_token_source: &AuthentikOidcCredential,
) -> AuthentikOidcCredential {
    identity_credential.access_token = access_token_source.access_token.clone();
    identity_credential
}

pub fn replace_with_malformed_id_token_for_test(
    mut credential: AuthentikOidcCredential,
) -> Result<AuthentikOidcCredential, HostileCredentialErrorForTest> {
    credential.id_token = "not.a.valid-json-web-token"
        .parse()
        .map_err(|_| HostileCredentialErrorForTest::MalformedIdToken)?;
    Ok(credential)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostileCredentialErrorForTest {
    MalformedIdToken,
}

impl std::fmt::Display for HostileCredentialErrorForTest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("hostile credential is not a structurally valid ID token")
    }
}

impl std::error::Error for HostileCredentialErrorForTest {}
