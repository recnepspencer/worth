use sha2::{Digest, Sha256};
use worth_store_physical_backend::{ArtifactTreeFile, ArtifactTreeMedia, QualifiedFilesystemMedia};
use worth_store_physical_format::CHECKPOINT_BINDING_RECORD_PREFIX_BYTES;

use crate::physical_runtime::durability::checkpoint::{
    admit_binding_payload, binding_frame_bytes, physical_range,
    NamespaceDurablePhysicalBindingCompactionReopen, PhysicalBindingCompactionRebuildBasis,
    PhysicalBindingCompactionReopenCounters, PhysicalBindingCompactionReopenFailure,
};

use super::{PhysicalIdempotencyRegistryRebuilder, PhysicalIdempotencyReopenFailure};

pub(super) struct CheckpointAggregateAdmission {
    _sealed: (),
}

pub(super) const fn admit_generation_zero() -> CheckpointAggregateAdmission {
    CheckpointAggregateAdmission { _sealed: () }
}

pub(super) fn rebuild(
    reopened: &NamespaceDurablePhysicalBindingCompactionReopen,
    media: &QualifiedFilesystemMedia,
    builder: &mut PhysicalIdempotencyRegistryRebuilder,
) -> Result<
    (
        PhysicalBindingCompactionReopenCounters,
        CheckpointAggregateAdmission,
    ),
    PhysicalIdempotencyReopenFailure,
> {
    let basis = reopened.rebuild_basis();
    if media.store_identity() != basis.checkpoint().store_identity() {
        return Err(checkpoint_failure(
            PhysicalBindingCompactionReopenFailure::ForeignStore,
        ));
    }
    let records_read = scan_records(&media.artifact_tree(), &basis, builder)?;
    let counters = basis
        .completed_counters(records_read)
        .map_err(checkpoint_failure)?;
    Ok((counters, CheckpointAggregateAdmission { _sealed: () }))
}

fn scan_records(
    tree: &ArtifactTreeMedia<'_>,
    basis: &PhysicalBindingCompactionRebuildBasis<'_>,
    builder: &mut PhysicalIdempotencyRegistryRebuilder,
) -> Result<u64, PhysicalIdempotencyReopenFailure> {
    let mut digest = Sha256::new();
    let mut offset = basis.records_offset();
    let mut records = 0_u64;
    while offset < basis.footer_offset() {
        let prefix = read_prefix(tree, basis.artifact(), offset)?;
        let prefix_range = physical_range(offset, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES as u64)
            .map_err(checkpoint_failure)?;
        let frame_bytes = binding_frame_bytes(&prefix, basis.checkpoint(), prefix_range)
            .map_err(checkpoint_failure)?;
        let end = offset.checked_add(frame_bytes).ok_or_else(|| {
            checkpoint_failure(PhysicalBindingCompactionReopenFailure::CounterOverflow)
        })?;
        if end > basis.footer_offset() {
            return Err(checkpoint_failure(
                PhysicalBindingCompactionReopenFailure::ArtifactLayoutMismatch,
            ));
        }
        let frame_bytes = usize::try_from(frame_bytes).map_err(|_| {
            checkpoint_failure(PhysicalBindingCompactionReopenFailure::AllocationRejected)
        })?;
        let mut record = Vec::new();
        record.try_reserve_exact(frame_bytes).map_err(|_| {
            checkpoint_failure(PhysicalBindingCompactionReopenFailure::AllocationRejected)
        })?;
        record.resize(frame_bytes, 0);
        record[..CHECKPOINT_BINDING_RECORD_PREFIX_BYTES].copy_from_slice(&prefix);
        tree.read_exact_at(
            basis.artifact(),
            offset + CHECKPOINT_BINDING_RECORD_PREFIX_BYTES as u64,
            &mut record[CHECKPOINT_BINDING_RECORD_PREFIX_BYTES..],
        )
        .map_err(|failure| {
            checkpoint_failure(PhysicalBindingCompactionReopenFailure::Media(failure))
        })?;
        let range = physical_range(offset, frame_bytes as u64).map_err(checkpoint_failure)?;
        let payload = admit_binding_payload(&record, basis.checkpoint(), range)
            .map_err(checkpoint_failure)?;
        builder.consume_compaction_record(payload)?;
        digest.update(&record);
        records = records.checked_add(1).ok_or_else(|| {
            checkpoint_failure(PhysicalBindingCompactionReopenFailure::CounterOverflow)
        })?;
        offset = end;
    }
    if records != basis.expected_records()
        || offset - basis.records_offset() != basis.expected_encoded_bytes()
        || <[u8; 32]>::from(digest.finalize()) != basis.expected_digest()
    {
        return Err(checkpoint_failure(
            PhysicalBindingCompactionReopenFailure::ArtifactLayoutMismatch,
        ));
    }
    Ok(records)
}

fn read_prefix(
    tree: &ArtifactTreeMedia<'_>,
    artifact: &ArtifactTreeFile,
    offset: u64,
) -> Result<[u8; CHECKPOINT_BINDING_RECORD_PREFIX_BYTES], PhysicalIdempotencyReopenFailure> {
    let mut prefix = [0; CHECKPOINT_BINDING_RECORD_PREFIX_BYTES];
    tree.read_exact_at(artifact, offset, &mut prefix)
        .map_err(|failure| {
            checkpoint_failure(PhysicalBindingCompactionReopenFailure::Media(failure))
        })?;
    Ok(prefix)
}

fn checkpoint_failure(
    failure: PhysicalBindingCompactionReopenFailure,
) -> PhysicalIdempotencyReopenFailure {
    PhysicalIdempotencyReopenFailure::Checkpoint(failure)
}
