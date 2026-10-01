use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use worth_store::physical_runtime::{
    PhysicalExtentCopyPhase, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationRequest, PhysicalRecordId,
};
use worth_store_physical_format::{
    decode_blob_record, decode_extent_chunk, BlobRecordV1, BootstrapCatalog,
    CurrentPhysicalRecordPlacement, DurableExtentManifest, DurableExtentRecordPlacement,
    DurablePhysicalRootManifest, ExtentArenaFrameLayout, ExtentChunkCoordinate,
    PersistedRecordIdentity, PhysicalRootRoutingBlock, DURABLE_EXTENT_FRAME_HEADER_BYTES,
    EXTENT_ARENA_MANIFEST_FRAME_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};

use super::{
    blob_abort::unfinished,
    blob_frontier::selected_blob_records,
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, placement, serving_from_initialization},
};

#[test]
fn held_historical_blob_copy_is_not_a_second_selected_chunk() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let serving = serving_from_initialization(root);
    let scope = admitted_blob_scope("c11.blob.copy.observer.provenance");
    let ingest = unfinished(&serving, &scope);
    drop(ingest);
    let selected = selected_blob_records(&serving);
    let (chunk, bytes) = selected
        .iter()
        .find(|(_, bytes)| matches!(decode_blob_record(bytes), Ok(BlobRecordV1::Chunk(_))))
        .expect("one real selected blob chunk");
    let chunk = *chunk;
    let bytes = bytes.clone();
    let source = current_extent_route(root, chunk);
    let retained_source_frame = extent_frame(root, source);

    // Only selection is certification-assisted. Source pin, arena transfer,
    // WAL, root publication, and unresolved custody use production owners.
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([113; 32]))
        .unwrap();
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    );
    let identity = PersistedRecordIdentity::new(chunk.allocation_epoch(), chunk.ordinal()).unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement(), request, identity)
        .unwrap()
    else {
        panic!("actual selected blob extent must admit bounded copy");
    };
    for _ in 0..128 {
        if progress.phase == PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(progress.phase, PhysicalExtentCopyPhase::ReadyForAdoption);
    let prepared = submission.prepare_completed_extent_copy().unwrap();
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
    let destination = current_extent_route(root, chunk);
    assert_ne!(source, destination, "the selected route must actually move");
    assert_eq!(
        extent_frame(root, source),
        retained_source_frame,
        "copy publication may not overwrite its protected old source frame"
    );
    assert!(
        extent_frame(root, destination)
            .iter()
            .any(|byte| *byte != 0),
        "the selected destination has a physical frame"
    );
    assert_eq!(
        selected_blob_records(&serving)
            .iter()
            .find(|(record, _)| *record == chunk)
            .unwrap()
            .1,
        bytes
    );
    let source_range = source.arena_range();
    let source_file = root.join(format!(
        "families/records/arenas/arena-{:016x}.data",
        source_range.arena().get()
    ));
    assert!(
        std::fs::metadata(source_file).unwrap().len() > source_range.offset(),
        "unresolved copy retains the original physical extent"
    );
    drop(submission);
    serving.close();

    let report = observe_closed_store_named(root, "c11-blob-copy", "held-source-provenance");
    assert_eq!(report["completeness"], "complete", "{report}");
    let chunk_rows: Vec<_> = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["family"] == "blob_chunk_frame")
        .collect();
    assert_eq!(
        chunk_rows.len(),
        1,
        "proof-only old extent is not a selected claim: {report}"
    );
    let row = chunk_rows[0];
    assert_eq!(
        row["identity"],
        format!("blob-record:{}", hex_record(chunk))
    );
    assert_eq!(row["outcome"]["posture"], "intact", "{row}");
    assert_eq!(
        row["path"],
        format!(
            "families/records/arenas/arena-{:016x}.data",
            destination.arena_range().arena().get()
        ),
        "the blob claim must follow the selected destination"
    );
}

pub(super) fn current_extent_route(
    root: &Path,
    record: PhysicalRecordId,
) -> DurableExtentRecordPlacement {
    let catalog = BootstrapCatalog::decode(
        &std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
    )
    .unwrap();
    let generation = catalog.current_root().generation().get();
    let manifest = std::fs::read(root.join(format!(
        "families/records/roots/root-{generation:016x}.manifest"
    )))
    .unwrap();
    let (manifest, _) = DurablePhysicalRootManifest::decode(&manifest, 64).unwrap();
    let identity =
        PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal()).unwrap();
    let mut pending = vec![manifest.routing_root().unwrap()];
    let mut found = None;
    while let Some(reference) = pending.pop() {
        if !reference.contains(identity) {
            continue;
        }
        let block = std::fs::read(root.join(format!(
            "families/records/roots/root-{:016x}-block-{:016x}.manifest",
            reference.generation(),
            reference.block()
        )))
        .unwrap();
        let (block, _) =
            PhysicalRootRoutingBlock::decode(&block, manifest.node_capacity()).unwrap();
        if let Some(entries) = block.entries() {
            for entry in entries {
                if entry.record() == identity {
                    let CurrentPhysicalRecordPlacement::Extent(extent) = entry else {
                        panic!("blob chunk must have an extent arena route");
                    };
                    assert!(found.replace(*extent).is_none());
                }
            }
        } else {
            pending.extend(block.children().unwrap().iter().copied());
        }
    }
    found.expect("exact selected blob extent route")
}

fn hex_record(record: PhysicalRecordId) -> String {
    record
        .allocation_epoch()
        .into_iter()
        .chain(record.ordinal().to_le_bytes())
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn extent_frame(root: &Path, placement: DurableExtentRecordPlacement) -> Vec<u8> {
    let range = placement.arena_range();
    let path = root.join(format!(
        "families/records/arenas/arena-{:016x}.data",
        range.arena().get()
    ));
    let mut file = std::fs::File::open(path).unwrap();
    file.seek(SeekFrom::Start(range.offset())).unwrap();
    let mut manifest_frame = vec![0; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    file.read_exact(&mut manifest_frame).unwrap();
    let (manifest, format) = DurableExtentManifest::decode(&manifest_frame).unwrap();
    assert_eq!(manifest.record(), placement.record());
    assert_eq!(manifest.extent_cell(), placement.extent_cell());
    assert_eq!(manifest.logical_bytes(), placement.payload_bytes());
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment()).unwrap();
    assert!(layout.admits(range, manifest.chunk_count()));
    let mut encoded = manifest_frame;
    let capacity = u64::from(manifest.chunk_payload_capacity());
    for ordinal in 1..=manifest.chunk_count() {
        let logical_offset = u64::from(ordinal - 1) * capacity;
        let logical_length = (manifest.logical_bytes() - logical_offset).min(capacity);
        let frame_length = DURABLE_EXTENT_FRAME_HEADER_BYTES as u64
            + EXTENT_CHUNK_METADATA_BYTES as u64
            + logical_length;
        let frame_offset = range.offset() + layout.chunk_offset(ordinal).unwrap();
        assert!(frame_offset + frame_length <= range.end());
        file.seek(SeekFrom::Start(frame_offset)).unwrap();
        let mut frame = vec![0; usize::try_from(frame_length).unwrap()];
        file.read_exact(&mut frame).unwrap();
        let coordinate = ExtentChunkCoordinate::new(
            placement.record(),
            placement.extent_cell(),
            manifest.logical_bytes(),
            logical_offset,
            ordinal,
        )
        .unwrap();
        let (payload, found_format) = decode_extent_chunk(&frame, coordinate).unwrap();
        assert_eq!(found_format, format);
        assert_eq!(payload.len() as u64, logical_length);
        encoded.extend_from_slice(&frame);
    }
    encoded
}
