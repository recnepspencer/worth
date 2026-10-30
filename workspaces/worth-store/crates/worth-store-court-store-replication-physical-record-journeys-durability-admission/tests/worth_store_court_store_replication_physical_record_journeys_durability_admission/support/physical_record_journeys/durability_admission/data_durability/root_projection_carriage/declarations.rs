pub(crate) use worth_proof::NonEmpty;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::TemporalDuration;
pub(crate) use {
    worth_store::physical_runtime::DataSettledPhysicalMutation,
    worth_store::physical_runtime::PhysicalCurrentRootAdvanceOutcome,
    worth_store::physical_runtime::PhysicalDataDispatchOutcome,
    worth_store::physical_runtime::PhysicalDataSettledGroupDenial,
    worth_store::physical_runtime::PhysicalDataSettlementOutcome,
    worth_store::physical_runtime::PhysicalDurabilityGroupBasis,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRootNamespaceDurabilityOutcome,
    worth_store::physical_runtime::PhysicalRootPublicationPreparationFailureCause,
    worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome,
    worth_store::physical_runtime::PhysicalRootReplacementOutcome,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::SettledRootProjectionMergeDenial,
};
pub(crate) use {
    worth_store_physical_format::BlobGenerationPublicationV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

pub(crate) use {
    worth_store_testkit_store::durable_publication::wal_data::settle_single,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

pub(crate) fn settled_member(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    idempotency: PhysicalMutationIdempotencyMaterial,
    payload: &'static [u8],
) -> (PhysicalDurabilityGroupBasis, DataSettledPhysicalMutation) {
    let settled = settle_single(
        submission,
        placement,
        idempotency,
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload]).unwrap(),
    );
    (settled.basis, settled.member)
}
