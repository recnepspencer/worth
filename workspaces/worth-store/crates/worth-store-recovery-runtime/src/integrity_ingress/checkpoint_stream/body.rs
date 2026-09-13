use worth_store_physical_format::{
    CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES,
    CHECKPOINT_DIRTY_FRAME_RECORD_BYTES, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use worth_store_physical_integrity::{
    project_checkpoint_binding_frame_length, validate_checkpoint_binding,
    validate_checkpoint_binding_compaction, validate_checkpoint_dirty_basis,
    validate_checkpoint_footer, CheckpointBindingCompactionIntegrityValidation,
    CheckpointBindingIntegrityValidation, CheckpointDirtyBasisIntegrityValidation,
    CheckpointFooterIntegrityValidation, CheckpointFooterValidationBasis, PhysicalArtifactScope,
};

use crate::integrity_ingress::OwnerCheckpointProjection;
use crate::integrity_ingress::{
    families::checkpoint::{
        IntegrityAdmittedCheckpointBinding, IntegrityAdmittedCheckpointBindingCompaction,
        IntegrityAdmittedCheckpointDirtyBasis, IntegrityAdmittedCheckpointStream,
    },
    RecoveryIntegrityIngressRejection, RecoveryIntegrityIngressTrace,
};

use super::envelope::CheckpointEnvelopeAdmission;
use super::{
    bind_binding, bind_compaction, bind_dirty, bind_footer, bounded, physical_range,
    physical_range_u64, record_integrity_rejection, record_recovery_rejection,
    CheckpointStreamAdmissionFailure,
};

pub(super) struct CheckpointBodyAdmission<'media> {
    envelope: CheckpointEnvelopeAdmission<'media>,
    dirty: Vec<IntegrityAdmittedCheckpointDirtyBasis<'media>>,
    compaction: IntegrityAdmittedCheckpointBindingCompaction<'media>,
    bindings: Vec<IntegrityAdmittedCheckpointBinding<'media>>,
}

impl<'media> CheckpointBodyAdmission<'media> {
    pub(super) fn admit(
        envelope: CheckpointEnvelopeAdmission<'media>,
        trace: &mut RecoveryIntegrityIngressTrace,
    ) -> Result<Self, CheckpointStreamAdmissionFailure> {
        let identity = envelope.header.checkpoint_identity();
        let mut offset = CHECKPOINT_STREAM_HEADER_RECORD_BYTES;
        let mut dirty = reserve_record_evidence(envelope.footer.dirty_record_count())?;
        for _ in 0..envelope.footer.dirty_record_count() {
            let range = physical_range(offset, CHECKPOINT_DIRTY_FRAME_RECORD_BYTES)?;
            let scope = PhysicalArtifactScope::checkpoint_dirty_basis(identity, range);
            let validation = validate_checkpoint_dirty_basis(
                bounded(envelope.bytes, range, scope, trace)?,
                scope,
            )
            .0;
            let CheckpointDirtyBasisIntegrityValidation::Intact(validated) = validation else {
                let CheckpointDirtyBasisIntegrityValidation::Rejected(rejection) = validation
                else {
                    unreachable!()
                };
                return Err(record_integrity_rejection(scope, rejection, trace));
            };
            dirty.push(bind_dirty(
                envelope.observed,
                scope,
                range,
                validated,
                trace,
            )?);
            offset = range.end_exclusive() as usize;
        }

        let compaction_range =
            physical_range(offset, CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES)?;
        let compaction_scope =
            PhysicalArtifactScope::checkpoint_binding_compaction(identity, compaction_range);
        let validation = validate_checkpoint_binding_compaction(
            bounded(envelope.bytes, compaction_range, compaction_scope, trace)?,
            compaction_scope,
        )
        .0;
        let CheckpointBindingCompactionIntegrityValidation::Intact(validated) = validation else {
            let CheckpointBindingCompactionIntegrityValidation::Rejected(rejection) = validation
            else {
                unreachable!()
            };
            return Err(record_integrity_rejection(
                compaction_scope,
                rejection,
                trace,
            ));
        };
        let compaction = bind_compaction(
            envelope.observed,
            compaction_scope,
            compaction_range,
            validated,
            trace,
        )?;
        offset = compaction_range.end_exclusive() as usize;

        let mut bindings = reserve_record_evidence(envelope.footer.binding_record_count())?;
        for _ in 0..envelope.footer.binding_record_count() {
            let prefix_range = physical_range(offset, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES)?;
            let prefix_scope = PhysicalArtifactScope::checkpoint_binding(identity, prefix_range);
            let frame = project_checkpoint_binding_frame_length(
                bounded(envelope.bytes, prefix_range, prefix_scope, trace)?,
                prefix_scope,
            )
            .map_err(|rejection| record_integrity_rejection(prefix_scope, rejection, trace))?;
            let range = physical_range_u64(offset as u64, frame.encoded_bytes())?;
            if range.end_exclusive() > envelope.footer_range.offset() {
                return Err(layout_rejection(&envelope, trace));
            }
            let scope = PhysicalArtifactScope::checkpoint_binding(identity, range);
            let validation =
                validate_checkpoint_binding(bounded(envelope.bytes, range, scope, trace)?, scope).0;
            let CheckpointBindingIntegrityValidation::Intact(validated) = validation else {
                let CheckpointBindingIntegrityValidation::Rejected(rejection) = validation else {
                    unreachable!()
                };
                return Err(record_integrity_rejection(scope, rejection, trace));
            };
            bindings.push(bind_binding(
                envelope.observed,
                scope,
                range,
                validated,
                trace,
            )?);
            offset = range.end_exclusive() as usize;
        }
        if offset as u64 != envelope.footer_range.offset() {
            return Err(layout_rejection(&envelope, trace));
        }
        Ok(Self {
            envelope,
            dirty,
            compaction,
            bindings,
        })
    }

    pub(super) fn finish(
        self,
        trace: &mut RecoveryIntegrityIngressTrace,
    ) -> Result<OwnerCheckpointProjection, CheckpointStreamAdmissionFailure> {
        let maximum_binding_records = self.envelope.maximum_binding_records;
        let mut dirty = reserve_record_evidence(self.dirty.len() as u64)?;
        dirty.extend(
            self.dirty
                .iter()
                .map(IntegrityAdmittedCheckpointDirtyBasis::validated),
        );
        let mut bindings = reserve_record_evidence(self.bindings.len() as u64)?;
        bindings.extend(
            self.bindings
                .iter()
                .map(IntegrityAdmittedCheckpointBinding::validated),
        );
        let validation = validate_checkpoint_footer(
            bounded(
                self.envelope.bytes,
                self.envelope.footer_range,
                self.envelope.footer_scope,
                trace,
            )?,
            self.envelope.footer_scope,
            CheckpointFooterValidationBasis::from_record_references(
                self.envelope.header.validated(),
                &dirty,
                self.compaction.validated(),
                &bindings,
            ),
        )
        .0;
        let CheckpointFooterIntegrityValidation::Intact(validated) = validation else {
            let CheckpointFooterIntegrityValidation::Rejected(rejection) = validation else {
                unreachable!()
            };
            return Err(record_integrity_rejection(
                self.envelope.footer_scope,
                rejection,
                trace,
            ));
        };
        let footer = bind_footer(
            self.envelope.observed,
            self.envelope.footer_scope,
            self.envelope.footer_range,
            validated,
            trace,
        )?;
        let admitted = IntegrityAdmittedCheckpointStream::assemble(
            self.envelope.header,
            self.dirty,
            self.compaction,
            self.bindings,
            footer,
        )
        .map_err(|rejection| {
            record_recovery_rejection(self.envelope.footer_scope, rejection, trace)
        })?;
        admitted
            .into_owner_checkpoint(maximum_binding_records, trace)
            .map_err(|rejection| {
                record_recovery_rejection(self.envelope.footer_scope, rejection, trace)
            })
    }
}

fn reserve_record_evidence<T>(count: u64) -> Result<Vec<T>, CheckpointStreamAdmissionFailure> {
    let count =
        usize::try_from(count).map_err(|_| CheckpointStreamAdmissionFailure::AllocationRejected)?;
    let mut records = Vec::new();
    records
        .try_reserve_exact(count)
        .map_err(|_| CheckpointStreamAdmissionFailure::AllocationRejected)?;
    Ok(records)
}

#[test]
fn record_evidence_capacity_overflow_is_a_typed_resource_refusal() {
    assert!(matches!(
        reserve_record_evidence::<[u8; 2]>(u64::MAX),
        Err(CheckpointStreamAdmissionFailure::AllocationRejected)
    ));
}

fn layout_rejection(
    envelope: &CheckpointEnvelopeAdmission<'_>,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> CheckpointStreamAdmissionFailure {
    record_recovery_rejection(
        envelope.footer_scope,
        RecoveryIntegrityIngressRejection::ScopeMismatch,
        trace,
    )
}
