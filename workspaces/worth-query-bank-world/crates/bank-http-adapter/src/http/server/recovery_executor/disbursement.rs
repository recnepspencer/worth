//! Constructs the actual declared SourceOnly estate owner; transport stores it before replying.
use super::super::super::protocol::{BankHttpEstateDisbursementOutcome, BankHttpMutationOutcome};
use super::super::{
    authenticated_owner::BankHttpAuthenticatedOwner,
    authentication::BankHttpApplicationAuthenticator, estate_source_custody::*,
    native_route::NativeRouteResult, recovery_registry::BankHttpRecoveryRegistry,
};
use super::{outcome::*, AdmittedBankHttpDisbursementRequest};
use bank_domain::{estate::EstateAction, schema::DisburseEstate};
use std::sync::Arc;
use tokio::sync::Mutex;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

pub(super) async fn execute_disbursement<A: BankHttpApplicationAuthenticator>(
    application: &A,
    registry: Arc<Mutex<BankHttpRecoveryRegistry>>,
    request: AdmittedBankHttpDisbursementRequest,
    cancellation: WorthQueryCancellationSource,
) -> NativeRouteResult<BankHttpEstateDisbursementOutcome> {
    let scope = WorthQueryRequestScope::new(Some(request.deadline), cancellation.token());
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => return disbursement_denied(Some(request.request_id), denial).into(),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    let EstateAction::DisburseEstate(disbursement) = request.action else {
        return disbursement_denied(
            Some(request.request_id),
            super::super::mutation_refusal::denial(
                super::super::super::protocol::BankHttpDenialKind::MalformedRequest,
            ),
        )
        .into();
    };
    let operation = match application
        .runtime()
        .mutate(bank_server::mutations::disburse_estate(
            DisburseEstate::new(disbursement),
        ))
        .as_principal(&principal)
        .controls(bank_server::BankMutationControls::new(
            scope.clone(),
            request.idempotency_key.clone(),
        ))
        .retain_operation()
    {
        Ok(operation) => operation,
        Err(_cause) => {
            return disbursement_denied(
                Some(request.request_id),
                super::super::mutation_refusal::denial(
                    super::super::super::protocol::BankHttpDenialKind::InternalDenied,
                ),
            )
            .into()
        }
    };
    let custody = EstateSourceCustody::Disbursement(DisbursementCustody::Active(operation));
    let (custody, cause) = custody.advance(&application.runtime().request(&principal, &scope));
    let outcome = match cause {
        Some(denial) => BankHttpMutationOutcome::Refused {
            request_id: request.request_id.clone(),
            custody_id: request.request_id.clone(),
            sequence: 1,
            denial,
            accepted_sequence: Some(1),
        },
        None => custody.describe(&request.request_id, &request.request_id, 1),
    };
    NativeRouteResult::Estate {
        record: EstateSourceRecord {
            owner,
            sequence: 1,
            custody,
            key: request.idempotency_key,
            action: request.action,
            registry,
        },
        outcome: BankHttpEstateDisbursementOutcome::Owning { outcome },
    }
}
