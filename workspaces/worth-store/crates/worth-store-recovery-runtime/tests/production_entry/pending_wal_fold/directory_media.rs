//! A released V3 edge's replacement directory frame is reread from selected
//! media by Store. A structurally valid substitute frame keeps every C.8/C.9
//! proof intact, so only Store's independent media rejoin can deny it.

use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use worth_store::physical_runtime::{
    PhysicalRecoverySelectedRejoinMismatch, RecoveredPhysicalRuntimeConstructionDenial,
};
use worth_store_physical_format::{
    decode_extent_chunk, durable_artifact_checksum, encode_extent_chunk,
    CurrentPhysicalRecordPlacement, DerivedFamilyRootDirectoryV1, DerivedFamilyRootEntry,
    DurableExtentManifest, DurablePhysicalRootManifest, ExtentArenaFrameLayout,
    ExtentChunkCoordinate, PersistedRecordIdentity, PhysicalRootRoutingBlock, RecordArtifactFile,
    DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_ARENA_MANIFEST_FRAME_BYTES,
    EXTENT_CHUNK_METADATA_BYTES,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

#[test]
fn substituted_replacement_directory_frame_denies_store_rejoin() {
    let world = super::super::pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let request = || super::super::certified_release_serving::request(world.root());
    let PhysicalRecoveryOutcome::Recovered(handoff) = WorthStoreRecovery::recover(request()) else {
        panic!("genuine two-batch release must recover before media substitution");
    };
    drop(handoff);
    // The first released edge whose drop invalidated the watermark published
    // an unindexed replacement directory; a later edge superseded it, so only
    // the historical rejoin of that edge rereads this frame.
    let manifests = root_manifests(world.root());
    let (replacement_root, replacement) = manifests
        .iter()
        .find_map(|manifest| {
            manifest
                .derived_family_directory()
                .filter(|binding| binding.indexed_through_blob_publication().is_none())
                .map(|binding| (manifest, binding.directory_record()))
        })
        .expect("a released edge published a replacement directory");
    let selected = manifests.last().unwrap();
    assert_ne!(
        selected
            .derived_family_directory()
            .map(|binding| binding.directory_record()),
        Some(replacement),
        "the substituted frame must not be the selected directory",
    );
    let route = routes(world.root(), replacement_root)
        .into_iter()
        .find(|route| route.record() == replacement)
        .expect("replacement root routes its directory");
    substitute_directory(world.root(), route);

    let outcome = WorthStoreRecovery::recover(request());
    let PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) = outcome else {
        panic!("substituted replacement directory media must deny Store rejoin: {outcome:?}")
    };
    assert_eq!(
        indeterminate.handoff_failure(),
        Some(
            RecoveredPhysicalRuntimeConstructionDenial::RejoinSelectedMedia(
                PhysicalRecoverySelectedRejoinMismatch::RoutingFrame,
            )
        ),
    );
}

fn root_manifests(root: &Path) -> Vec<DurablePhysicalRootManifest> {
    let roots = root.join("families/records/roots");
    let mut found = fs::read_dir(&roots)
        .unwrap()
        .filter_map(|entry| {
            let bytes = fs::read(entry.unwrap().path()).ok()?;
            Some(
                DurablePhysicalRootManifest::decode(&bytes, u16::MAX)
                    .ok()?
                    .0,
            )
        })
        .collect::<Vec<_>>();
    found.sort_unstable_by_key(DurablePhysicalRootManifest::generation);
    found
}

fn routes(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
) -> Vec<CurrentPhysicalRecordPlacement> {
    let roots = root.join("families/records/roots");
    let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
    let mut routes = Vec::new();
    while let Some(reference) = pending.pop() {
        let bytes = fs::read(
            roots.join(
                RecordArtifactFile::RootRoutingBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .unwrap();
        let (block, _) =
            PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity()).unwrap();
        assert_eq!(
            block.reference(durable_artifact_checksum(&bytes)),
            reference
        );
        match block {
            PhysicalRootRoutingBlock::Leaf { entries, .. } => routes.extend(entries),
            PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
        }
    }
    routes
}

/// Rewrites the directory's single extent chunk with a same-length, validly
/// framed directory naming a different family root record.
fn substitute_directory(root: &Path, route: CurrentPhysicalRecordPlacement) {
    let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        panic!("replacement directory must have an extent route")
    };
    let path = root.join("families/records/arenas").join(
        RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        }
        .file_name(),
    );
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    file.seek(SeekFrom::Start(extent.arena_range().offset()))
        .unwrap();
    let mut manifest_bytes = [0; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    file.read_exact(&mut manifest_bytes).unwrap();
    let (manifest, format) = DurableExtentManifest::decode(&manifest_bytes).unwrap();
    assert_eq!(manifest.record(), extent.record());
    assert_eq!(
        manifest.chunk_count(),
        1,
        "a directory is one bounded chunk"
    );
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment()).unwrap();
    let coordinate = ExtentChunkCoordinate::new(
        extent.record(),
        extent.extent_cell(),
        manifest.logical_bytes(),
        0,
        1,
    )
    .unwrap();
    let chunk_offset = extent.arena_range().offset() + layout.chunk_offset(1).unwrap();
    let mut original = vec![
        0;
        DURABLE_EXTENT_FRAME_HEADER_BYTES
            + EXTENT_CHUNK_METADATA_BYTES
            + manifest.logical_bytes() as usize
    ];
    file.seek(SeekFrom::Start(chunk_offset)).unwrap();
    file.read_exact(&mut original).unwrap();
    let (payload, _) = decode_extent_chunk(&original, coordinate).unwrap();
    let directory = DerivedFamilyRootDirectoryV1::decode(payload).unwrap();
    assert!(directory.indexed_through_blob_publication().is_none());
    let mut entries = directory.entries().to_vec();
    let first = entries.first_mut().expect("replacement keeps family roots");
    let mut epoch = first.root_record().allocation_epoch();
    epoch[0] ^= 0x80;
    *first = DerivedFamilyRootEntry::new(
        first.family(),
        PersistedRecordIdentity::new(epoch, first.root_record().ordinal()).unwrap(),
    )
    .unwrap();
    let changed = DerivedFamilyRootDirectoryV1::new(entries)
        .unwrap()
        .with_indexed_through_quarantine(directory.indexed_through_quarantine())
        .encode();
    assert_eq!(changed.len(), payload.len());
    assert_ne!(changed.as_slice(), payload);
    let replacement = encode_extent_chunk(format, coordinate, &changed).unwrap();
    assert_eq!(replacement.len(), original.len());
    assert_eq!(
        decode_extent_chunk(&replacement, coordinate).unwrap().0,
        changed.as_slice()
    );
    file.seek(SeekFrom::Start(chunk_offset)).unwrap();
    file.write_all(&replacement).unwrap();
    file.sync_all().unwrap();
}
