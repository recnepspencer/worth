use bank_server::BankEstateMandatoryReviewOutcome;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::super::super::protocol::{BankHttpCommitDisposition, BankHttpMandatoryReviewOutcome};
use super::super::authenticated_owner::BankHttpAuthenticatedOwner;
use super::super::authentication::BankHttpApplicationAuthenticator;
use super::super::elevation_registry::{
    BankHttpElevationRegistry, BankHttpElevationReplay, BankHttpElevationReviewFacts,
};
use super::super::estate_denial::estate_denial;
use super::revocation::closure;
use super::{outcome::*, AdmittedBankHttpElevationTransition};

pub(super) async fn execute_mandatory_review<A>(
    application: &A,
    registry: &mut BankHttpElevationRegistry,
    request: AdmittedBankHttpElevationTransition,
    cancellation: WorthQueryCancellationSource,
) -> BankHttpMandatoryReviewOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let request_id = request.request_id;
    let scope = WorthQueryRequestScope::new(request.deadline, cancellation.token());
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => return review_denied(Some(request_id), denial),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    match registry.review_replay(&owner, &request.token, &request.idempotency_key) {
        BankHttpElevationReplay::Applied(facts) => {
            return reviewed(
                request_id,
                BankHttpCommitDisposition::AlreadyCommitted,
                facts,
            );
        }
        BankHttpElevationReplay::Conflicting => {
            return review_denied(Some(request_id), malformed());
        }
        BankHttpElevationReplay::Missing => {}
    }
    let Some(mandatory) = registry.take_mandatory_review(&request.token) else {
        return review_denied(Some(request_id), stale());
    };
    let action = mandatory.context.review_action();
    let outcome = match application
        .runtime()
        .complete_estate_mandatory_review_with_key(
            &principal,
            mandatory.authority,
            action,
            &request.idempotency_key,
            &scope,
        ) {
        Ok(outcome) => outcome,
        Err(failure) => {
            let (denial, authority) = failure.into_parts();
            if let Some(authority) = authority {
                registry.restore_mandatory_review(&request.token, authority);
            }
            return review_denied(Some(request_id), estate_denial(denial));
        }
    };
    let (reviewed_authority, disposition) = match outcome {
        BankEstateMandatoryReviewOutcome::ProductStale(_, mandatory) => {
            registry.restore_mandatory_review(&request.token, mandatory);
            return review_denied(Some(request_id), stale());
        }
        BankEstateMandatoryReviewOutcome::NoEffect(_, mandatory) => {
            registry.restore_mandatory_review(&request.token, mandatory);
            return review_denied(Some(request_id), unavailable());
        }
        BankEstateMandatoryReviewOutcome::ProductUnpublished(_) => {
            return review_denied(Some(request_id), indeterminate());
        }
        BankEstateMandatoryReviewOutcome::Reviewed(authority) => {
            (authority, BankHttpCommitDisposition::Committed)
        }
        BankEstateMandatoryReviewOutcome::AlreadyReviewed(authority) => {
            (authority, BankHttpCommitDisposition::AlreadyCommitted)
        }
        BankEstateMandatoryReviewOutcome::Stale { mandatory, .. } => {
            registry.restore_mandatory_review(&request.token, mandatory);
            return review_denied(Some(request_id), stale());
        }
        BankEstateMandatoryReviewOutcome::Cancelled(mandatory) => {
            registry.restore_mandatory_review(&request.token, mandatory);
            return review_denied(Some(request_id), cancelled());
        }
        BankEstateMandatoryReviewOutcome::TimedOut => {
            return review_denied(Some(request_id), deadline_exceeded());
        }
        BankEstateMandatoryReviewOutcome::Denied { mandatory, .. }
        | BankEstateMandatoryReviewOutcome::Aborted(mandatory) => {
            registry.restore_mandatory_review(&request.token, mandatory);
            return review_denied(Some(request_id), unavailable());
        }
        BankEstateMandatoryReviewOutcome::Deferred(_)
        | BankEstateMandatoryReviewOutcome::SettlementDeferred(_)
        | BankEstateMandatoryReviewOutcome::Indeterminate => {
            return review_denied(Some(request_id), indeterminate());
        }
    };
    let facts = BankHttpElevationReviewFacts {
        closure: reviewed_authority.closure_kind(),
        changed_record_count: reviewed_authority.review_changed_record_count(),
    };
    registry.install_reviewed(&request.token, owner, request.idempotency_key, facts);
    reviewed(request_id, disposition, facts)
}

fn reviewed(
    request_id: String,
    disposition: BankHttpCommitDisposition,
    facts: BankHttpElevationReviewFacts,
) -> BankHttpMandatoryReviewOutcome {
    BankHttpMandatoryReviewOutcome::Reviewed {
        request_id,
        disposition,
        closure: closure(facts.closure),
        changed_record_count: facts.changed_record_count,
    }
}
