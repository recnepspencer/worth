use std::sync::atomic::{AtomicUsize, Ordering};

use openidconnect::core::{CoreAuthenticationFlow, CoreJwsSigningAlgorithm, CoreRevocableToken};
use openidconnect::reqwest;
use openidconnect::{
    CsrfToken, Nonce, OAuth2TokenResponse, PkceCodeChallenge, Scope, TokenResponse,
};
use tokio::sync::{Mutex, RwLock};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAdapter, WorthQueryAuthenticationFuture, WorthQueryRequestScope,
};

use crate::authorization::{
    AuthentikAuthorizationCallback, AuthentikAuthorizationRequest, AuthentikPendingAuthorization,
};
use crate::client::{discover_client, http_client, AuthentikClient};
use crate::configuration::AuthentikOidcConfiguration;
use crate::credential::AuthentikOidcCredential;
use crate::error::{AuthentikOidcAdapterBuildError, AuthentikOidcFlowError};
use crate::scope::await_in_scope;
use crate::validation::validate_credential;

pub struct AuthentikOidcAdapter {
    pub(crate) configuration: AuthentikOidcConfiguration,
    pub(crate) http_client: reqwest::Client,
    pub(crate) client: RwLock<AuthentikClient>,
    pub(crate) jwks_refresh: Mutex<()>,
    pub(crate) jwks_refresh_count: AtomicUsize,
}

impl AuthentikOidcAdapter {
    pub async fn discover(
        configuration: AuthentikOidcConfiguration,
        scope: &WorthQueryRequestScope,
    ) -> Result<Self, AuthentikOidcAdapterBuildError> {
        let http_client = http_client()?;
        Self::discover_with_http_client(configuration, http_client, scope).await
    }

    pub(crate) async fn discover_for_cold_certification_for_test(
        configuration: AuthentikOidcConfiguration,
        scope: &WorthQueryRequestScope,
    ) -> Result<Self, AuthentikOidcAdapterBuildError> {
        let http_client = crate::client::cold_certification_http_client_for_test()?;
        Self::discover_with_http_client(configuration, http_client, scope).await
    }

    async fn discover_with_http_client(
        configuration: AuthentikOidcConfiguration,
        http_client: reqwest::Client,
        scope: &WorthQueryRequestScope,
    ) -> Result<Self, AuthentikOidcAdapterBuildError> {
        let client = discover_client(&configuration, &http_client, scope).await?;
        Ok(Self {
            configuration,
            http_client,
            client: RwLock::new(client),
            jwks_refresh: Mutex::new(()),
            jwks_refresh_count: AtomicUsize::new(0),
        })
    }

    pub async fn begin_authorization(&self) -> AuthentikAuthorizationRequest {
        let client = self.client.read().await;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorization_url, state, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        AuthentikAuthorizationRequest::new(
            authorization_url.to_string(),
            AuthentikPendingAuthorization {
                state,
                nonce,
                pkce_verifier,
            },
        )
    }

    pub async fn finish_authorization(
        &self,
        pending: AuthentikPendingAuthorization,
        callback: AuthentikAuthorizationCallback,
        scope: &WorthQueryRequestScope,
    ) -> Result<AuthentikOidcCredential, AuthentikOidcFlowError> {
        if pending.state != callback.state {
            return Err(AuthentikOidcFlowError::StateMismatch);
        }
        let client = self.client.read().await;
        let exchange = client
            .exchange_code(callback.code)
            .set_pkce_verifier(pending.pkce_verifier)
            .request_async(&self.http_client);
        let response = await_in_scope(scope, exchange)
            .await
            .map_err(AuthentikOidcFlowError::RequestInterrupted)?
            .map_err(|_| AuthentikOidcFlowError::TokenExchangeRejected)?;
        let id_token = response
            .id_token()
            .cloned()
            .ok_or(AuthentikOidcFlowError::MissingIdToken)?;
        Ok(AuthentikOidcCredential {
            id_token,
            access_token: response.access_token().clone(),
            nonce: pending.nonce,
        })
    }

    pub fn jwks_refresh_count(&self) -> usize {
        self.jwks_refresh_count.load(Ordering::Acquire)
    }

    pub async fn revoke_credential(
        &self,
        credential: &AuthentikOidcCredential,
        scope: &WorthQueryRequestScope,
    ) -> Result<(), AuthentikOidcFlowError> {
        let client = self.client.read().await;
        let request = client
            .revoke_token(CoreRevocableToken::AccessToken(
                credential.access_token.clone(),
            ))
            .map_err(|_| AuthentikOidcFlowError::RevocationRejected)?
            .request_async(&self.http_client);
        await_in_scope(scope, request)
            .await
            .map_err(AuthentikOidcFlowError::RequestInterrupted)?
            .map_err(|_| AuthentikOidcFlowError::RevocationRejected)
    }

    pub(crate) fn allowed_signing_algorithms() -> [CoreJwsSigningAlgorithm; 1] {
        [CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256]
    }
}

impl WorthQueryAuthenticationAdapter for AuthentikOidcAdapter {
    type Credential = AuthentikOidcCredential;

    fn configuration_identity(&self) -> &str {
        self.configuration.configuration_identity()
    }

    fn validate<'a>(
        &'a self,
        credential: Self::Credential,
        scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move { validate_credential(self, credential, scope).await })
    }
}
