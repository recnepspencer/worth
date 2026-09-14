use bank_server::{BankEstateElevationCloseOutcome, BankEstateElevationClosureKind};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::super::super::protocol::{
    BankHttpCommitDisposition, BankHttpElevationClosureKind, BankHttpElevationRevocationOutcome,
};
use super::super::authenticated_owner::BankHttpAuthenticatedOwner;
use super::super::authentication::BankHttpApplicationAuthenticator;
use super::super::elevation_registry::{
    BankHttpElevationRegistry, BankHttpElevationReplay, BankHttpElevationReviewFacts,
};
use super::super::estate_denial::estate_denial;
use super::{outcome::*, AdmittedBankHttpElevationTransition};

pub(super) async fn execute_revocation<A>(
    application: &A,
    registry: &mut BankHttpElevationRegistry,
    request: AdmittedBankHttpElevationTransition,
    cancellation: WorthQueryCancellationSource,
) -> BankHttpElevationRevocationOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let request_id = request.request_id;
    let scope = WorthQueryRequestScope::new(request.deadline, cancellation.token());
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => return revocation_denied(Some(request_id), denial),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    match registry.close_replay(&owner, &request.token, &request.idempotency_key) {
        BankHttpElevationReplay::Applied(facts) => {
            return closed(
                request_id,
                BankHttpCommitDisposition::AlreadyCommitted,
                request.token,
                facts,
            );
        }
        BankHttpElevationReplay::Conflicting => {
            return revocation_denied(Some(request_id), malformed());
        }
        BankHttpElevationReplay::Missing => {}
    }
    let Some(approved) = registry.take_approved(&request.token) else {
        return revocation_denied(Some(request_id), stale());
    };
    let action = approved.context.revocation_action();
    let outcome = match application
        .runtime()
        .revoke_estate_emergency_access_with_key(
            &principal,
            approved.authority,
            action,
            &request.idempotency_key,
            &scope,
        ) {
        Ok(outcome) => outcome,
        Err(failure) => {
            let (denial, authority) = failure.into_parts();
            if let Some(authority) = authority {
                registry.restore_approved(&request.token, authority);
            }
            return revocation_denied(Some(request_id), estate_denial(denial));
        }
    };
    let (authority, disposition) = match outcome {
        BankEstateElevationCloseOutcome::ProductStale(_, approved) => {
            registry.restore_approved(&request.token, approved);
            return revocation_denied(Some(request_id), stale());
        }
        BankEstateElevationCloseOutcome::NoEffect(_, approved) => {
            registry.restore_approved(&request.token, approved);
            return revocation_denied(Some(request_id), unavailable());
        }
        BankEstateElevationCloseOutcome::ProductUnpublished(_) => {
            return revocation_denied(Some(request_id), unavailable());
        }
        BankEstateElevationCloseOutcome::Closed(authority) => {
            (authority, BankHttpCommitDisposition::Committed)
        }
        BankEstateElevationCloseOutcome::AlreadyClosed(authority) => {
            (authority, BankHttpCommitDisposition::AlreadyCommitted)
        }
        BankEstateElevationCloseOutcome::Stale { approved, .. } => {
            registry.restore_approved(&request.token, approved);
            return revocation_denied(Some(request_id), stale());
        }
        BankEstateElevationCloseOutcome::Cancelled(approved) => {
            registry.restore_approved(&request.token, approved);
            return revocation_denied(Some(request_id), cancelled());
        }
        BankEstateElevationCloseOutcome::TimedOut => {
            return revocation_denied(Some(request_id), deadline_exceeded());
        }
        BankEstateElevationCloseOutcome::Denied { approved, .. }
        | BankEstateElevationCloseOutcome::Aborted(approved) => {
            registry.restore_approved(&request.token, approved);
            return revocation_denied(Some(request_id), unavailable());
        }
        BankEstateElevationCloseOutcome::Deferred(_)
        | BankEstateElevationCloseOutcome::SettlementDeferred(_)
        | BankEstateElevationCloseOutcome::Indeterminate => {
            return revocation_denied(Some(request_id), indeterminate());
        }
    };
    let facts = BankHttpElevationReviewFacts {
        closure: authority.closure_kind(),
        changed_record_count: authority.close_changed_record_count(),
    };
    registry.install_mandatory_review(
        &request.token,
        owner,
        request.idempotency_key,
        authority,
        facts,
    );
    closed(request_id, disposition, request.token, facts)
}

fn closed(
    request_id: String,
    disposition: BankHttpCommitDisposition,
    mandatory_review: String,
    facts: BankHttpElevationReviewFacts,
) -> BankHttpElevationRevocationOutcome {
    BankHttpElevationRevocationOutcome::Closed {
        request_id,
        disposition,
        mandatory_review,
        closure: closure(facts.closure),
        changed_record_count: facts.changed_record_count,
    }
}

pub(super) const fn closure(
    closure: BankEstateElevationClosureKind,
) -> BankHttpElevationClosureKind {
    match closure {
        BankEstateElevationClosureKind::Revoked => BankHttpElevationClosureKind::Revoked,
        BankEstateElevationClosureKind::Expired => BankHttpElevationClosureKind::Expired,
    }
}
