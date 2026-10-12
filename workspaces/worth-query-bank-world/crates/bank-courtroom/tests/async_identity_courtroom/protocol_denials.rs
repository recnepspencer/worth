use std::time::{Duration, Instant};

use bank_http_adapter::{AuthentikBankIdentity, AuthentikOidcFlowError};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestInterruption, WorthQueryRequestScope,
};

use super::browser::{
    complete_browser_authorization, complete_browser_authorization_after_response_loss,
};
use super::callback::CallbackReceiver;
use super::fixture::IdentityFixture;
use super::installed_identity::request_scope;

pub async fn prove_real_state_and_interruption_denials(
    identity: &AuthentikBankIdentity,
    webdriver_url: &str,
    fixture: &IdentityFixture,
    callback: &CallbackReceiver,
) {
    prove_callback_response_loss(identity, webdriver_url, fixture, callback).await;
    prove_state_denial(identity, webdriver_url, fixture, callback).await;
    prove_deadline_denial(identity, webdriver_url, fixture, callback).await;
    prove_cancellation_denial(identity, webdriver_url, fixture, callback).await;
}

async fn prove_callback_response_loss(
    identity: &AuthentikBankIdentity,
    webdriver_url: &str,
    fixture: &IdentityFixture,
    callback: &CallbackReceiver,
) {
    let authorization = identity.begin_authorization().await;
    let url = authorization.authorization_url().to_string();
    let pending = authorization.into_pending();
    let callback_value = complete_browser_authorization_after_response_loss(
        webdriver_url,
        &url,
        fixture.primary_participant(),
        callback,
    )
    .await
    .expect("real callback request must survive response loss")
    .into_authentik()
    .expect("response-loss callback should parse");
    let scope = request_scope(Duration::from_secs(60));
    let credential = identity
        .finish_authorization(pending, callback_value, &scope)
        .await
        .expect("callback response loss must not invalidate the authorization code");
    identity
        .authenticate_credential(credential, &scope)
        .await
        .expect("response-loss authorization must still admit its principal");
}

async fn prove_state_denial(
    identity: &AuthentikBankIdentity,
    webdriver_url: &str,
    fixture: &IdentityFixture,
    callback: &CallbackReceiver,
) {
    let authorization = identity.begin_authorization().await;
    let url = authorization.authorization_url().to_string();
    let pending = authorization.into_pending();
    let callback_value = complete_browser_authorization(
        webdriver_url,
        &url,
        fixture.primary_participant(),
        callback,
    )
    .await
    .expect("real hostile-state browser flow should return");
    let error = identity
        .finish_authorization(
            pending,
            callback_value
                .with_state("structurally-valid-wrong-state")
                .expect("hostile state should remain structurally valid"),
            &request_scope(Duration::from_secs(60)),
        )
        .await
        .expect_err("wrong state must fail before token exchange");
    assert_eq!(error, AuthentikOidcFlowError::StateMismatch);
}

async fn prove_deadline_denial(
    identity: &AuthentikBankIdentity,
    webdriver_url: &str,
    fixture: &IdentityFixture,
    callback: &CallbackReceiver,
) {
    let authorization = identity.begin_authorization().await;
    let url = authorization.authorization_url().to_string();
    let pending = authorization.into_pending();
    let callback_value = complete_browser_authorization(
        webdriver_url,
        &url,
        fixture.primary_participant(),
        callback,
    )
    .await
    .expect("real deadline browser flow should return")
    .into_authentik()
    .expect("real deadline callback should parse");
    let error = identity
        .finish_authorization(pending, callback_value, &request_scope(Duration::ZERO))
        .await
        .expect_err("expired request scope must stop token exchange");
    assert_eq!(
        error,
        AuthentikOidcFlowError::RequestInterrupted(WorthQueryRequestInterruption::DeadlineExceeded)
    );
}

async fn prove_cancellation_denial(
    identity: &AuthentikBankIdentity,
    webdriver_url: &str,
    fixture: &IdentityFixture,
    callback: &CallbackReceiver,
) {
    let authorization = identity.begin_authorization().await;
    let url = authorization.authorization_url().to_string();
    let pending = authorization.into_pending();
    let callback_value = complete_browser_authorization(
        webdriver_url,
        &url,
        fixture.primary_participant(),
        callback,
    )
    .await
    .expect("real cancellation browser flow should return")
    .into_authentik()
    .expect("real cancellation callback should parse");
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let error = identity
        .finish_authorization(pending, callback_value, &scope)
        .await
        .expect_err("cancelled request scope must stop token exchange");
    assert_eq!(
        error,
        AuthentikOidcFlowError::RequestInterrupted(WorthQueryRequestInterruption::Cancelled)
    );
}
