use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;

static SIGNING_KEY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub struct AuthentikAdministration {
    client: reqwest::Client,
    api_origin: String,
    bootstrap_token: String,
}

impl AuthentikAdministration {
    pub fn new(
        api_origin: String,
        bootstrap_token: String,
        trusted_root_pem: &[u8],
    ) -> Result<Self, String> {
        let root = reqwest::Certificate::from_pem(trusted_root_pem)
            .map_err(|error| format!("administration named root invalid: {error}"))?;
        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .add_root_certificate(root)
            .build()
            .map_err(|error| format!("Authentik administration client failed: {error}"))?;
        Ok(Self {
            client,
            api_origin,
            bootstrap_token,
        })
    }

    pub async fn rotate_provider_signing_key(&self, provider_name: &str) -> Result<(), String> {
        let key = self.generate_signing_key(provider_name).await?;
        let provider = self.provider(provider_name).await?;
        let response = self
            .client
            .patch(format!(
                "{}/api/v3/providers/oauth2/{}/",
                self.api_origin, provider.pk
            ))
            .bearer_auth(&self.bootstrap_token)
            .json(&SigningKeyPatch {
                signing_key: &key.pk,
            })
            .send()
            .await
            .map_err(|error| format!("provider key rotation request failed: {error}"))?;
        require_success(response, "provider key rotation").await
    }

    pub async fn set_access_token_validity(
        &self,
        provider_name: &str,
        validity: &str,
    ) -> Result<(), String> {
        let provider = self.provider(provider_name).await?;
        let response = self
            .client
            .patch(format!(
                "{}/api/v3/providers/oauth2/{}/",
                self.api_origin, provider.pk
            ))
            .bearer_auth(&self.bootstrap_token)
            .json(&AccessTokenValidityPatch {
                access_token_validity: validity,
            })
            .send()
            .await
            .map_err(|error| format!("provider token-validity request failed: {error}"))?;
        require_success(response, "provider token-validity update").await
    }

    async fn generate_signing_key(&self, provider_name: &str) -> Result<GeneratedKey, String> {
        let common_name = format!(
            "{provider_name}-{}-{}",
            std::process::id(),
            SIGNING_KEY_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let response = self
            .client
            .post(format!(
                "{}/api/v3/crypto/certificatekeypairs/generate/",
                self.api_origin
            ))
            .bearer_auth(&self.bootstrap_token)
            .json(&CertificateGeneration {
                common_name: &common_name,
                validity_days: 2,
                alg: "rsa",
            })
            .send()
            .await
            .map_err(|error| format!("signing-key generation request failed: {error}"))?;
        parse_success(response, "signing-key generation").await
    }

    async fn provider(&self, provider_name: &str) -> Result<OAuthProvider, String> {
        let response = self
            .client
            .get(format!("{}/api/v3/providers/oauth2/", self.api_origin))
            .bearer_auth(&self.bootstrap_token)
            .query(&[("name", provider_name)])
            .send()
            .await
            .map_err(|error| format!("provider lookup request failed: {error}"))?;
        let providers: ProviderPage = parse_success(response, "provider lookup").await?;
        match providers.results.as_slice() {
            [provider] => Ok(provider.clone()),
            results => Err(format!(
                "provider lookup returned {} candidates instead of one",
                results.len()
            )),
        }
    }
}

#[derive(serde::Serialize)]
struct CertificateGeneration<'a> {
    common_name: &'a str,
    validity_days: u16,
    alg: &'a str,
}

#[derive(Deserialize)]
struct GeneratedKey {
    pk: String,
}

#[derive(Deserialize)]
struct ProviderPage {
    results: Vec<OAuthProvider>,
}

#[derive(Clone, Deserialize)]
struct OAuthProvider {
    pk: u64,
}

#[derive(serde::Serialize)]
struct SigningKeyPatch<'a> {
    signing_key: &'a str,
}

#[derive(serde::Serialize)]
struct AccessTokenValidityPatch<'a> {
    access_token_validity: &'a str,
}

async fn parse_success<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
    operation: &str,
) -> Result<T, String> {
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("{operation} failed with {status}: {body}"));
    }
    response
        .json()
        .await
        .map_err(|error| format!("{operation} response was invalid: {error}"))
}

async fn require_success(response: reqwest::Response, operation: &str) -> Result<(), String> {
    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(format!("{operation} failed with {status}: {body}"))
    }
}
