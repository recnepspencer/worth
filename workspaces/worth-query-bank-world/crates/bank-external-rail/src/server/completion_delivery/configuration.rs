//! Installed rail sender identity, peer and finite resource contract.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RailCompletionDeliveryConfiguration {
    pub(crate) callback_url: String,
    pub(crate) audience: String,
    pub(crate) source: String,
    pub(crate) key_epoch: u64,
    pub(crate) signing_seed: [u8; 32],
    pub(crate) bank_ack_verifying_key: [u8; 32],
    pub(crate) bank_tls_root_pem: Option<String>,
    pub(crate) validity_seconds: u64,
    pub(crate) maximum_pending: usize,
    pub(crate) maximum_attempts: u32,
    pub(crate) retry_delay_millis: u64,
    pub(crate) contact_timeout_millis: u64,
}

impl RailCompletionDeliveryConfiguration {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        callback_url: String,
        audience: String,
        source: String,
        key_epoch: u64,
        signing_seed: [u8; 32],
        bank_ack_verifying_key: [u8; 32],
        validity_seconds: u64,
        maximum_pending: usize,
        maximum_attempts: u32,
        retry_delay: Duration,
        contact_timeout: Duration,
        bank_tls_root_pem: Option<String>,
    ) -> Result<Self, RailCompletionDeliveryConfigurationDenial> {
        let retry_delay_millis = u64::try_from(retry_delay.as_millis())
            .map_err(|_| RailCompletionDeliveryConfigurationDenial::InvalidLimit)?;
        let contact_timeout_millis = u64::try_from(contact_timeout.as_millis())
            .map_err(|_| RailCompletionDeliveryConfigurationDenial::InvalidLimit)?;
        let candidate = Self {
            callback_url,
            audience,
            source,
            key_epoch,
            signing_seed,
            bank_ack_verifying_key,
            bank_tls_root_pem,
            validity_seconds,
            maximum_pending,
            maximum_attempts,
            retry_delay_millis,
            contact_timeout_millis,
        };
        candidate.validate()?;
        Ok(candidate)
    }

    pub(crate) fn validate(&self) -> Result<(), RailCompletionDeliveryConfigurationDenial> {
        let url = reqwest::Url::parse(&self.callback_url)
            .map_err(|_| RailCompletionDeliveryConfigurationDenial::InvalidDestination)?;
        let loopback_http = url.scheme() == "http"
            && url
                .host_str()
                .is_some_and(|host| host == "localhost" || host == "127.0.0.1" || host == "[::1]");
        if url.scheme() != "https" && !loopback_http {
            return Err(RailCompletionDeliveryConfigurationDenial::InvalidDestination);
        }
        match (url.scheme(), self.bank_tls_root_pem.as_deref()) {
            ("https", Some(pem)) if !pem.is_empty() && pem.len() <= 16 * 1024 => {
                let mut reader = pem.as_bytes();
                let certificate = rustls_pemfile::read_one(&mut reader)
                    .map_err(|_| RailCompletionDeliveryConfigurationDenial::InvalidPeerTrust)?;
                let trailing = rustls_pemfile::read_one(&mut reader)
                    .map_err(|_| RailCompletionDeliveryConfigurationDenial::InvalidPeerTrust)?;
                if !matches!(certificate, Some(rustls_pemfile::Item::X509Certificate(_)))
                    || trailing.is_some()
                {
                    return Err(RailCompletionDeliveryConfigurationDenial::InvalidPeerTrust);
                }
                let certificate = reqwest::Certificate::from_pem(pem.as_bytes())
                    .map_err(|_| RailCompletionDeliveryConfigurationDenial::InvalidPeerTrust)?;
                reqwest::Client::builder()
                    .tls_built_in_root_certs(false)
                    .add_root_certificate(certificate)
                    .build()
                    .map_err(|_| RailCompletionDeliveryConfigurationDenial::InvalidPeerTrust)?;
            }
            ("https", None) => {
                return Err(RailCompletionDeliveryConfigurationDenial::MissingPeerTrust)
            }
            ("https", Some(_)) | ("http", Some(_)) => {
                return Err(RailCompletionDeliveryConfigurationDenial::InvalidPeerTrust);
            }
            ("http", None) => {}
            _ => return Err(RailCompletionDeliveryConfigurationDenial::InvalidDestination),
        }
        if self.audience.is_empty()
            || self.source.is_empty()
            || self.audience.len() > 128
            || self.source.len() > 128
            || self.key_epoch == 0
        {
            return Err(RailCompletionDeliveryConfigurationDenial::InvalidIdentity);
        }
        if self.validity_seconds == 0
            || self.validity_seconds > 86_400
            || self.maximum_pending == 0
            || self.maximum_attempts == 0
            || self.retry_delay_millis == 0
            || self.contact_timeout_millis == 0
        {
            return Err(RailCompletionDeliveryConfigurationDenial::InvalidLimit);
        }
        ed25519_dalek::VerifyingKey::from_bytes(&self.bank_ack_verifying_key)
            .map_err(|_| RailCompletionDeliveryConfigurationDenial::InvalidPeerKey)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RailCompletionDeliveryConfigurationDenial {
    AlreadyInstalled,
    InvalidDestination,
    InvalidIdentity,
    InvalidLimit,
    InvalidPeerKey,
    MissingPeerTrust,
    InvalidPeerTrust,
}
