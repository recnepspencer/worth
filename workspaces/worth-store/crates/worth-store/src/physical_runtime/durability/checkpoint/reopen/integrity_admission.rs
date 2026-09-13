use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, CheckpointBindingCompactionHeader,
    CheckpointStreamFooter, PhysicalCheckpointIdentity, PhysicalCheckpointSource,
};
use worth_store_physical_integrity::{
    project_checkpoint_binding_frame_length, validate_checkpoint_binding,
    validate_checkpoint_binding_compaction, validate_checkpoint_footer_envelope,
    validate_checkpoint_stream_header, CheckpointBindingCompactionIntegrityValidation,
    CheckpointBindingIntegrityValidation, CheckpointFooterEnvelopeIntegrityValidation,
    CheckpointStreamHeaderIntegrityValidation, CheckpointStreamHeaderScopeIdentity,
    PhysicalArtifactScope, PhysicalByteRange, UntrustedPhysicalArtifact,
};

use super::super::PhysicalBindingCompactionReopenFailure;

pub(super) fn admit_stream_header(
    record: &[u8],
    store: StableStoreIdentity,
    range: PhysicalByteRange,
) -> Result<PhysicalCheckpointSource, PhysicalBindingCompactionReopenFailure> {
    let scope = PhysicalArtifactScope::checkpoint_stream_header(
        CheckpointStreamHeaderScopeIdentity::staged(store),
        range,
    );
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(record);
    match validate_checkpoint_stream_header(input, scope).0 {
        CheckpointStreamHeaderIntegrityValidation::Intact(validated) => {
            if !validated.matches_input(input) {
                return Err(PhysicalBindingCompactionReopenFailure::SourceIncarnationMismatch);
            }
            Ok(validated.source())
        }
        CheckpointStreamHeaderIntegrityValidation::Rejected(rejection) => {
            Err(PhysicalBindingCompactionReopenFailure::Integrity(rejection))
        }
    }
}

pub(super) fn admit_footer_envelope(
    record: &[u8],
    identity: PhysicalCheckpointIdentity,
    range: PhysicalByteRange,
) -> Result<CheckpointStreamFooter, PhysicalBindingCompactionReopenFailure> {
    let scope = PhysicalArtifactScope::checkpoint_footer(identity, range);
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(record);
    match validate_checkpoint_footer_envelope(input, scope).0 {
        CheckpointFooterEnvelopeIntegrityValidation::Intact(validated) => {
            if !validated.matches_input(input) {
                return Err(PhysicalBindingCompactionReopenFailure::SourceIncarnationMismatch);
            }
            Ok(validated.routing_projection().footer())
        }
        CheckpointFooterEnvelopeIntegrityValidation::Rejected(rejection) => {
            Err(PhysicalBindingCompactionReopenFailure::Integrity(rejection))
        }
    }
}

pub(super) fn admit_binding_compaction(
    record: &[u8],
    identity: PhysicalCheckpointIdentity,
    range: PhysicalByteRange,
) -> Result<CheckpointBindingCompactionHeader, PhysicalBindingCompactionReopenFailure> {
    let scope = PhysicalArtifactScope::checkpoint_binding_compaction(identity, range);
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(record);
    match validate_checkpoint_binding_compaction(input, scope).0 {
        CheckpointBindingCompactionIntegrityValidation::Intact(validated) => {
            if !validated.matches_input(input) {
                return Err(PhysicalBindingCompactionReopenFailure::SourceIncarnationMismatch);
            }
            CheckpointBindingCompactionHeader::new(
                validated.generation(),
                validated.wal_cutoff_lsn_exclusive(),
            )
            .ok_or(PhysicalBindingCompactionReopenFailure::SourceIncarnationMismatch)
        }
        CheckpointBindingCompactionIntegrityValidation::Rejected(rejection) => {
            Err(PhysicalBindingCompactionReopenFailure::Integrity(rejection))
        }
    }
}

pub(in crate::physical_runtime::durability) fn binding_frame_bytes(
    prefix: &[u8],
    identity: PhysicalCheckpointIdentity,
    range: PhysicalByteRange,
) -> Result<u64, PhysicalBindingCompactionReopenFailure> {
    project_checkpoint_binding_frame_length(
        UntrustedPhysicalArtifact::from_bounded_bytes(prefix),
        PhysicalArtifactScope::checkpoint_binding(identity, range),
    )
    .map(|projection| projection.encoded_bytes())
    .map_err(PhysicalBindingCompactionReopenFailure::Integrity)
}

pub(in crate::physical_runtime::durability) fn admit_binding_payload<'record>(
    record: &'record [u8],
    identity: PhysicalCheckpointIdentity,
    range: PhysicalByteRange,
) -> Result<&'record [u8], PhysicalBindingCompactionReopenFailure> {
    let scope = PhysicalArtifactScope::checkpoint_binding(identity, range);
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(record);
    match validate_checkpoint_binding(input, scope).0 {
        CheckpointBindingIntegrityValidation::Intact(validated) => {
            let projection = validated
                .project_payload(input, identity)
                .map_err(|_| PhysicalBindingCompactionReopenFailure::SourceIncarnationMismatch)?;
            input
                .bytes()
                .get(projection.payload_range())
                .ok_or(PhysicalBindingCompactionReopenFailure::SourceIncarnationMismatch)
        }
        CheckpointBindingIntegrityValidation::Rejected(rejection) => {
            Err(PhysicalBindingCompactionReopenFailure::Integrity(rejection))
        }
    }
}

pub(in crate::physical_runtime::durability) fn physical_range(
    offset: u64,
    length: u64,
) -> Result<PhysicalByteRange, PhysicalBindingCompactionReopenFailure> {
    PhysicalByteRange::new(offset, length)
        .map_err(|_| PhysicalBindingCompactionReopenFailure::CounterOverflow)
}
