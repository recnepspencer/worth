use bank_server::BankAuthenticationConfiguration;
use openidconnect::{
    ClientId, ClientSecret, IntrospectionUrl, IssuerUrl, RedirectUrl, RevocationUrl,
};
use sha2::{Digest, Sha256};
use url::Url;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod,
};

worth_query_host::facade::worth_query_authentication_audience!(
    /// The bank's OIDC client: introspection pins the token audience to the
    /// configured client id before identities carry this audience.
    pub BankClientAudience
);
worth_query_host::facade::worth_query_authentication_method!(
    /// Authentik OIDC authorization-code flow with PKCE.
    pub AuthentikOidcCodePkce
);
worth_query_host::facade::worth_query_principal_issuer!(
    /// The configured Authentik issuer: ID-token verification pins the claimed
    /// issuer to the configured issuer URL before identities carry it.
    pub AuthentikIssuer
);

#[derive(Clone)]
pub struct AuthentikOidcConfiguration {
    pub(crate) issuer: IssuerUrl,
    pub(crate) client_id: ClientId,
    pub(crate) client_secret: ClientSecret,
    pub(crate) redirect_url: RedirectUrl,
    pub(crate) introspection_url: IntrospectionUrl,
    pub(crate) revocation_url: RevocationUrl,
    configuration_identity: String,
}

impl AuthentikOidcConfiguration {
    pub fn builder() -> AuthentikOidcConfigurationBuilder {
        AuthentikOidcConfigurationBuilder::default()
    }

    fn from_builder(
        builder: AuthentikOidcConfigurationBuilder,
    ) -> Result<Self, AuthentikOidcConfigurationError> {
        let issuer = IssuerUrl::new(
            builder
                .issuer
                .ok_or(AuthentikOidcConfigurationError::InvalidIssuer)?,
        )
        .map_err(|_| AuthentikOidcConfigurationError::InvalidIssuer)?;
        let issuer_origin =
            secure_origin(issuer.as_str()).ok_or(AuthentikOidcConfigurationError::InvalidIssuer)?;
        let client_id = checked_text(
            builder
                .client_id
                .ok_or(AuthentikOidcConfigurationError::InvalidClientId)?,
        )
        .map(ClientId::new)
        .ok_or(AuthentikOidcConfigurationError::InvalidClientId)?;
        let client_secret = checked_text(
            builder
                .client_secret
                .ok_or(AuthentikOidcConfigurationError::InvalidClientSecret)?,
        )
        .map(ClientSecret::new)
        .ok_or(AuthentikOidcConfigurationError::InvalidClientSecret)?;
        let redirect_url = RedirectUrl::new(
            builder
                .redirect_url
                .ok_or(AuthentikOidcConfigurationError::InvalidRedirectUrl)?,
        )
        .map_err(|_| AuthentikOidcConfigurationError::InvalidRedirectUrl)?;
        let introspection_url = IntrospectionUrl::new(
            builder
                .introspection_url
                .ok_or(AuthentikOidcConfigurationError::InvalidIntrospectionUrl)?,
        )
        .map_err(|_| AuthentikOidcConfigurationError::InvalidIntrospectionUrl)?;
        require_origin_affinity(introspection_url.as_str(), &issuer_origin)
            .ok_or(AuthentikOidcConfigurationError::InvalidIntrospectionUrl)?;
        let revocation_url = RevocationUrl::new(
            builder
                .revocation_url
                .ok_or(AuthentikOidcConfigurationError::InvalidRevocationUrl)?,
        )
        .map_err(|_| AuthentikOidcConfigurationError::InvalidRevocationUrl)?;
        require_origin_affinity(revocation_url.as_str(), &issuer_origin)
            .ok_or(AuthentikOidcConfigurationError::InvalidRevocationUrl)?;
        let configuration_identity = configuration_identity(
            &issuer,
            &client_id,
            &redirect_url,
            &introspection_url,
            &revocation_url,
        );
        Ok(Self {
            issuer,
            client_id,
            client_secret,
            redirect_url,
            introspection_url,
            revocation_url,
            configuration_identity,
        })
    }

    pub fn bank_authentication_configuration(
        &self,
    ) -> Result<BankAuthenticationConfiguration, AuthentikOidcConfigurationError> {
        Ok(BankAuthenticationConfiguration::new(
            self.authentication_audience()?,
            self.authentication_method()?,
        ))
    }

    pub(crate) fn configuration_identity(&self) -> &str {
        &self.configuration_identity
    }

    pub(crate) fn authentication_audience(
        &self,
    ) -> Result<WorthQueryAuthenticationAudience, AuthentikOidcConfigurationError> {
        Ok(WorthQueryAuthenticationAudience::of(BankClientAudience))
    }

    pub(crate) fn authentication_method(
        &self,
    ) -> Result<WorthQueryAuthenticationMethod, AuthentikOidcConfigurationError> {
        Ok(WorthQueryAuthenticationMethod::of(AuthentikOidcCodePkce))
    }
}

#[derive(Default)]
pub struct AuthentikOidcConfigurationBuilder {
    issuer: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    redirect_url: Option<String>,
    introspection_url: Option<String>,
    revocation_url: Option<String>,
}

impl AuthentikOidcConfigurationBuilder {
    pub fn issuer(mut self, value: impl Into<String>) -> Self {
        self.issuer = Some(value.into());
        self
    }

    pub fn client_id(mut self, value: impl Into<String>) -> Self {
        self.client_id = Some(value.into());
        self
    }

    pub fn client_secret(mut self, value: impl Into<String>) -> Self {
        self.client_secret = Some(value.into());
        self
    }

    pub fn redirect_url(mut self, value: impl Into<String>) -> Self {
        self.redirect_url = Some(value.into());
        self
    }

    pub fn introspection_url(mut self, value: impl Into<String>) -> Self {
        self.introspection_url = Some(value.into());
        self
    }

    pub fn revocation_url(mut self, value: impl Into<String>) -> Self {
        self.revocation_url = Some(value.into());
        self
    }

    pub fn build(self) -> Result<AuthentikOidcConfiguration, AuthentikOidcConfigurationError> {
        AuthentikOidcConfiguration::from_builder(self)
    }
}

impl std::fmt::Debug for AuthentikOidcConfiguration {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AuthentikOidcConfiguration")
            .field("issuer", &self.issuer)
            .field("client_id", &self.client_id)
            .field("redirect_url", &self.redirect_url)
            .field("introspection_url", &self.introspection_url)
            .field("revocation_url", &self.revocation_url)
            .field("client_secret", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthentikOidcConfigurationError {
    InvalidIssuer,
    InvalidClientId,
    InvalidClientSecret,
    InvalidRedirectUrl,
    InvalidIntrospectionUrl,
    InvalidRevocationUrl,
    InvalidAuthenticationMethod,
}

impl std::fmt::Display for AuthentikOidcConfigurationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid Authentik OIDC configuration: {self:?}")
    }
}

impl std::error::Error for AuthentikOidcConfigurationError {}

fn checked_text(value: String) -> Option<String> {
    (!value.is_empty()
        && value.trim() == value
        && value.len() <= 512
        && !value.chars().any(char::is_control))
    .then_some(value)
}

fn secure_origin(value: &str) -> Option<(String, u16)> {
    let url = Url::parse(value).ok()?;
    (url.scheme() == "https").then_some(())?;
    Some((url.host_str()?.to_owned(), url.port_or_known_default()?))
}

fn require_origin_affinity(value: &str, issuer_origin: &(String, u16)) -> Option<()> {
    let endpoint_origin = secure_origin(value)?;
    (endpoint_origin == *issuer_origin).then_some(())
}

fn configuration_identity(
    issuer: &IssuerUrl,
    client_id: &ClientId,
    redirect_url: &RedirectUrl,
    introspection_url: &IntrospectionUrl,
    revocation_url: &RevocationUrl,
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"worth-bank-authentik-oidc-v1");
    for value in [
        issuer.as_str(),
        client_id.as_str(),
        redirect_url.as_str(),
        introspection_url.as_str(),
        revocation_url.as_str(),
    ] {
        digest.update(value.len().to_le_bytes());
        digest.update(value.as_bytes());
    }
    format!("authentik-oidc:{:x}", digest.finalize())
}
