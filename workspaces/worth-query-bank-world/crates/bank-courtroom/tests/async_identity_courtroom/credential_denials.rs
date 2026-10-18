use bank_domain::model::BankPrincipalId;
use bank_http_adapter::cold_certification_for_test;
use bank_http_adapter::{
    AuthentikBankAuthenticationError, AuthentikBankIdentity, AuthentikOidcConfiguration,
    AuthentikOidcCredential,
};
use bank_server::{BankPrincipalAdmissionError, BankPrincipalSeed};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationDenialKind,
    WorthQueryRequestScope,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

use super::callback::CallbackReceiver;
use super::docker_world::IdentityEndpoints;
use super::fixture::IdentityFixture;

pub async fn prove_real_hostile_credential_denials(
    identity: &AuthentikBankIdentity,
    credential: &AuthentikOidcCredential,
    substitution_source: &AuthentikOidcCredential,
    scope: &WorthQueryRequestScope,
) {
    let forged = cold_certification_for_test::corrupt_signature_for_test(credential.clone())
        .expect("real compact ID token should support signature corruption");
    let forged_error = identity
        .authenticate_credential(forged, scope)
        .await
        .expect_err("a forged signature over a real token must fail");
    assert_adapter_failure(
        forged_error,
        WorthQueryAuthenticationAdapterFailureKind::CredentialRejected,
    );
    assert_eq!(identity.jwks_refresh_count(), 0);

    let wrong_nonce = cold_certification_for_test::mismatch_nonce_for_test(credential.clone());
    let nonce_error = identity
        .authenticate_credential(wrong_nonce, scope)
        .await
        .expect_err("a real token bound to another nonce must fail");
    assert_adapter_failure(
        nonce_error,
        WorthQueryAuthenticationAdapterFailureKind::CredentialRejected,
    );
    assert_eq!(identity.jwks_refresh_count(), 0);

    let malformed =
        cold_certification_for_test::replace_with_malformed_id_token_for_test(credential.clone());
    assert_eq!(
        malformed.expect_err("malformed compact tokens must fail at the adapter boundary"),
        cold_certification_for_test::HostileCredentialErrorForTest::MalformedIdToken
    );

    identity
        .authenticate_credential(substitution_source.clone(), scope)
        .await
        .expect("the real substitution-source credential must be independently valid");
    let crossed = cold_certification_for_test::substitute_access_token_for_test(
        credential.clone(),
        substitution_source,
    );
    let crossed_error = identity
        .authenticate_credential(crossed, scope)
        .await
        .expect_err("a valid ID token and another user's valid access token must not compose");
    assert_adapter_failure(
        crossed_error,
        WorthQueryAuthenticationAdapterFailureKind::BindingMismatch,
    );
}

pub async fn prove_real_wrong_audience_denial(
    credential: AuthentikOidcCredential,
    endpoints: &IdentityEndpoints,
    fixture: &IdentityFixture,
    callback: &CallbackReceiver,
    scope: &WorthQueryRequestScope,
) {
    let configuration = wrong_audience_configuration(endpoints, fixture, callback);
    let external_identity = WorthQueryExternalPrincipalIdentity::new(
        endpoints.issuer(),
        fixture.primary_participant().username(),
    )
    .expect("wrong-audience mapping identity should admit");
    let wrong_audience = cold_certification_for_test::install_identity_for_test(
        configuration,
        [BankPrincipalSeed::enabled(
            BankPrincipalId::new(2).expect("bank principal id should admit"),
            external_identity,
        )],
        Some(endpoints.trusted_root_pem().as_bytes()),
        scope,
    )
    .await
    .expect("wrong-audience adapter should discover the real issuer");
    let error = wrong_audience
        .authenticate_credential(credential.clone(), scope)
        .await
        .expect_err("a real token for another audience must fail");
    assert_adapter_failure(
        error,
        WorthQueryAuthenticationAdapterFailureKind::CredentialRejected,
    );
    assert_eq!(wrong_audience.jwks_refresh_count(), 0);
    prove_real_wrong_issuer_denial(credential, endpoints, fixture, callback, scope).await;
}

pub fn assert_adapter_failure(
    error: AuthentikBankAuthenticationError,
    expected: WorthQueryAuthenticationAdapterFailureKind,
) {
    match error {
        AuthentikBankAuthenticationError::PrincipalAdmission(
            BankPrincipalAdmissionError::Authentication(denial),
        ) => assert_eq!(
            denial.kind(),
            WorthQueryAuthenticationDenialKind::AdapterFailed(expected)
        ),
        other => panic!("unexpected authentication denial: {other:?}"),
    }
}

fn wrong_audience_configuration(
    endpoints: &IdentityEndpoints,
    fixture: &IdentityFixture,
    callback: &CallbackReceiver,
) -> AuthentikOidcConfiguration {
    AuthentikOidcConfiguration::builder()
        .issuer(endpoints.issuer())
        .client_id(format!("{}-wrong-audience", fixture.client_id()))
        .client_secret(fixture.client_secret())
        .redirect_url(callback.redirect_url())
        .introspection_url(endpoints.introspection_url())
        .revocation_url(endpoints.revocation_url())
        .build()
        .expect("wrong-audience certification configuration should be structural")
}

async fn prove_real_wrong_issuer_denial(
    credential: AuthentikOidcCredential,
    endpoints: &IdentityEndpoints,
    fixture: &IdentityFixture,
    callback: &CallbackReceiver,
    scope: &WorthQueryRequestScope,
) {
    let issuer = endpoints.issuer_for(fixture.alternate_slug());
    let configuration = AuthentikOidcConfiguration::builder()
        .issuer(&issuer)
        .client_id(fixture.client_id())
        .client_secret(fixture.client_secret())
        .redirect_url(callback.redirect_url())
        .introspection_url(endpoints.introspection_url())
        .revocation_url(endpoints.revocation_url())
        .build()
        .expect("alternate real issuer configuration should admit");
    let external_identity =
        WorthQueryExternalPrincipalIdentity::new(&issuer, fixture.primary_participant().username())
            .expect("alternate issuer mapping should admit");
    let alternate = cold_certification_for_test::install_identity_for_test(
        configuration,
        [BankPrincipalSeed::enabled(
            BankPrincipalId::new(3).expect("bank principal id should admit"),
            external_identity,
        )],
        Some(endpoints.trusted_root_pem().as_bytes()),
        scope,
    )
    .await
    .expect("alternate real issuer should discover");
    let error = alternate
        .authenticate_credential(credential, scope)
        .await
        .expect_err("a token from another real issuer must fail");
    assert_adapter_failure(
        error,
        WorthQueryAuthenticationAdapterFailureKind::CredentialRejected,
    );
    assert_eq!(
        alternate.jwks_refresh_count(),
        0,
        "a known-key issuer mismatch must not refresh JWKS"
    );
}
