use bank_server::BankEstateElevationRequestOutcome;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::super::super::protocol::{BankHttpCommitDisposition, BankHttpElevationRequestOutcome};
use super::super::authenticated_owner::BankHttpAuthenticatedOwner;
use super::super::authentication::BankHttpApplicationAuthenticator;
use super::super::elevation_registry::{
    BankHttpElevationFacts, BankHttpElevationRegistry, BankHttpElevationReplay,
};
use super::super::estate_denial::estate_denial;
use super::{outcome::*, AdmittedBankHttpElevationRequest};

pub(super) async fn execute_request<A>(
    application: &A,
    registry: &mut BankHttpElevationRegistry,
    request: AdmittedBankHttpElevationRequest,
    cancellation: WorthQueryCancellationSource,
) -> BankHttpElevationRequestOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let request_id = request.request_id;
    let scope = WorthQueryRequestScope::new(request.deadline, cancellation.token());
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => return request_denied(Some(request_id), denial),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    match registry.request_replay(&owner, &request.idempotency_key, request.action) {
        BankHttpElevationReplay::Applied((elevation, facts)) => {
            return requested(
                request_id,
                BankHttpCommitDisposition::AlreadyCommitted,
                elevation,
                facts,
            );
        }
        BankHttpElevationReplay::Conflicting => {
            return request_denied(Some(request_id), malformed());
        }
        BankHttpElevationReplay::Missing => {}
    }
    let Some(reservation) = registry.reserve_request() else {
        return request_denied(Some(request_id), saturated());
    };
    let outcome = match application
        .runtime()
        .request_estate_emergency_access_with_key(
            &principal,
            request.action,
            &request.idempotency_key,
            &scope,
        ) {
        Ok(outcome) => outcome,
        Err(denial) => return request_denied(Some(request_id), estate_denial(denial)),
    };
    let (authority, disposition) = match outcome {
        BankEstateElevationRequestOutcome::ProductStale(_) => {
            return request_denied(Some(request_id), stale());
        }
        BankEstateElevationRequestOutcome::ProductUnpublished(_)
        | BankEstateElevationRequestOutcome::NoEffect(_) => {
            return request_denied(Some(request_id), unavailable());
        }
        BankEstateElevationRequestOutcome::Requested(authority) => {
            (authority, BankHttpCommitDisposition::Committed)
        }
        BankEstateElevationRequestOutcome::AlreadyRequested(authority) => {
            (authority, BankHttpCommitDisposition::AlreadyCommitted)
        }
        BankEstateElevationRequestOutcome::Stale { .. } => {
            return request_denied(Some(request_id), stale());
        }
        BankEstateElevationRequestOutcome::Cancelled => {
            return request_denied(Some(request_id), cancelled());
        }
        BankEstateElevationRequestOutcome::TimedOut => {
            return request_denied(Some(request_id), deadline_exceeded());
        }
        BankEstateElevationRequestOutcome::Denied { .. }
        | BankEstateElevationRequestOutcome::Aborted => {
            return request_denied(Some(request_id), unavailable());
        }
        BankEstateElevationRequestOutcome::Deferred(_)
        | BankEstateElevationRequestOutcome::SettlementDeferred(_)
        | BankEstateElevationRequestOutcome::Indeterminate => {
            return request_denied(Some(request_id), indeterminate());
        }
    };
    let facts = BankHttpElevationFacts {
        changed_record_count: authority.request_changed_record_count(),
        emitted_effect_count: authority.request_emitted_effect_count(),
    };
    let elevation = registry.register_requested(
        reservation,
        owner,
        request.idempotency_key,
        request.action,
        request.context,
        authority,
        facts,
    );
    requested(request_id, disposition, elevation, facts)
}

fn requested(
    request_id: String,
    disposition: BankHttpCommitDisposition,
    elevation: String,
    facts: BankHttpElevationFacts,
) -> BankHttpElevationRequestOutcome {
    BankHttpElevationRequestOutcome::Requested {
        request_id,
        disposition,
        elevation,
        changed_record_count: facts.changed_record_count,
        emitted_effect_count: facts.emitted_effect_count,
    }
}
