use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::super::super::protocol::{
    BankHttpRecoveryRetryDisposition, BankHttpRecoverySafeRetryOutcome,
};
use super::super::authenticated_owner::BankHttpAuthenticatedOwner;
use super::super::authentication::BankHttpApplicationAuthenticator;
use super::super::recovery_registry::{BankHttpRecoveryRegistry, BankHttpRecoveryRetry};
use super::{outcome::*, AdmittedBankHttpRecoveryRequest};

pub(super) async fn execute_safe_retry<A>(
    application: &A,
    registry: &mut BankHttpRecoveryRegistry,
    request: AdmittedBankHttpRecoveryRequest,
    cancellation: WorthQueryCancellationSource,
) -> BankHttpRecoverySafeRetryOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let scope = WorthQueryRequestScope::new(request.deadline, cancellation.token());
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => return safe_retry_denied(Some(request.request_id), denial),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    match registry.retry(&owner, &request.token, |handle, action| {
        application
            .runtime()
            .safe_retry_commit_recovery(handle, &principal, action, &scope)
    }) {
        BankHttpRecoveryRetry::Missing => safe_retry_denied(Some(request.request_id), stale()),
        BankHttpRecoveryRetry::Applied { result, replay } => {
            BankHttpRecoverySafeRetryOutcome::Applied {
                request_id: request.request_id,
                disposition: if replay {
                    BankHttpRecoveryRetryDisposition::AlreadyRetried
                } else {
                    BankHttpRecoveryRetryDisposition::Retried
                },
                external_completion: result.external_completion,
                fresh_attempt: result.fresh_attempt,
            }
        }
        BankHttpRecoveryRetry::Denied(denial) => {
            safe_retry_denied(Some(request.request_id), estate_denial(denial))
        }
    }
}
