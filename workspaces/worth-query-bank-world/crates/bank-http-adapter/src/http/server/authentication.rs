use std::future::Future;
use std::pin::Pin;

use bank_server::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankPrincipalAdmissionError};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationDenialKind,
    WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionDenialKind;

use crate::{AuthentikBankAuthenticationError, AuthentikBankIdentity};

use super::super::protocol::{
    BankHttpCredential, BankHttpDenial, BankHttpDenialKind, BankHttpNextAction,
};

pub(super) trait BankHttpApplicationAuthenticator: Send + Sync + 'static {
    fn runtime(&self) -> &BankIdentityRuntime;

    fn authenticate<'a>(
        &'a self,
        credential: BankHttpCredential,
        scope: &'a WorthQueryRequestScope,
    ) -> Pin<Box<dyn Future<Output = Result<BankAuthenticatedPrincipal, BankHttpDenial>> + Send + 'a>>;
}

impl BankHttpApplicationAuthenticator for AuthentikBankIdentity {
    fn runtime(&self) -> &BankIdentityRuntime {
        self.runtime()
    }

    fn authenticate<'a>(
        &'a self,
        credential: BankHttpCredential,
        scope: &'a WorthQueryRequestScope,
    ) -> Pin<Box<dyn Future<Output = Result<BankAuthenticatedPrincipal, BankHttpDenial>> + Send + 'a>>
    {
        Box::pin(async move {
            let credential = credential
                .into_authentik()
                .ok_or_else(malformed_credential)?;
            self.authenticate_credential(credential, scope)
                .await
                .map_err(authentication_denial)
        })
    }
}

fn malformed_credential() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::Unauthenticated,
        BankHttpNextAction::Authenticate,
    )
}

fn authentication_denial(error: AuthentikBankAuthenticationError) -> BankHttpDenial {
    match error {
        AuthentikBankAuthenticationError::AuthorizationFlow(_) => malformed_credential(),
        AuthentikBankAuthenticationError::PrincipalAdmission(error) => match error {
            BankPrincipalAdmissionError::Authentication(denial) => {
                authentication_kind(denial.kind())
            }
            BankPrincipalAdmissionError::ProductSelection(_) => unavailable(),
            BankPrincipalAdmissionError::Resolution(denial) => resolution_kind(denial.kind()),
        },
    }
}

fn authentication_kind(kind: WorthQueryAuthenticationDenialKind) -> BankHttpDenial {
    use WorthQueryAuthenticationAdapterFailureKind as Adapter;
    use WorthQueryAuthenticationDenialKind as Authentication;
    match kind {
        Authentication::AdapterFailed(Adapter::Cancelled) | Authentication::Cancelled => {
            cancelled()
        }
        Authentication::AdapterFailed(Adapter::DeadlineExceeded)
        | Authentication::DeadlineExceeded => deadline(),
        Authentication::AdapterFailed(
            Adapter::CredentialRejected
            | Adapter::CredentialExpired
            | Adapter::CredentialRevoked
            | Adapter::BindingMismatch
            | Adapter::ProtocolViolation,
        )
        | Authentication::AudienceMismatch
        | Authentication::MethodMismatch
        | Authentication::ValidationTimeInFuture
        | Authentication::Expired => malformed_credential(),
        Authentication::AdapterFailed(Adapter::DependencyUnavailable) => unavailable(),
    }
}

fn resolution_kind(kind: WorthQueryPrincipalResolutionDenialKind) -> BankHttpDenial {
    use WorthQueryPrincipalResolutionDenialKind as Resolution;
    match kind {
        Resolution::Cancelled => cancelled(),
        Resolution::DeadlineExceeded => deadline(),
        Resolution::UnknownPrincipal | Resolution::MissingPrincipalTarget => malformed_credential(),
        Resolution::DisabledPrincipal => BankHttpDenial::new(
            BankHttpDenialKind::PermissionDenied,
            BankHttpNextAction::None,
        ),
        Resolution::PrimaryGraphNotInstalled
        | Resolution::BindingNotInstalled
        | Resolution::IdentityIndexUnavailable => unavailable(),
        Resolution::ActiveSnapshotCapacityExhausted { .. }
        | Resolution::SnapshotIdentityExhausted
        | Resolution::RetentionCapacityExhausted
        | Resolution::RetentionIdentityExhausted => unavailable(),
        Resolution::ForeignRuntime
        | Resolution::StaleInstalledSchema
        | Resolution::ExpiredAuthentication
        | Resolution::StalePrincipalProof => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Authenticate)
        }
        Resolution::CorruptIdentityIndex
        | Resolution::AmbiguousPrincipal
        | Resolution::AmbiguousPrincipalTarget
        | Resolution::WrongPrincipalTargetKind => {
            BankHttpDenial::new(BankHttpDenialKind::InternalDenied, BankHttpNextAction::None)
        }
    }
}

const fn cancelled() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry)
}

const fn deadline() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::DeadlineExceeded,
        BankHttpNextAction::Retry,
    )
}

const fn unavailable() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
}
