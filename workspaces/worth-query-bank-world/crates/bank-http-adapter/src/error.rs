use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestInterruption;

use bank_server::{
    BankAuthenticationBoundaryBuildError, BankIdentityRuntimeBuildError,
    BankPrincipalAdmissionError,
};

use crate::configuration::AuthentikOidcConfigurationError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthentikOidcAdapterBuildError {
    HttpClient,
    MissingColdCertificationTrustedRoot,
    InvalidColdCertificationTrustedRoot,
    DiscoveryUnavailable,
    DiscoveryInterrupted(WorthQueryRequestInterruption),
    MissingTokenEndpoint,
}

impl std::fmt::Display for AuthentikOidcAdapterBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Authentik OIDC adapter build failed: {self:?}")
    }
}

impl std::error::Error for AuthentikOidcAdapterBuildError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthentikOidcFlowError {
    StateMismatch,
    InvalidAuthorizationCode,
    InvalidState,
    RequestInterrupted(WorthQueryRequestInterruption),
    TokenExchangeRejected,
    MissingIdToken,
    RevocationRejected,
}

impl std::fmt::Display for AuthentikOidcFlowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Authentik authorization flow failed: {self:?}")
    }
}

impl std::error::Error for AuthentikOidcFlowError {}

#[derive(Debug)]
pub enum AuthentikBankIdentityBuildError {
    Configuration(AuthentikOidcConfigurationError),
    Adapter(AuthentikOidcAdapterBuildError),
    Runtime(BankIdentityRuntimeBuildError),
    AuthenticationBoundary(BankAuthenticationBoundaryBuildError),
}

impl std::fmt::Display for AuthentikBankIdentityBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration(error) => error.fmt(formatter),
            Self::Adapter(error) => error.fmt(formatter),
            Self::Runtime(error) => error.fmt(formatter),
            Self::AuthenticationBoundary(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for AuthentikBankIdentityBuildError {}

#[derive(Debug)]
pub enum AuthentikBankAuthenticationError {
    AuthorizationFlow(AuthentikOidcFlowError),
    PrincipalAdmission(BankPrincipalAdmissionError),
}

impl std::fmt::Display for AuthentikBankAuthenticationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthorizationFlow(error) => error.fmt(formatter),
            Self::PrincipalAdmission(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for AuthentikBankAuthenticationError {}
