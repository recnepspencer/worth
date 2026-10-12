use std::time::{Duration, Instant};

use bank_domain::model::BankPrincipalId;
use bank_http_adapter::{
    cold_certification_for_test, AuthentikBankAuthenticationError, AuthentikBankIdentity,
    AuthentikOidcConfiguration, AuthentikOidcCredential,
};
use bank_server::BankPrincipalSeed;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryRequestScope,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

use super::administration::AuthentikAdministration;
use super::authorization_flow::acquire_browser_credential;
use super::credential_denials::assert_adapter_failure;
use super::installed_identity::InstalledIdentityWorld;

pub async fn prove_rotation_revocation_and_expiration(world: &InstalledIdentityWorld) {
    let administration = AuthentikAdministration::new(
        world.endpoints.authentik_origin(),
        world.fixture.bootstrap_token().to_string(),
    )
    .expect("real Authentik administration should configure");
    prove_unknown_key_retry_ceiling(world, &administration).await;
    administration
        .rotate_provider_signing_key(world.fixture.slug())
        .await
        .expect("real Authentik provider signing key should rotate");
    let rotated = acquire_browser_credential(
        &world.identity,
        &world.endpoints.webdriver_url(),
        &world.fixture,
        &world.callback,
        &world.scope,
    )
    .await
    .expect("rotated real browser authorization should exchange");
    world
        .identity
        .authenticate_credential(rotated.clone(), &world.scope)
        .await
        .expect("one bounded JWKS refresh should admit the rotated key");
    assert_eq!(world.identity.jwks_refresh_count(), 1);
    prove_revocation(world, rotated).await;
    prove_expiration(world, &administration).await;
}

async fn prove_unknown_key_retry_ceiling(
    world: &InstalledIdentityWorld,
    administration: &AuthentikAdministration,
) {
    let stale = install_identity(
        world,
        world.endpoints.issuer(),
        world.fixture.client_id(),
        world.fixture.client_secret(),
        10,
    )
    .await;
    administration
        .rotate_provider_signing_key(world.fixture.slug())
        .await
        .expect("primary provider should install a displaced signing key");
    let displaced = acquire_browser_credential(
        &stale,
        &world.endpoints.webdriver_url(),
        &world.fixture,
        &world.callback,
        &world.scope,
    )
    .await
    .expect("displaced-key browser authorization should exchange");
    administration
        .rotate_provider_signing_key(world.fixture.slug())
        .await
        .expect("primary provider should replace the displaced signing key");
    let error = stale
        .authenticate_credential(displaced, &world.scope)
        .await
        .expect_err("a key absent after one primary JWKS refresh must fail");
    assert_adapter_failure(
        error,
        WorthQueryAuthenticationAdapterFailureKind::CredentialRejected,
    );
    assert_eq!(
        stale.jwks_refresh_count(),
        1,
        "an unresolved key must receive one refresh attempt, never a loop"
    );
}

async fn install_identity(
    world: &InstalledIdentityWorld,
    issuer: String,
    client_id: &str,
    client_secret: &str,
    principal_id: u64,
) -> AuthentikBankIdentity {
    let configuration = AuthentikOidcConfiguration::builder()
        .issuer(&issuer)
        .client_id(client_id)
        .client_secret(client_secret)
        .redirect_url(world.callback.redirect_url())
        .introspection_url(world.endpoints.introspection_url())
        .revocation_url(world.endpoints.revocation_url())
        .build()
        .expect("real certification identity configuration should admit");
    let external = WorthQueryExternalPrincipalIdentity::new(
        &issuer,
        world.fixture.primary_participant().username(),
    )
    .expect("real certification mapping should admit");
    cold_certification_for_test::install_identity_for_test(
        configuration,
        [BankPrincipalSeed::enabled(
            BankPrincipalId::new(principal_id).expect("certification principal id should admit"),
            external,
        )],
        &world.scope,
    )
    .await
    .expect("real certification identity should install")
}

async fn prove_revocation(world: &InstalledIdentityWorld, credential: AuthentikOidcCredential) {
    world
        .identity
        .revoke_credential(&credential, &world.scope)
        .await
        .expect("real Authentik revocation should accept its token");
    let error = world
        .identity
        .authenticate_credential(credential, &world.scope)
        .await
        .expect_err("online introspection must reject the revoked access token");
    assert_adapter_failure(
        error,
        WorthQueryAuthenticationAdapterFailureKind::CredentialRevoked,
    );
}

async fn prove_expiration(
    world: &InstalledIdentityWorld,
    administration: &AuthentikAdministration,
) {
    administration
        .set_access_token_validity(world.fixture.slug(), "seconds=1")
        .await
        .expect("real Authentik provider should accept a bounded token lifetime");
    let credential = acquire_browser_credential(
        &world.identity,
        &world.endpoints.webdriver_url(),
        &world.fixture,
        &world.callback,
        &world.scope,
    )
    .await
    .expect("short-lived real browser authorization should exchange");
    let error = await_real_expiration(&world.identity, credential, &world.scope).await;
    assert_adapter_failure(
        error,
        WorthQueryAuthenticationAdapterFailureKind::CredentialExpired,
    );
}

async fn await_real_expiration(
    identity: &AuthentikBankIdentity,
    credential: AuthentikOidcCredential,
    scope: &WorthQueryRequestScope,
) -> AuthentikBankAuthenticationError {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match identity
            .authenticate_credential(credential.clone(), scope)
            .await
        {
            Err(error) => return error,
            Ok(_) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Ok(_) => panic!("real one-second token did not expire before the five-second ceiling"),
        }
    }
}
