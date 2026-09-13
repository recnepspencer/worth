use super::OwnerCheckpointProjection;
use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalByteRange, PhysicalIntegrityRejection, UntrustedPhysicalArtifact,
};

use super::{
    IntegrityAdmittedRecoveryArtifact, RecoveryIntegrityIngressObservation,
    RecoveryIntegrityIngressRejection, RecoveryIntegrityIngressTrace,
};

mod body;
mod envelope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckpointStreamAdmissionFailure {
    Integrity(RecoveryIntegrityIngressRejection),
    DirtyRecordLimit { observed: u64, admitted: u64 },
    BindingRecordLimit { observed: u64, admitted: u64 },
    AllocationRejected,
}

pub(crate) fn admit_observed_checkpoint_stream(
    observed: &ObservedRecoveryArtifact,
    store: StableStoreIdentity,
    maximum_dirty_records: u64,
    maximum_binding_records: u64,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<Option<OwnerCheckpointProjection>, CheckpointStreamAdmissionFailure> {
    let Some(envelope) = envelope::CheckpointEnvelopeAdmission::admit(
        observed,
        store,
        maximum_dirty_records,
        maximum_binding_records,
        trace,
    )?
    else {
        return Ok(None);
    };
    let body = body::CheckpointBodyAdmission::admit(envelope, trace)?;
    body.finish(trace).map(Some)
}

pub(super) fn physical_range(
    offset: usize,
    length: usize,
) -> Result<PhysicalByteRange, CheckpointStreamAdmissionFailure> {
    physical_range_u64(offset as u64, length as u64)
}

pub(super) fn physical_range_u64(
    offset: u64,
    length: u64,
) -> Result<PhysicalByteRange, CheckpointStreamAdmissionFailure> {
    PhysicalByteRange::new(offset, length).map_err(|_| {
        CheckpointStreamAdmissionFailure::Integrity(
            RecoveryIntegrityIngressRejection::ScopeMismatch,
        )
    })
}

pub(super) fn bounded<'media>(
    bytes: &'media [u8],
    range: PhysicalByteRange,
    scope: PhysicalArtifactScope,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<UntrustedPhysicalArtifact<'media>, CheckpointStreamAdmissionFailure> {
    let Some(selected) = bytes.get(range.offset() as usize..range.end_exclusive() as usize) else {
        return Err(record_recovery_rejection(
            scope,
            RecoveryIntegrityIngressRejection::SourceRangeOutsideObservation,
            trace,
        ));
    };
    Ok(UntrustedPhysicalArtifact::from_bounded_bytes(selected))
}

pub(super) fn record_integrity_rejection(
    scope: PhysicalArtifactScope,
    rejection: PhysicalIntegrityRejection,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> CheckpointStreamAdmissionFailure {
    record_recovery_rejection(
        scope,
        RecoveryIntegrityIngressRejection::Integrity(rejection),
        trace,
    )
}

pub(super) fn record_recovery_rejection(
    scope: PhysicalArtifactScope,
    rejection: RecoveryIntegrityIngressRejection,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> CheckpointStreamAdmissionFailure {
    trace.record(RecoveryIntegrityIngressObservation::rejected(
        scope, rejection,
    ));
    CheckpointStreamAdmissionFailure::Integrity(rejection)
}

macro_rules! bind_record {
    ($name:ident, $method:ident, $validated:ident, $validation:ident, $variant:ident, $output:ty) => {
        pub(super) fn $name<'media>(
            observed: &'media ObservedRecoveryArtifact,
            scope: PhysicalArtifactScope,
            range: PhysicalByteRange,
            validated: worth_store_physical_integrity::$validated<'media>,
            trace: &mut RecoveryIntegrityIngressTrace,
        ) -> Result<$output, CheckpointStreamAdmissionFailure> {
            let attempt = IntegrityAdmittedRecoveryArtifact::$method(
                observed,
                scope,
                range,
                worth_store_physical_integrity::$validation::Intact(validated),
                trace.counters_mut(),
            );
            trace.retain(attempt.observation());
            match attempt
                .into_outcome()
                .map_err(CheckpointStreamAdmissionFailure::Integrity)?
            {
                IntegrityAdmittedRecoveryArtifact::$variant(admitted) => Ok(admitted),
                _ => unreachable!("checkpoint record binding preserves its concrete family"),
            }
        }
    };
}

bind_record!(
    bind_header,
    bind_checkpoint_stream_header,
    IntegrityValidatedCheckpointStreamHeader,
    CheckpointStreamHeaderIntegrityValidation,
    CheckpointStreamHeader,
    super::families::checkpoint::IntegrityAdmittedCheckpointStreamHeader<'media>
);
bind_record!(
    bind_dirty,
    bind_checkpoint_dirty_basis,
    IntegrityValidatedCheckpointDirtyBasis,
    CheckpointDirtyBasisIntegrityValidation,
    CheckpointDirtyBasis,
    super::families::checkpoint::IntegrityAdmittedCheckpointDirtyBasis<'media>
);
bind_record!(
    bind_compaction,
    bind_checkpoint_binding_compaction,
    IntegrityValidatedCheckpointBindingCompaction,
    CheckpointBindingCompactionIntegrityValidation,
    CheckpointBindingCompaction,
    super::families::checkpoint::IntegrityAdmittedCheckpointBindingCompaction<'media>
);
bind_record!(
    bind_binding,
    bind_checkpoint_binding,
    IntegrityValidatedCheckpointBinding,
    CheckpointBindingIntegrityValidation,
    CheckpointBinding,
    super::families::checkpoint::IntegrityAdmittedCheckpointBinding<'media>
);
bind_record!(
    bind_footer,
    bind_checkpoint_footer,
    IntegrityValidatedCheckpointFooter,
    CheckpointFooterIntegrityValidation,
    CheckpointFooter,
    super::families::checkpoint::IntegrityAdmittedCheckpointFooter<'media>
);
