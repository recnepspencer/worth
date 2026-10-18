use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::super::super::protocol::{BankHttpRecoveryInspectionOutcome, BankHttpRecoveryWork};
use super::super::authenticated_owner::BankHttpAuthenticatedOwner;
use super::super::authentication::BankHttpApplicationAuthenticator;
use super::super::recovery_registry::BankHttpRecoveryRegistry;
use super::{outcome::*, AdmittedBankHttpRecoveryRequest};

pub(in crate::http::server) async fn execute_inspection<A>(
    application: &A,
    registry: &mut BankHttpRecoveryRegistry,
    request: AdmittedBankHttpRecoveryRequest,
    cancellation: WorthQueryCancellationSource,
) -> BankHttpRecoveryInspectionOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let scope = WorthQueryRequestScope::new(Some(request.deadline), cancellation.token());
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => return inspection_denied(Some(request.request_id), denial),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    let Some(recovery) = registry.recovery(&owner, &request.token) else {
        return inspection_denied(Some(request.request_id), stale());
    };
    match application.runtime().inspect_commit_recovery(
        recovery.handle(),
        &principal,
        recovery.action(),
        &scope,
    ) {
        Ok(inspection) => {
            let work = inspection.canonical_work();
            BankHttpRecoveryInspectionOutcome::Inspected {
                request_id: request.request_id,
                posture: recovery_posture(inspection.posture()),
                work: BankHttpRecoveryWork {
                    basis_preparations: work.basis_preparations() as usize,
                    digest_derivations: work.digest_derivations() as usize,
                    canonical_encoded_bytes: work.canonical_encoded_bytes(),
                },
            }
        }
        Err(denial) => inspection_denied(Some(request.request_id), estate_denial(denial)),
    }
}
