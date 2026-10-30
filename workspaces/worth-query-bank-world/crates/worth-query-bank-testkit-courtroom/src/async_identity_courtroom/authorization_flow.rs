pub use bank_http_adapter::{AuthentikBankIdentity, AuthentikOidcCredential};
pub use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

pub use super::browser::complete_browser_authorization;
pub use super::callback::CallbackReceiver;
pub use super::fixture::{IdentityFixture, IdentityParticipant};

pub async fn acquire_browser_credential(
    identity: &AuthentikBankIdentity,
    webdriver_url: &str,
    fixture: &IdentityFixture,
    callback: &CallbackReceiver,
    scope: &WorthQueryRequestScope,
) -> Result<AuthentikOidcCredential, String> {
    acquire_browser_credential_for(
        identity,
        webdriver_url,
        fixture.primary_participant(),
        callback,
        scope,
    )
    .await
}

pub async fn acquire_browser_credential_for(
    identity: &AuthentikBankIdentity,
    webdriver_url: &str,
    participant: &IdentityParticipant,
    callback: &CallbackReceiver,
    scope: &WorthQueryRequestScope,
) -> Result<AuthentikOidcCredential, String> {
    let authorization = identity.begin_authorization().await;
    let authorization_url = authorization.authorization_url().to_string();
    let pending = authorization.into_pending();
    let callback_result =
        complete_browser_authorization(webdriver_url, &authorization_url, participant, callback)
            .await?
            .into_authentik()?;
    identity
        .finish_authorization(pending, callback_result, scope)
        .await
        .map_err(|error| format!("authorization code exchange failed: {error}"))
}
