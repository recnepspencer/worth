use std::time::SystemTime;

use crate::configuration::AuthentikIssuer;
use openidconnect::{
    AccessTokenHash, ClaimsVerificationError, SignatureVerificationError,
    TokenIntrospectionResponse,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAdapterFailure, WorthQueryAuthenticationAdapterFailureKind,
    WorthQueryPrincipalAttribute, WorthQueryRequestInterruption, WorthQueryRequestScope,
    WorthQueryValidatedExternalPrincipal,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

use crate::adapter::AuthentikOidcAdapter;
use crate::client::{discover_client, AuthentikClient};
use crate::credential::AuthentikOidcCredential;
use crate::scope::await_in_scope;

pub(crate) async fn validate_credential(
    adapter: &AuthentikOidcAdapter,
    credential: AuthentikOidcCredential,
    scope: &WorthQueryRequestScope,
) -> Result<WorthQueryValidatedExternalPrincipal, WorthQueryAuthenticationAdapterFailure> {
    let local = {
        let client = adapter.client.read().await;
        validate_id_token(&client, &credential)
    };
    let mut local = match local {
        Err(LocalValidationError::MissingSigningKey) => {
            validate_after_missing_key(adapter, &credential, scope).await
        }
        result => result,
    }
    .map_err(LocalValidationError::failure)?;
    require_unexpired(local.expires_at, sampled_wall()?)?;

    let introspection = {
        let client = adapter.client.read().await;
        let request = client
            .introspect(&credential.access_token)
            .request_async(&adapter.http_client);
        await_in_scope(scope, request)
            .await
            .map_err(interruption_failure)?
            .map_err(|_| {
                failure(WorthQueryAuthenticationAdapterFailureKind::DependencyUnavailable)
            })?
    };
    let validated_at = sampled_wall()?;
    require_unexpired(local.expires_at, validated_at)?;
    validate_introspection(adapter, &local, &introspection)?;
    if let Some(introspection_expiry) = introspection.exp().map(SystemTime::from) {
        local.expires_at = local.expires_at.min(introspection_expiry);
    }
    require_unexpired(local.expires_at, validated_at)?;

    WorthQueryValidatedExternalPrincipal::new(
        WorthQueryExternalPrincipalIdentity::new(AuthentikIssuer, local.subject)
            .map_err(|_| failure(WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation))?,
        adapter
            .configuration
            .authentication_audience()
            .map_err(|_| failure(WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation))?,
        adapter
            .configuration
            .authentication_method()
            .map_err(|_| failure(WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation))?,
        validated_at,
        local.expires_at,
        local.attributes,
    )
    .map_err(|_| failure(WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation))
}

fn sampled_wall() -> Result<SystemTime, WorthQueryAuthenticationAdapterFailure> {
    worth_execution::time::system_time_sampler()
        .system_time()
        .map_err(|_| failure(WorthQueryAuthenticationAdapterFailureKind::DependencyUnavailable))
}

fn require_unexpired(
    expires_at: SystemTime,
    validated_at: SystemTime,
) -> Result<(), WorthQueryAuthenticationAdapterFailure> {
    (expires_at > validated_at)
        .then_some(())
        .ok_or_else(|| failure(WorthQueryAuthenticationAdapterFailureKind::CredentialExpired))
}

fn validate_introspection(
    adapter: &AuthentikOidcAdapter,
    local: &LocallyValidatedIdentity,
    introspection: &openidconnect::core::CoreTokenIntrospectionResponse,
) -> Result<(), WorthQueryAuthenticationAdapterFailure> {
    if !introspection.active() {
        return Err(failure(
            WorthQueryAuthenticationAdapterFailureKind::CredentialRevoked,
        ));
    }
    let issuer_matches = introspection
        .iss()
        .is_some_and(|issuer| issuer == local.issuer);
    let subject_matches = introspection
        .sub()
        .is_some_and(|subject| subject == local.subject);
    let binding_mismatch = introspection.client_id() != Some(&adapter.configuration.client_id)
        || !issuer_matches
        || !subject_matches
        || introspection.aud().is_some_and(|audiences| {
            !audiences
                .iter()
                .any(|audience| audience == adapter.configuration.client_id.as_str())
        });
    if binding_mismatch {
        return Err(failure(
            WorthQueryAuthenticationAdapterFailureKind::BindingMismatch,
        ));
    }
    Ok(())
}

async fn validate_after_missing_key(
    adapter: &AuthentikOidcAdapter,
    credential: &AuthentikOidcCredential,
    scope: &WorthQueryRequestScope,
) -> Result<LocallyValidatedIdentity, LocalValidationError> {
    let _refresh_guard = await_in_scope(scope, adapter.jwks_refresh.lock())
        .await
        .map_err(LocalValidationError::Interrupted)?;
    {
        let client = adapter.client.read().await;
        match validate_id_token(&client, credential) {
            Ok(validated) => return Ok(validated),
            Err(LocalValidationError::MissingSigningKey) => {}
            Err(other) => return Err(other),
        }
    }
    let refreshed = discover_client(&adapter.configuration, &adapter.http_client, scope)
        .await
        .map_err(|error| match error {
            crate::error::AuthentikOidcAdapterBuildError::DiscoveryInterrupted(interruption) => {
                LocalValidationError::Interrupted(interruption)
            }
            _ => LocalValidationError::DependencyUnavailable,
        })?;
    *adapter.client.write().await = refreshed;
    adapter
        .jwks_refresh_count
        .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    let client = adapter.client.read().await;
    validate_id_token(&client, credential)
}

fn validate_id_token(
    client: &AuthentikClient,
    credential: &AuthentikOidcCredential,
) -> Result<LocallyValidatedIdentity, LocalValidationError> {
    let verifier = client
        .id_token_verifier()
        .set_allowed_algs(AuthentikOidcAdapter::allowed_signing_algorithms());
    let claims = credential
        .id_token
        .claims(&verifier, &credential.nonce)
        .map_err(LocalValidationError::from_claims)?;
    if let Some(expected_hash) = claims.access_token_hash() {
        let actual_hash = AccessTokenHash::from_token(
            &credential.access_token,
            credential
                .id_token
                .signing_alg()
                .map_err(|_| LocalValidationError::Rejected)?,
            credential
                .id_token
                .signing_key(&verifier)
                .map_err(|_| LocalValidationError::Rejected)?,
        )
        .map_err(|_| LocalValidationError::Rejected)?;
        if &actual_hash != expected_hash {
            return Err(LocalValidationError::Rejected);
        }
    }
    Ok(LocallyValidatedIdentity {
        issuer: claims.issuer().as_str().to_string(),
        subject: claims.subject().as_str().to_string(),
        expires_at: SystemTime::from(claims.expiration().to_owned()),
        attributes: display_attributes(claims)?,
    })
}

fn display_attributes(
    claims: &openidconnect::core::CoreIdTokenClaims,
) -> Result<Vec<WorthQueryPrincipalAttribute>, LocalValidationError> {
    let mut attributes = Vec::new();
    if let Some(email) = claims.email() {
        attributes.push(
            WorthQueryPrincipalAttribute::new("email", email.as_str())
                .map_err(|_| LocalValidationError::Rejected)?,
        );
    }
    if let Some(username) = claims.preferred_username() {
        attributes.push(
            WorthQueryPrincipalAttribute::new("preferred_username", username.as_str())
                .map_err(|_| LocalValidationError::Rejected)?,
        );
    }
    Ok(attributes)
}

struct LocallyValidatedIdentity {
    issuer: String,
    subject: String,
    expires_at: SystemTime,
    attributes: Vec<WorthQueryPrincipalAttribute>,
}

enum LocalValidationError {
    MissingSigningKey,
    Expired,
    Rejected,
    DependencyUnavailable,
    Interrupted(WorthQueryRequestInterruption),
}

impl LocalValidationError {
    fn from_claims(error: ClaimsVerificationError) -> Self {
        match error {
            ClaimsVerificationError::SignatureVerification(
                SignatureVerificationError::NoMatchingKey,
            ) => Self::MissingSigningKey,
            ClaimsVerificationError::Expired(_) => Self::Expired,
            _ => Self::Rejected,
        }
    }

    fn failure(self) -> WorthQueryAuthenticationAdapterFailure {
        match self {
            Self::Expired => failure(WorthQueryAuthenticationAdapterFailureKind::CredentialExpired),
            Self::DependencyUnavailable => {
                failure(WorthQueryAuthenticationAdapterFailureKind::DependencyUnavailable)
            }
            Self::Interrupted(interruption) => interruption_failure(interruption),
            Self::MissingSigningKey | Self::Rejected => {
                failure(WorthQueryAuthenticationAdapterFailureKind::CredentialRejected)
            }
        }
    }
}

fn interruption_failure(
    interruption: WorthQueryRequestInterruption,
) -> WorthQueryAuthenticationAdapterFailure {
    failure(match interruption {
        WorthQueryRequestInterruption::Cancelled => {
            WorthQueryAuthenticationAdapterFailureKind::Cancelled
        }
        WorthQueryRequestInterruption::DeadlineExceeded => {
            WorthQueryAuthenticationAdapterFailureKind::DeadlineExceeded
        }
    })
}

fn failure(
    kind: WorthQueryAuthenticationAdapterFailureKind,
) -> WorthQueryAuthenticationAdapterFailure {
    WorthQueryAuthenticationAdapterFailure::new(kind)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn signed_expiry_has_a_specific_boundary_denial() {
        let validated_at = SystemTime::UNIX_EPOCH + Duration::from_secs(10);
        assert_eq!(
            require_unexpired(validated_at, validated_at)
                .expect_err("expiry equality must deny")
                .kind(),
            WorthQueryAuthenticationAdapterFailureKind::CredentialExpired
        );
        assert!(
            require_unexpired(validated_at + Duration::from_secs(1), validated_at).is_ok(),
            "strictly future expiry must remain admissible"
        );
    }
}
