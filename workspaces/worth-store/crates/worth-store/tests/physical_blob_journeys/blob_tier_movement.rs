//! Feature-only proof of the Store's real arena-segregated SourceCopy path.
//! Public Hot/Cold admission stays sealed until C.8 accepts the epoch witness.

use std::{num::NonZeroU64, path::Path};

use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobMovementFailure, BlobReadLimits,
    PhysicalExtentCopyPhase, PhysicalExtentCopyResolutionProgress, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationRequest, RecordAppendDenial,
    RecordAppendError,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{
    arena_tier_at_epoch, BootstrapCatalog, DurablePhysicalRootManifest, PhysicalTierClass,
};

use super::{
    blob_copy_observer::current_extent_route,
    fixture::{admitted_blob_scope, placement, serving_from_initialization},
};

#[test]
fn selected_chunk_crosses_hot_and_cold_arenas_without_half_moved_reads() {
    const CHUNK: usize = 256 * 1024;
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let blobs = serving.blobs().unwrap();
    let scope = admitted_blob_scope("c11.blob.tier.selected");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let payload = (0..CHUNK)
        .map(|index| ((index * 19 + 7) % 251) as u8)
        .collect::<Vec<_>>();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        CHUNK as u64 + 1,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, limits)
        .unwrap();
    ingest.push(&payload).unwrap();
    ingest.push(&[0xa5]).unwrap();
    let published = ingest.finish().unwrap();

    let initial_hold = blobs
        .hold_chunk_for_relocation(published, &scope, 0, limits)
        .unwrap();
    assert!(selected_root_anchor(directory.path()).is_none());
    let pre_epoch_key = serving
        .record_submission()
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([160; 32]))
        .unwrap();
    let pre_epoch_request = PhysicalMutationRequest::platform_durable(
        pre_epoch_key,
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    );
    assert!(matches!(
        blobs.certification_begin_chunk_relocation_to_tier(
            initial_hold,
            placement(),
            pre_epoch_request,
            PhysicalTierClass::Hot,
        ),
        Err(BlobMovementFailure::Copy(RecordAppendError::Denied(
            RecordAppendDenial::BlobMovementUnsupportedTier
        )))
    ));
    let chunk_record = super::blob_frontier::selected_blob_records(&serving)
        .into_iter()
        .find(|(_, bytes)| {
            matches!(
                worth_store_physical_format::decode_blob_record(bytes),
                Ok(worth_store_physical_format::BlobRecordV1::Chunk(value))
                    if value.occurrence().ordinal() == 0
            )
        })
        .unwrap()
        .0;
    let first = current_extent_route(directory.path(), chunk_record);
    assert_eq!(first.tier_class(), PhysicalTierClass::Primary);

    let epoch = serving
        .certification_activate_tier_epoch(placement())
        .unwrap();
    assert!(selected_root_anchor(directory.path()).is_some());
    assert!(first.arena_range().arena().get() < epoch);
    assert_eq!(
        arena_tier_at_epoch(Some(epoch), first.arena_range().arena()),
        PhysicalTierClass::Primary
    );

    let mut previous = first;
    for (index, target) in [
        PhysicalTierClass::Hot,
        PhysicalTierClass::Cold,
        PhysicalTierClass::Hot,
    ]
    .into_iter()
    .enumerate()
    {
        let mut old_read = blobs
            .read(published, &scope, 0, CHUNK as u64, limits)
            .unwrap();
        let mut held_during_copy = blobs
            .read(published, &scope, 0, CHUNK as u64, limits)
            .unwrap();
        let hold = blobs
            .hold_chunk_for_relocation(published, &scope, 0, limits)
            .unwrap();
        let key = serving
            .record_submission()
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
                [161 + index as u8; 32],
            ))
            .unwrap();
        let request = PhysicalMutationRequest::platform_durable(
            key,
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        );
        let mut movement = blobs
            .certification_begin_chunk_relocation_to_tier(hold, placement(), request, target)
            .unwrap_or_else(|failure| match failure {
                worth_store::physical_runtime::BlobMovementFailure::Copy(error) => panic!(
                    "begin leg {index} {:?}->{target:?}, arena {}: {error:?}",
                    previous.tier_class(),
                    previous.arena_range().arena().get()
                ),
                other => panic!(
                    "begin leg {index} {:?}->{target:?}: {other:?}",
                    previous.tier_class()
                ),
            });
        let mut observed_partial_copy = false;
        for _ in 0..512 {
            if movement.progress().phase == PhysicalExtentCopyPhase::ReadyForAdoption {
                break;
            }
            movement.advance().unwrap_or_else(|failure| {
                panic!(
                    "tier movement at {:?}: {failure:?}",
                    movement.progress().phase
                )
            });
            let progress = movement.progress();
            if !observed_partial_copy
                && progress.phase == PhysicalExtentCopyPhase::Copying
                && progress.completed_payload_bytes > 0
                && progress.completed_payload_bytes < progress.logical_bytes
            {
                assert_eq!(
                    current_extent_route(directory.path(), chunk_record),
                    previous
                );
                assert_chunk(&mut held_during_copy, &payload);
                let mut selected_during_copy = blobs
                    .read(published, &scope, 0, CHUNK as u64, limits)
                    .unwrap();
                assert_chunk(&mut selected_during_copy, &payload);
                observed_partial_copy = true;
            }
        }
        assert!(
            observed_partial_copy,
            "copy never exposed a partial progress phase"
        );
        drop(held_during_copy);
        assert_eq!(
            movement.progress().phase,
            PhysicalExtentCopyPhase::ReadyForAdoption
        );
        assert_eq!(
            current_extent_route(directory.path(), chunk_record),
            previous
        );
        let mut before_publish = blobs
            .read(published, &scope, 0, CHUNK as u64, limits)
            .unwrap();
        assert_chunk(&mut before_publish, &payload);
        drop(before_publish);
        let receipt = movement.publish().unwrap();
        assert_eq!(
            receipt.record().allocation_epoch(),
            chunk_record.allocation_epoch()
        );
        assert_eq!(receipt.record().ordinal(), chunk_record.ordinal());
        let selected = current_extent_route(directory.path(), chunk_record);
        assert_ne!(
            selected.arena_range().arena(),
            previous.arena_range().arena()
        );
        assert_eq!(
            selected.tier_class(),
            target,
            "selected arena {} epoch {epoch} prior arena {}",
            selected.arena_range().arena().get(),
            previous.arena_range().arena().get()
        );
        assert_eq!(
            arena_tier_at_epoch(Some(epoch), selected.arena_range().arena()),
            target
        );
        assert_eq!(selected.content_class(), previous.content_class());
        assert_chunk(&mut old_read, &payload);
        let mut new_read = blobs
            .read(published, &scope, 0, CHUNK as u64, limits)
            .unwrap();
        assert_chunk(&mut new_read, &payload);
        drop(new_read);
        drop(old_read);
        drop(movement);
        let submission = serving.record_submission();
        let first = copy_checkpoint(&serving, (index * 2 + 1) as u8);
        assert!(matches!(
            submission.finalize_extent_copy(&first).unwrap(),
            PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint { .. }
        ));
        let second = copy_checkpoint(&serving, (index * 2 + 2) as u8);
        assert_eq!(
            submission.finalize_extent_copy(&second).unwrap(),
            PhysicalExtentCopyResolutionProgress::Resolved
        );
        previous = selected;
    }
}

fn assert_chunk(read: &mut worth_store::physical_runtime::BlobReadSession<'_>, expected: &[u8]) {
    let mut bytes = vec![0; expected.len()];
    assert_eq!(read.read_next(&mut bytes).unwrap(), expected.len());
    assert_eq!(bytes, expected);
}

fn selected_root_anchor(root: &Path) -> Option<[u8; 32]> {
    let catalog = BootstrapCatalog::decode(
        &std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
    )
    .unwrap();
    let generation = catalog.current_root().generation().get();
    let bytes = std::fs::read(root.join(format!(
        "families/records/roots/root-{generation:016x}.manifest"
    )))
    .unwrap();
    DurablePhysicalRootManifest::decode(&bytes, 64)
        .unwrap()
        .0
        .tier_epoch_anchor()
}

fn copy_checkpoint(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    ordinal: u8,
) -> worth_store::physical_runtime::CompletedPhysicalCheckpoint {
    use worth_store::physical_runtime::{
        PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
        PhysicalCheckpointRequest,
    };
    let mut key = [219; 32];
    key[0] = ordinal;
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::at(TemporalDuration::temporal_duration(30_000).unwrap()),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("tier copy checkpoint {ordinal} must admit");
    };
    let PhysicalCheckpointOutcome::Completed(completed) = handle.wait() else {
        panic!("tier copy checkpoint {ordinal} must complete");
    };
    completed
}
