use std::future::Future;
use std::pin::Pin;
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;

use bank_server::{BankAuthenticatedPrincipal, BankPrincipalAdmissionError};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationDenialKind,
    WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionDenialKind;

use crate::{AuthentikBankAuthenticationError, AuthentikBankIdentity};

use super::super::protocol::{
    BankHttpCredential, BankHttpDenial, BankHttpDenialKind, BankHttpNextAction,
};

pub(super) trait BankHttpApplicationAuthenticator:
    super::inbound_completion::BankRailCompletionRuntime + Send + Sync + 'static
{
    fn authenticate<'a>(
        &'a self,
        credential: BankHttpCredential,
        scope: &'a WorthQueryRequestScope,
    ) -> Pin<Box<dyn Future<Output = Result<BankAuthenticatedPrincipal, BankHttpDenial>> + Send + 'a>>;
}

impl BankHttpApplicationAuthenticator for AuthentikBankIdentity {
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

pub(super) fn authentication_denial(error: AuthentikBankAuthenticationError) -> BankHttpDenial {
    match error {
        AuthentikBankAuthenticationError::AuthorizationFlow(_) => malformed_credential(),
        AuthentikBankAuthenticationError::PrincipalAdmission(error) => match error {
            BankPrincipalAdmissionError::Authentication(denial) => {
                authentication_kind(denial.kind())
            }
            BankPrincipalAdmissionError::ExecutionRequest(cause) => {
                super::advancement_denial::advancement(cause)
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
        Resolution::Stopped(Err(worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind::Cancelled)) => cancelled(),
        Resolution::Stopped(Err(worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind::TimedOut)) => deadline(),
        Resolution::Refused(PrincipalRefusal::UnknownPrincipal) | Resolution::Refused(PrincipalRefusal::MissingPrincipalTarget) => malformed_credential(),
        Resolution::Refused(PrincipalRefusal::DisabledPrincipal) => BankHttpDenial::new(
            BankHttpDenialKind::PermissionDenied,
            BankHttpNextAction::None,
        ),
        Resolution::Refused(PrincipalRefusal::PrimaryGraphNotInstalled)
        | Resolution::Refused(PrincipalRefusal::BindingNotInstalled)
        | Resolution::Refused(PrincipalRefusal::BranchMaterializationSuspended)
        | Resolution::Refused(PrincipalRefusal::IdentityIndexUnavailable) => unavailable(),
        Resolution::Refused(PrincipalRefusal::ActiveSnapshotCapacityExhausted{ .. })
        | Resolution::Refused(PrincipalRefusal::SnapshotIdentityExhausted)
        | Resolution::Refused(PrincipalRefusal::RetentionCapacityExhausted)
        | Resolution::Refused(PrincipalRefusal::RetentionIdentityExhausted) => unavailable(),
        Resolution::Refused(PrincipalRefusal::ForeignRuntime)
        | Resolution::Refused(PrincipalRefusal::StaleInstalledSchema)
        | Resolution::Refused(PrincipalRefusal::ExpiredAuthentication)
        | Resolution::Refused(PrincipalRefusal::StalePrincipalProof) => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Authenticate)
        }
        Resolution::Refused(PrincipalRefusal::CorruptIdentityIndex)
        | Resolution::Refused(PrincipalRefusal::AmbiguousPrincipal)
        | Resolution::Refused(PrincipalRefusal::AmbiguousPrincipalTarget)
        | Resolution::Refused(PrincipalRefusal::WrongPrincipalTargetKind) => {
            BankHttpDenial::new(BankHttpDenialKind::InternalDenied, BankHttpNextAction::None)
        }
        Resolution::Stopped(Ok(_)) | Resolution::Refused(PrincipalRefusal::ForeignPhase) => unavailable(),
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
