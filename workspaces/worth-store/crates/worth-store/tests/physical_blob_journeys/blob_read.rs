use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, BlobReadOpenFailure,
    PhysicalMutationDeadline, PhysicalOperationAllocationScope, PhysicalWorkCounterStage,
    PhysicalWorkOperationFamily, PhysicalWorkPressureClass,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::fixture::{
    admitted_blob_scope, admitted_blob_scope_for_replay_boundary, placement,
    serving_from_initialization, serving_from_open,
};

#[test]
fn protected_blob_range_crosses_chunk_boundary_and_rejects_foreign_scope() {
    const CHUNK_BYTES: usize = 64 * 1024;
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(parent.path());
    let blobs = serving.blobs().unwrap();
    let placement = placement();
    let scope = admitted_blob_scope("c11.blob.read.scope");
    let other_scope = admitted_blob_scope_for_replay_boundary("c11.blob.read.other_scope");
    let payload = (0..3 * CHUNK_BYTES + 13)
        .map(|index| ((index * 31 + index / CHUNK_BYTES) % 251) as u8)
        .collect::<Vec<_>>();
    let selected_limit = NonZeroU64::new(128).unwrap();
    let object = blobs
        .issue_object_id(BlobReadLimits::new(selected_limit))
        .unwrap_or_else(|_| panic!("Store-issued object identity must admit"));
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        payload.len() as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(
            declaration,
            placement,
            CHUNK_BYTES as u64,
            BlobReadLimits::new(selected_limit),
        )
        .unwrap_or_else(|error| panic!("durable blob declaration must admit: {error:?}"));
    for source_frame in payload.chunks(CHUNK_BYTES / 2) {
        ingest
            .push(source_frame)
            .unwrap_or_else(|_| panic!("bounded source frame must append"));
    }
    let published = ingest
        .finish()
        .unwrap_or_else(|_| panic!("all declared bytes must publish"));
    drop(blobs);
    serving.close();
    let serving = serving_from_open(parent.path());
    let blobs = serving.blobs().unwrap();
    let before_selection = serving.physical_work_counters();
    assert_eq!(
        blobs
            .resolve_publication(
                object.bytes(),
                published.generation().sequence(),
                &scope,
                BlobReadLimits::new(NonZeroU64::new(1).unwrap()),
            )
            .unwrap(),
        published,
        "indexed resolution does not walk a selected-record scan"
    );
    let resolved = blobs
        .resolve_publication(
            object.bytes(),
            published.generation().sequence(),
            &scope,
            BlobReadLimits::new(selected_limit),
        )
        .unwrap();
    assert_eq!(resolved, published);
    let after_selection = serving.physical_work_counters();
    // Catalog point selection uses protected C.5 reads, not the rebuild scan
    // lane, even when reopened from the same durable directory root.
    assert_eq!(
        after_selection.count_under_pressure(
            PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkPressureClass::BackgroundRebuild,
            PhysicalWorkCounterStage::Terminal,
        ),
        before_selection.count_under_pressure(
            PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkPressureClass::BackgroundRebuild,
            PhysicalWorkCounterStage::Terminal,
        ),
        "reopened catalog point must not execute rebuild range work"
    );
    assert!(matches!(
        blobs.resolve_publication(
            [7; 16],
            published.generation().sequence(),
            &scope,
            BlobReadLimits::new(selected_limit),
        ),
        Err(BlobReadOpenFailure::PublicationNotFound)
    ));

    let offset = CHUNK_BYTES as u64 - 17;
    let length = CHUNK_BYTES as u64 + 41;
    let allocation_before = serving.residency_observation();
    assert_eq!(
        allocation_before
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Blob),
        0
    );
    let mut read = blobs
        .read(
            resolved,
            &scope,
            offset,
            length,
            BlobReadLimits::new(selected_limit),
        )
        .unwrap();
    let allocation_during = serving.residency_observation();
    assert_eq!(
        allocation_during
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        0,
        "completed catalog selection releases the maintenance operation"
    );
    let rebuild_before_stream = serving.physical_work_counters().count_under_pressure(
        PhysicalWorkOperationFamily::ArtifactRangeRead,
        PhysicalWorkPressureClass::BackgroundRebuild,
        PhysicalWorkCounterStage::Terminal,
    );
    assert_eq!(
        allocation_during.store_identity(),
        allocation_before.store_identity()
    );
    assert_eq!(
        allocation_during.store_generation(),
        allocation_before.store_generation()
    );
    assert_eq!(
        allocation_during.allocations().pool_incarnation(),
        allocation_before.allocations().pool_incarnation()
    );
    let read_charge = allocation_during
        .counters()
        .active_operation_bytes_for(PhysicalOperationAllocationScope::Blob);
    assert!(
        read_charge > 0,
        "protected read must retain its Store allocation"
    );
    assert_eq!(read.published_bytes(), payload.len() as u64);
    let mut observed = Vec::new();
    let mut transfer = [0_u8; 1021];
    while read.remaining_bytes() != 0 {
        let count = read.read_next(&mut transfer).unwrap();
        assert!(count > 0 && count <= transfer.len());
        observed.extend_from_slice(&transfer[..count]);
    }
    assert_eq!(read.read_next(&mut transfer).unwrap(), 0);
    assert_eq!(
        serving.physical_work_counters().count_under_pressure(
            PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkPressureClass::BackgroundRebuild,
            PhysicalWorkCounterStage::Terminal,
        ),
        rebuild_before_stream,
        "selected blob traversal must not retain the catalog's rebuild lane"
    );
    assert_eq!(read.observation().touched_chunks(), 3);
    assert_eq!(read.observation().tree_nodes_loaded(), 1);
    assert_eq!(read.observation().returned_bytes(), length);
    assert_eq!(
        observed,
        payload[offset as usize..(offset + length) as usize]
    );
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Blob),
        read_charge,
        "completed streaming retains authority until the session is dropped"
    );
    drop(read);
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Blob),
        0
    );

    assert!(matches!(
        blobs.read(
            resolved,
            &scope,
            payload.len() as u64 - 1,
            2,
            BlobReadLimits::new(selected_limit),
        ),
        Err(BlobReadOpenFailure::RangeOutOfBounds)
    ));

    assert!(matches!(
        blobs.read(
            published,
            &other_scope,
            offset,
            length,
            BlobReadLimits::new(selected_limit),
        ),
        Err(BlobReadOpenFailure::ScopeMismatch)
    ));
    drop(blobs);
    serving.close();
}
