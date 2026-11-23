use worth_store::physical_runtime::{
    StoreRecoveryBindingFreshness, StoreRecoveryBindingFreshnessSample, StoreRecoveryOperationFate,
};
use worth_store_recovery_physics::{
    reconcile_operation_fates, OperationReconciliationDenial, PhysicalRedoGroupBinding,
    PhysicalRedoMemberInput, ReconciledOperationFates, RecoveryBindingFreshness,
    RecoveryOperationEvidenceInput, RecoveryOperationFate, RecoveryOperationIdentity,
};

pub(super) fn reconcile_sample(
    sample: &StoreRecoveryBindingFreshnessSample,
    maximum_bindings: u64,
) -> Result<ReconciledOperationFates, OperationReconciliationDenial> {
    let mut inputs = Vec::with_capacity(sample.operations().len());
    for evidence in sample.operations() {
        let mutation = evidence.mutation_identity();
        let identity = RecoveryOperationIdentity::new(
            mutation.store_identity().bytes(),
            mutation.runtime_identity().get(),
            mutation.lifecycle_generation(),
            mutation.operation_identity().get(),
            evidence.idempotency_identity(),
        )
        .ok_or(OperationReconciliationDenial::InvalidLease)?;
        inputs.push(RecoveryOperationEvidenceInput::new(
            identity,
            evidence.request_fingerprint().bytes(),
            evidence.lease_issuance_generation(),
            evidence.lease_expiry_generation(),
            map_freshness(evidence.freshness()),
            map_fate(evidence.fate()),
        ));
    }
    reconcile_operation_fates(
        sample.selected_checkpoint_generation(),
        inputs,
        maximum_bindings,
    )
}

pub(super) fn redo_inputs(
    sample: &StoreRecoveryBindingFreshnessSample,
    fates: &ReconciledOperationFates,
) -> Result<Vec<PhysicalRedoMemberInput>, OperationReconciliationDenial> {
    let mut fate_by_operation = Vec::with_capacity(fates.operations().len());
    for fate in fates.operations() {
        fate_by_operation.push((fate.identity().idempotency(), fate.fate()));
    }
    fate_by_operation.sort_unstable_by_key(|(idempotency, _)| *idempotency);
    let mut inputs = Vec::with_capacity(sample.wal_members().len());
    for member in sample.wal_members() {
        let index = fate_by_operation
            .binary_search_by_key(&member.operation_identity(), |(idempotency, _)| *idempotency)
            .map_err(|_| OperationReconciliationDenial::DuplicateIdentity)?;
        let fate = fate_by_operation[index].1;
        let group = PhysicalRedoGroupBinding::new(
            member.group_identity(),
            member.group_member_identity(),
            member.group_member_ordinal(),
            member.group_member_count(),
            member.group_membership_digest(),
        )
        .ok_or(OperationReconciliationDenial::DuplicateIdentity)?;
        inputs.push(PhysicalRedoMemberInput::new_grouped(
            member.lsn_range(),
            member.operation_identity(),
            group,
            fate,
            member.canonical_redo(),
        ));
    }
    Ok(inputs)
}

const fn map_freshness(value: StoreRecoveryBindingFreshness) -> RecoveryBindingFreshness {
    match value {
        StoreRecoveryBindingFreshness::Retained => RecoveryBindingFreshness::Retained,
        StoreRecoveryBindingFreshness::ExpiredAtSelectedCheckpoint => {
            RecoveryBindingFreshness::ExpiredAtSelectedCheckpoint
        }
    }
}

const fn map_fate(value: StoreRecoveryOperationFate) -> RecoveryOperationFate {
    match value {
        StoreRecoveryOperationFate::AcknowledgedDurable => {
            RecoveryOperationFate::AcknowledgedDurable
        }
        StoreRecoveryOperationFate::DurableUnacknowledged => {
            RecoveryOperationFate::DurableUnacknowledged
        }
        StoreRecoveryOperationFate::ProvenNoEffect => RecoveryOperationFate::ProvenNoEffect,
        StoreRecoveryOperationFate::Indeterminate => RecoveryOperationFate::Indeterminate,
    }
}
