pub(crate) use {std::num::NonZeroU32, std::num::NonZeroU64};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::certification::CertificationDurableMutationInput, worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy, worth_store::physical_runtime::DataDispatchedPhysicalMutation, worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalDataDispatchFailureCause, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalDataEffectSource, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalResidencyDimension, worth_store::physical_runtime::PhysicalResidencyRetryPosture, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalSpeculativeWorkKind, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordServingTerminalPosture};
pub(crate) use {worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};

pub(crate) fn assert_inline_candidate_trace(dispatched: &DataDispatchedPhysicalMutation) -> u64 {
    let fates = dispatched
        .effects()
        .iter()
        .map(|effect| (effect.source(), effect.effect_fate()))
        .collect::<Vec<_>>();
    assert_eq!(
        fates
            .iter()
            .filter(|(source, fate)| {
                *source == worth_store::physical_runtime::PhysicalDataEffectSource::NewArtifact
                    && *fate == worth_store::physical_runtime::PhysicalWorkEffectFate::PublicationCompleted
            })
            .count(),
        1,
        "an inline batch creates exactly one segment candidate artifact"
    );
    let writebacks = fates
        .iter()
        .filter(|(source, fate)| {
            *source == worth_store::physical_runtime::PhysicalDataEffectSource::ExistingArtifactWriteback
                && *fate == worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
        })
        .count() as u64;
    assert!(
        writebacks > 0,
        "the multi-page inline segment must require writeback"
    );
    assert_eq!(fates.len() as u64, writebacks + 1);
    writebacks
}

pub(crate) fn two_append_writebehind_policy(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use {worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope};
    use {worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Kind};

    let page = u64::from(format.declaration().page_size().bytes());
    let operation = page * 512;
    let metadata = page * 2;
    let resident = page * 4;
    let mut builder = worth_store::physical_runtime::PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero(operation + metadata + resident))
        .resident_bytes(nonzero(resident))
        .metadata_bytes(nonzero(metadata))
        .frame_entries(nonzero_count(12))
        .pinned_frames(nonzero_count(4))
        .pin_leases(nonzero_count(6))
        .dirty_frames(nonzero_count(2))
        .dirty_replacement_bytes(nonzero(resident))
        .operation_bytes(nonzero(operation));
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        builder = builder.scope_bytes(scope, nonzero(operation));
    }
    for (kind, frames) in [
        (Kind::ReadAhead, 2),
        (Kind::Prefetch, 2),
        (Kind::WriteBehind, 1),
    ] {
        builder = builder.speculative_frames(kind, nonzero_count(frames));
    }
    builder.admit(format).into_result().unwrap()
}

pub(crate) fn nonzero(value: u64) -> NonZeroU64 {
    std::num::NonZeroU64::new(value).unwrap()
}

pub(crate) fn nonzero_count(value: u32) -> NonZeroU32 {
    std::num::NonZeroU32::new(value).unwrap()
}