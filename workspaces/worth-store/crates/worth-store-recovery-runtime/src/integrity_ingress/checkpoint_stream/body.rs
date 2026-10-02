use worth_store_physical_format::{
    checkpoint_certificate_frame_bytes, decode_checkpoint_certificate,
    CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES,
    CHECKPOINT_CERTIFICATE_PREFIX_BYTES, CHECKPOINT_DIRTY_FRAME_RECORD_BYTES,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
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
    CheckpointStreamAdmissionFailure, RecordEvidenceAllocation,
};

pub(super) struct CheckpointBodyAdmission<'media> {
    envelope: CheckpointEnvelopeAdmission<'media>,
    dirty: Vec<IntegrityAdmittedCheckpointDirtyBasis<'media>>,
    compaction: IntegrityAdmittedCheckpointBindingCompaction<'media>,
    bindings: Vec<IntegrityAdmittedCheckpointBinding<'media>>,
    certificates: Vec<(
        worth_store_physical_integrity::PhysicalByteRange,
        &'media [u8],
    )>,
}

impl<'media> CheckpointBodyAdmission<'media> {
    pub(super) fn admit(
        envelope: CheckpointEnvelopeAdmission<'media>,
        trace: &mut RecoveryIntegrityIngressTrace,
        allocation: &mut RecordEvidenceAllocation<'_, '_>,
    ) -> Result<Self, CheckpointStreamAdmissionFailure> {
        let identity = envelope.header.checkpoint_identity();
        let mut offset = CHECKPOINT_STREAM_HEADER_RECORD_BYTES;
        let mut dirty = allocation.reserve_records(envelope.footer.dirty_record_count())?;
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

        let mut bindings = allocation.reserve_records(envelope.footer.binding_record_count())?;
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
        let mut certificates =
            allocation.reserve_records(envelope.footer.certificate_record_count())?;
        for _ in 0..envelope.footer.certificate_record_count() {
            let prefix_range = physical_range(offset, CHECKPOINT_CERTIFICATE_PREFIX_BYTES)?;
            let prefix = envelope
                .bytes
                .get(offset..prefix_range.end_exclusive() as usize)
                .ok_or_else(|| layout_rejection(&envelope, trace))?;
            let frame_bytes = checkpoint_certificate_frame_bytes(prefix)
                .map_err(|_| layout_rejection(&envelope, trace))?;
            let range = physical_range(offset, frame_bytes)?;
            if range.end_exclusive() > envelope.footer_range.offset() {
                return Err(layout_rejection(&envelope, trace));
            }
            let bytes = envelope
                .bytes
                .get(offset..range.end_exclusive() as usize)
                .ok_or_else(|| layout_rejection(&envelope, trace))?;
            decode_checkpoint_certificate(bytes).map_err(|_| layout_rejection(&envelope, trace))?;
            certificates.push((range, bytes));
            offset = range.end_exclusive() as usize;
        }
        if offset as u64 != envelope.footer_range.offset() {
            return Err(layout_rejection(&envelope, trace));
        }
        let schema = envelope.bytes[8];
        if dirty.iter().any(|record| {
            record
                .source()
                .input()
                .map_or(true, |input| input.bytes()[8] != schema)
        }) || compaction
            .source()
            .input()
            .map_or(true, |input| input.bytes()[8] != schema)
            || bindings.iter().any(|binding| {
                binding
                    .source()
                    .input()
                    .map_or(true, |input| input.bytes()[8] != schema)
            })
            || certificates.iter().any(|(_, bytes)| bytes[8] != schema)
        {
            return Err(layout_rejection(&envelope, trace));
        }
        Ok(Self {
            envelope,
            dirty,
            compaction,
            bindings,
            certificates,
        })
    }

    pub(super) fn finish(
        self,
        trace: &mut RecoveryIntegrityIngressTrace,
        allocation: &mut RecordEvidenceAllocation<'_, '_>,
    ) -> Result<OwnerCheckpointProjection, CheckpointStreamAdmissionFailure> {
        let maximum_binding_records = self.envelope.maximum_binding_records;
        let retained_record_bytes = allocation.live_bytes();
        let validation = {
            let mut dirty = allocation.reserve_records(self.dirty.len() as u64)?;
            dirty.extend(
                self.dirty
                    .iter()
                    .map(IntegrityAdmittedCheckpointDirtyBasis::validated),
            );
            let mut bindings = allocation.reserve_records(self.bindings.len() as u64)?;
            bindings.extend(
                self.bindings
                    .iter()
                    .map(IntegrityAdmittedCheckpointBinding::validated),
            );
            validate_checkpoint_footer(
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
                )
                .with_certificates(&self.certificates),
            )
            .0
        };
        allocation.finish_reference_scope(retained_record_bytes);
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
            self.certificates,
            footer,
        )
        .map_err(|rejection| {
            record_recovery_rejection(self.envelope.footer_scope, rejection, trace)
        })?;
        admitted
            .into_owner_checkpoint(maximum_binding_records, trace, allocation)
            .map_err(|failure| match failure {
                CheckpointStreamAdmissionFailure::Integrity(rejection) => {
                    record_recovery_rejection(self.envelope.footer_scope, rejection, trace)
                }
                other => other,
            })
    }
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
