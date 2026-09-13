use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, CheckpointStreamFooter,
    CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES,
    CHECKPOINT_DIRTY_FRAME_RECORD_BYTES, CHECKPOINT_STREAM_FOOTER_RECORD_BYTES,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use worth_store_physical_integrity::{
    validate_checkpoint_footer_envelope, validate_checkpoint_stream_header,
    CheckpointFooterEnvelopeIntegrityValidation, CheckpointStreamHeaderIntegrityValidation,
    CheckpointStreamHeaderScopeIdentity, PhysicalArtifactScope, PhysicalByteRange,
    UntrustedPhysicalArtifact,
};

use crate::integrity_ingress::{
    families::checkpoint::IntegrityAdmittedCheckpointStreamHeader,
    RecoveryIntegrityIngressObservation, RecoveryIntegrityIngressRejection,
    RecoveryIntegrityIngressTrace,
};

use super::{
    bind_header, bounded, physical_range, record_integrity_rejection, record_recovery_rejection,
    CheckpointStreamAdmissionFailure,
};

pub(super) struct CheckpointEnvelopeAdmission<'media> {
    pub(super) observed: &'media ObservedRecoveryArtifact,
    pub(super) bytes: &'media [u8],
    pub(super) header: IntegrityAdmittedCheckpointStreamHeader<'media>,
    pub(super) footer_range: PhysicalByteRange,
    pub(super) footer_scope: PhysicalArtifactScope,
    pub(super) footer: CheckpointStreamFooter,
    pub(super) maximum_binding_records: u64,
}

impl<'media> CheckpointEnvelopeAdmission<'media> {
    pub(super) fn admit(
        observed: &'media ObservedRecoveryArtifact,
        store: StableStoreIdentity,
        maximum_dirty_records: u64,
        maximum_binding_records: u64,
        trace: &mut RecoveryIntegrityIngressTrace,
    ) -> Result<Option<Self>, CheckpointStreamAdmissionFailure> {
        let Some(bytes) = observed.bytes() else {
            return Ok(None);
        };
        let header_range = physical_range(0, CHECKPOINT_STREAM_HEADER_RECORD_BYTES)?;
        let header_scope = PhysicalArtifactScope::checkpoint_stream_header(
            CheckpointStreamHeaderScopeIdentity::staged(store),
            header_range,
        );
        let header_input = if bytes.len() < CHECKPOINT_STREAM_HEADER_RECORD_BYTES {
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes)
        } else {
            bounded(bytes, header_range, header_scope, trace)?
        };
        let header_validation = validate_checkpoint_stream_header(header_input, header_scope).0;
        let CheckpointStreamHeaderIntegrityValidation::Intact(validated) = header_validation else {
            let CheckpointStreamHeaderIntegrityValidation::Rejected(rejection) = header_validation
            else {
                unreachable!()
            };
            return Err(record_integrity_rejection(header_scope, rejection, trace));
        };
        let header = bind_header(observed, header_scope, header_range, validated, trace)?;
        let identity = header.checkpoint_identity();

        let minimum = CHECKPOINT_STREAM_HEADER_RECORD_BYTES
            + CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES
            + CHECKPOINT_STREAM_FOOTER_RECORD_BYTES;
        if bytes.len() < minimum {
            let footer_range = physical_range(
                CHECKPOINT_STREAM_HEADER_RECORD_BYTES
                    + CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
                CHECKPOINT_STREAM_FOOTER_RECORD_BYTES,
            )?;
            let scope = PhysicalArtifactScope::checkpoint_footer(identity, footer_range);
            return Err(record_recovery_rejection(
                scope,
                RecoveryIntegrityIngressRejection::SourceRangeOutsideObservation,
                trace,
            ));
        }
        let footer_offset = bytes.len() - CHECKPOINT_STREAM_FOOTER_RECORD_BYTES;
        let footer_range = physical_range(footer_offset, CHECKPOINT_STREAM_FOOTER_RECORD_BYTES)?;
        let footer_scope = PhysicalArtifactScope::checkpoint_footer(identity, footer_range);
        let validation = validate_checkpoint_footer_envelope(
            bounded(bytes, footer_range, footer_scope, trace)?,
            footer_scope,
        )
        .0;
        let CheckpointFooterEnvelopeIntegrityValidation::Intact(envelope) = validation else {
            let CheckpointFooterEnvelopeIntegrityValidation::Rejected(rejection) = validation
            else {
                unreachable!()
            };
            return Err(record_integrity_rejection(footer_scope, rejection, trace));
        };
        trace.record(RecoveryIntegrityIngressObservation::admitted(footer_scope));
        let footer = envelope.routing_projection().footer();
        // A valid footer CRC admits its bytes, not its counts. Bound retained
        // evidence against this observed body before any count-sized allocation.
        if !counts_fit_observed_body(footer, footer_offset as u64) {
            return Err(record_recovery_rejection(
                footer_scope,
                RecoveryIntegrityIngressRejection::ScopeMismatch,
                trace,
            ));
        }
        enforce_record_limits(
            footer.dirty_record_count(),
            footer.binding_record_count(),
            maximum_dirty_records,
            maximum_binding_records,
        )?;
        Ok(Some(Self {
            observed,
            bytes,
            header,
            footer_range,
            footer_scope,
            footer,
            maximum_binding_records,
        }))
    }
}

fn counts_fit_observed_body(footer: CheckpointStreamFooter, footer_offset: u64) -> bool {
    let Some(compaction_offset) = footer
        .dirty_record_count()
        .checked_mul(CHECKPOINT_DIRTY_FRAME_RECORD_BYTES as u64)
        .and_then(|bytes| bytes.checked_add(CHECKPOINT_STREAM_HEADER_RECORD_BYTES as u64))
    else {
        return false;
    };
    let Some(binding_bytes) = compaction_offset
        .checked_add(CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES as u64)
        .and_then(|offset| footer_offset.checked_sub(offset))
    else {
        return false;
    };
    // Prefix + nonempty payload + CRC, independent of the footer's claimed bytes.
    let minimum_binding_bytes = (CHECKPOINT_BINDING_RECORD_PREFIX_BYTES + 1 + 4) as u64;
    compaction_offset == footer.binding_compaction_header_offset()
        && binding_bytes == footer.binding_record_bytes()
        && footer.binding_record_count() <= binding_bytes / minimum_binding_bytes
}

fn enforce_record_limits(
    dirty: u64,
    bindings: u64,
    maximum_dirty: u64,
    maximum_bindings: u64,
) -> Result<(), CheckpointStreamAdmissionFailure> {
    if dirty > maximum_dirty {
        return Err(CheckpointStreamAdmissionFailure::DirtyRecordLimit {
            observed: dirty,
            admitted: maximum_dirty,
        });
    }
    if bindings > maximum_bindings {
        return Err(CheckpointStreamAdmissionFailure::BindingRecordLimit {
            observed: bindings,
            admitted: maximum_bindings,
        });
    }
    Ok(())
}
