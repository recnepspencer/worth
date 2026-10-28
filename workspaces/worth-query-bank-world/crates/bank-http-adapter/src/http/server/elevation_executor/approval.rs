use bank_server::BankEstateElevationApprovalOutcome;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::super::super::protocol::{BankHttpCommitDisposition, BankHttpElevationApprovalOutcome};
use super::super::authenticated_owner::BankHttpAuthenticatedOwner;
use super::super::authentication::BankHttpApplicationAuthenticator;
use super::super::elevation_registry::{
    BankHttpElevationFacts, BankHttpElevationRegistry, BankHttpElevationReplay,
};
use super::super::estate_denial::estate_denial;
use super::{outcome::*, AdmittedBankHttpElevationTransition};

pub(super) async fn execute_approval<A>(
    application: &A,
    registry: &mut BankHttpElevationRegistry,
    request: AdmittedBankHttpElevationTransition,
    cancellation: WorthQueryCancellationSource,
) -> super::super::native_route::NativeRouteResult<BankHttpElevationApprovalOutcome>
where
    A: BankHttpApplicationAuthenticator,
{
    let request_id = request.request_id;
    let scope = WorthQueryRequestScope::new(Some(request.deadline), cancellation.token());
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => return (approval_denied(Some(request_id), denial)).into(),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    match registry.approval_replay(&owner, &request.token, &request.idempotency_key) {
        BankHttpElevationReplay::Applied(facts) => {
            return (approved(
                request_id,
                BankHttpCommitDisposition::AlreadyCommitted,
                request.token,
                facts,
            ))
            .into();
        }
        BankHttpElevationReplay::Conflicting => {
            return (approval_denied(Some(request_id), malformed())).into();
        }
        BankHttpElevationReplay::Missing => {}
    }
    let Some(requested) = registry.take_requested(&request.token) else {
        return (approval_denied(Some(request_id), stale())).into();
    };
    let action = requested.context.approval_action();
    let outcome = match application
        .runtime()
        .approve_estate_emergency_access_with_key(
            &principal,
            requested.authority,
            action,
            &request.idempotency_key,
            &scope,
        ) {
        Ok(outcome) => outcome,
        Err(failure) => {
            let (denial, authority) = failure.into_parts();
            if let Some(authority) = authority {
                registry.restore_requested(&request.token, authority);
            }
            return (approval_denied(Some(request_id), estate_denial(denial))).into();
        }
    };
    let (authority, disposition) = match outcome {
        BankEstateElevationApprovalOutcome::CleanupRequired(custody) => {
            return super::super::native_route::NativeRouteResult::Cleanup {
                owner,
                custody,
                outcome: BankHttpElevationApprovalOutcome::CleanupRequired {
                    request_id: request_id.clone(),
                    custody_id: request_id,
                    sequence: 1,
                },
            };
        }
        BankEstateElevationApprovalOutcome::ProductStale(_, requested) => {
            registry.restore_requested(&request.token, requested);
            return (approval_denied(Some(request_id), stale())).into();
        }
        BankEstateElevationApprovalOutcome::NoEffect(_, requested) => {
            registry.restore_requested(&request.token, requested);
            return (approval_denied(Some(request_id), unavailable())).into();
        }
        BankEstateElevationApprovalOutcome::ProductUnpublished(_) => {
            return (approval_denied(Some(request_id), indeterminate())).into();
        }
        BankEstateElevationApprovalOutcome::Approved(authority) => {
            (authority, BankHttpCommitDisposition::Committed)
        }
        BankEstateElevationApprovalOutcome::AlreadyApproved(authority) => {
            (authority, BankHttpCommitDisposition::AlreadyCommitted)
        }
        BankEstateElevationApprovalOutcome::Stale { requested, .. } => {
            registry.restore_requested(&request.token, requested);
            return (approval_denied(Some(request_id), stale())).into();
        }
        BankEstateElevationApprovalOutcome::Cancelled(requested) => {
            registry.restore_requested(&request.token, requested);
            return (approval_denied(Some(request_id), cancelled())).into();
        }
        BankEstateElevationApprovalOutcome::TimedOut => {
            return (approval_denied(Some(request_id), deadline_exceeded())).into();
        }
        BankEstateElevationApprovalOutcome::Denied { requested, .. }
        | BankEstateElevationApprovalOutcome::Aborted(requested) => {
            registry.restore_requested(&request.token, requested);
            return (approval_denied(Some(request_id), unavailable())).into();
        }
        BankEstateElevationApprovalOutcome::Deferred(_)
        | BankEstateElevationApprovalOutcome::Indeterminate => {
            return (approval_denied(Some(request_id), indeterminate())).into();
        }
    };
    let facts = BankHttpElevationFacts {
        changed_record_count: authority.approval_changed_record_count(),
        emitted_effect_count: authority.approval_emitted_effect_count(),
    };
    registry.install_approved(
        &request.token,
        owner,
        request.idempotency_key,
        authority,
        facts,
    );
    approved(request_id, disposition, request.token, facts).into()
}

fn approved(
    request_id: String,
    disposition: BankHttpCommitDisposition,
    elevation: String,
    facts: BankHttpElevationFacts,
) -> BankHttpElevationApprovalOutcome {
    BankHttpElevationApprovalOutcome::Approved {
        request_id,
        disposition,
        elevation,
        changed_record_count: facts.changed_record_count,
        emitted_effect_count: facts.emitted_effect_count,
    }
}
