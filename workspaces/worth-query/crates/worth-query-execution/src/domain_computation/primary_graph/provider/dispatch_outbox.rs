//! Deriving the dispatch-outbox record that co-commits with the mutation.
//!
//! An operation that declares no external effect derives nothing and pushes
//! nothing: the absence of a record is what makes ordinary commits pay zero
//! (R8.4).

use worth_query_installation::facade::InstalledExternalEffectContract;
use worth_relational::facade::history::BranchId;

use crate::domain_computation::application_aftermath::{
    derive_external_effect_correlation_identity, ExternalEffectCorrelationBasis,
    WorthQueryAftermathDerivationFailure, WorthQueryDispatchOutboxRecord,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCommitOutcomeIdentity, WorthQueryApplicationIdempotencyBinding,
};

/// The operation-side facts that bind one dispatch correlation.
pub(in crate::domain_computation::primary_graph) struct WorthQueryDispatchOutboxBasis<'a> {
    pub external_effect: &'a InstalledExternalEffectContract,
    pub external_payload: Option<&'a [u8]>,
    pub operation_slot: &'a str,
    pub operation_version: u64,
    pub idempotency: WorthQueryApplicationIdempotencyBinding,
    pub outcome_identity: WorthQueryApplicationCommitOutcomeIdentity,
    pub branch: &'a BranchId,
}

/// Derives the durable outbox record for a declared external effect.
///
/// Returns `None` for `InstalledExternalEffectContract::None` without touching
/// canonicalization, so an undeclared operation performs no derivation work.
pub(in crate::domain_computation::primary_graph) fn derive_dispatch_outbox_record(
    basis: WorthQueryDispatchOutboxBasis<'_>,
) -> Result<Option<WorthQueryDispatchOutboxRecord>, WorthQueryAftermathDerivationFailure> {
    let InstalledExternalEffectContract::Declared {
        correlation_family, ..
    } = basis.external_effect
    else {
        return Ok(None);
    };
    let payload = basis
        .external_payload
        .ok_or(WorthQueryAftermathDerivationFailure::MissingExternalPayload)?;
    let correlation =
        derive_external_effect_correlation_identity(ExternalEffectCorrelationBasis {
            correlation_family: correlation_family.clone(),
            operation_slot: basis.operation_slot,
            operation_version: basis.operation_version,
            outcome_identity: basis.outcome_identity.get(),
            idempotency_key: basis.idempotency.key_identity(),
            branch: &basis.branch.0,
        })?;
    Ok(WorthQueryDispatchOutboxRecord::from_installed_contract(
        correlation,
        basis.external_effect,
        payload.to_vec(),
        basis.outcome_identity.get(),
    )
    .map(|record| record.with_operation_slot(basis.operation_slot)))
}
