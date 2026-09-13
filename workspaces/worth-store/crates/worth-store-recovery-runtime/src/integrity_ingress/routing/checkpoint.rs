use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_integrity::{
    CheckpointBindingCompactionIntegrityValidation, CheckpointBindingIntegrityValidation,
    CheckpointDirtyBasisIntegrityValidation, CheckpointFooterIntegrityValidation,
    CheckpointStreamHeaderIntegrityValidation, PhysicalArtifactScope, PhysicalByteRange,
};

use super::super::admitted_artifact::IntegrityAdmittedRecoveryArtifact;
use super::super::families::checkpoint::{
    IntegrityAdmittedCheckpointBinding, IntegrityAdmittedCheckpointBindingCompaction,
    IntegrityAdmittedCheckpointDirtyBasis, IntegrityAdmittedCheckpointFooter,
    IntegrityAdmittedCheckpointStreamHeader,
};
use super::super::{ObservedRecoverySource, RecoveryIntegrityIngressCounters};
use super::{recorded, rejected_integrity, RecoveryIntegrityIngressAttempt};

macro_rules! checkpoint_record_binding {
    ($name:ident, $validation:ident, $wrapper:ty, $variant:ident) => {
        pub(crate) fn $name(
            observed: &'media ObservedRecoveryArtifact,
            expected_scope: PhysicalArtifactScope,
            relative_range: PhysicalByteRange,
            validation: $validation<'media>,
            counters: &mut RecoveryIntegrityIngressCounters,
        ) -> RecoveryIntegrityIngressAttempt<'media> {
            match validation {
                $validation::Intact(validated) => recorded(
                    expected_scope,
                    <$wrapper>::bind(
                        ObservedRecoverySource::bounded_subrange(
                            observed,
                            expected_scope,
                            relative_range,
                        ),
                        validated,
                    )
                    .map(Self::$variant),
                    counters,
                ),
                $validation::Rejected(rejection) => {
                    rejected_integrity(expected_scope, rejection, counters)
                }
            }
        }
    };
}

impl<'media> IntegrityAdmittedRecoveryArtifact<'media> {
    checkpoint_record_binding!(
        bind_checkpoint_stream_header,
        CheckpointStreamHeaderIntegrityValidation,
        IntegrityAdmittedCheckpointStreamHeader<'media>,
        CheckpointStreamHeader
    );
    checkpoint_record_binding!(
        bind_checkpoint_dirty_basis,
        CheckpointDirtyBasisIntegrityValidation,
        IntegrityAdmittedCheckpointDirtyBasis<'media>,
        CheckpointDirtyBasis
    );
    checkpoint_record_binding!(
        bind_checkpoint_binding_compaction,
        CheckpointBindingCompactionIntegrityValidation,
        IntegrityAdmittedCheckpointBindingCompaction<'media>,
        CheckpointBindingCompaction
    );
    checkpoint_record_binding!(
        bind_checkpoint_binding,
        CheckpointBindingIntegrityValidation,
        IntegrityAdmittedCheckpointBinding<'media>,
        CheckpointBinding
    );
    checkpoint_record_binding!(
        bind_checkpoint_footer,
        CheckpointFooterIntegrityValidation,
        IntegrityAdmittedCheckpointFooter<'media>,
        CheckpointFooter
    );
}
