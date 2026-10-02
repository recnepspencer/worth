//! One arrival-order rule for checkpoint and WAL operation evidence.

use super::{
    StoreRecoveryBindingSampleDenial, StoreRecoveryOperationEvidence, StoreRecoveryOperationFate,
};

pub(super) fn merge_existing_evidence(
    existing: &mut StoreRecoveryOperationEvidence,
    incoming: StoreRecoveryOperationEvidence,
) -> Result<(), StoreRecoveryBindingSampleDenial> {
    if existing.mutation != incoming.mutation
        || existing.request_fingerprint != incoming.request_fingerprint
        || existing.lease_issuance_generation != incoming.lease_issuance_generation
        || existing.lease_expiry_generation != incoming.lease_expiry_generation
        || matches!((existing.attempt_binding_identity, incoming.attempt_binding_identity),
            (Some(left), Some(right)) if left != right)
        || conflicting_terminal_fates(existing.fate, incoming.fate)
    {
        return Err(StoreRecoveryBindingSampleDenial::ConflictingOperationEvidence);
    }
    if existing.fate == StoreRecoveryOperationFate::Indeterminate
        && incoming.fate != StoreRecoveryOperationFate::Indeterminate
    {
        *existing = incoming;
    }
    Ok(())
}

pub(super) fn conflicting_terminal_fates(
    existing: StoreRecoveryOperationFate,
    incoming: StoreRecoveryOperationFate,
) -> bool {
    existing != StoreRecoveryOperationFate::Indeterminate
        && incoming != StoreRecoveryOperationFate::Indeterminate
        && existing != incoming
}
