use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use worth_store_physical_format::{
    decode_blob_record, decode_extent_chunk, encode_extent_chunk, BlobRecordV1,
    CurrentPhysicalRecordPlacement, DurableExtentManifest, DurablePhysicalRootManifest,
    DurableRootSelector, ExtentArenaFrameLayout, ExtentChunkCoordinate, OriginalDropReservedV1,
    RecordArtifactFile, DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_ARENA_MANIFEST_FRAME_BYTES,
    EXTENT_CHUNK_METADATA_BYTES,
};

pub(super) fn alter_selected_checkpoint(path: &Path) {
    let mut bytes = std::fs::read(path).expect("selected checkpoint");
    let last = bytes.last_mut().expect("nonempty selected checkpoint");
    *last ^= 0x01;
    std::fs::write(path, bytes).expect("alter selected checkpoint");
}

pub(super) fn alter_selected_file(directory: &Path, selected: impl Fn(&str) -> bool) {
    let path = std::fs::read_dir(directory)
        .expect("selected artifact directory")
        .map(|entry| entry.expect("selected artifact entry").path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(&selected)
        })
        .expect("selected artifact to alter");
    alter_file(&path);
}

pub(super) fn alter_selected_routing_root(root: &Path) {
    let records = root.join("families/records");
    let selector = DurableRootSelector::decode(
        &std::fs::read(records.join(RecordArtifactFile::CurrentRootSelector.file_name())).unwrap(),
    )
    .expect("selected current root selector");
    let roots = records.join("roots");
    let manifest = roots.join(
        RecordArtifactFile::RootManifest {
            generation: selector.root_generation(),
        }
        .file_name(),
    );
    let (manifest, format) =
        DurablePhysicalRootManifest::decode(&std::fs::read(manifest).unwrap(), u16::MAX)
            .expect("selected current root manifest");
    assert_eq!(format, selector.format());
    let selected = manifest
        .routing_root()
        .expect("selected released root has a routing tree");
    let path = roots.join(
        RecordArtifactFile::RootRoutingBlock {
            generation: selected.generation(),
            block: selected.block(),
        }
        .file_name(),
    );
    alter_file(&path);
}

fn alter_file(path: &Path) {
    let mut handle = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("selected artifact");
    let mut byte = [0];
    handle.read_exact(&mut byte).expect("selected byte");
    byte[0] ^= 0x01;
    handle.seek(SeekFrom::Start(0)).unwrap();
    handle.write_all(&byte).unwrap();
    handle.sync_all().unwrap();
}

pub(super) fn alter_selected_control(root: &Path, route: CurrentPhysicalRecordPlacement) {
    let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        panic!("selected V3 descriptor must have an extent route")
    };
    let file = root.join("families/records/arenas").join(
        RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        }
        .file_name(),
    );
    let mut handle = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(file)
        .expect("selected descriptor arena");
    handle
        .seek(SeekFrom::Start(extent.arena_range().offset()))
        .unwrap();
    let mut byte = [0];
    handle.read_exact(&mut byte).unwrap();
    byte[0] ^= 0x01;
    handle
        .seek(SeekFrom::Start(extent.arena_range().offset()))
        .unwrap();
    handle.write_all(&byte).unwrap();
    handle.sync_all().unwrap();
}

/// Substitute one canonical reservation while keeping its arena extent frame
/// structurally valid, so the Store rejoin must reject selected-media drift.
pub(super) fn substitute_selected_reservation(root: &Path, route: CurrentPhysicalRecordPlacement) {
    let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        panic!("selected reservation must have an extent route")
    };
    let path = root.join("families/records/arenas").join(
        RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        }
        .file_name(),
    );
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("selected reservation arena");
    file.seek(SeekFrom::Start(extent.arena_range().offset()))
        .unwrap();
    let mut manifest_bytes = [0; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    file.read_exact(&mut manifest_bytes).unwrap();
    let (manifest, format) = DurableExtentManifest::decode(&manifest_bytes).unwrap();
    assert_eq!(manifest.record(), extent.record());
    assert_eq!(manifest.extent_cell(), extent.extent_cell());
    assert_eq!(
        manifest.chunk_count(),
        1,
        "reservation is one bounded chunk"
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
    let chunk_bytes = DURABLE_EXTENT_FRAME_HEADER_BYTES
        + EXTENT_CHUNK_METADATA_BYTES
        + manifest.logical_bytes() as usize;
    file.seek(SeekFrom::Start(chunk_offset)).unwrap();
    let mut original_frame = vec![0; chunk_bytes];
    file.read_exact(&mut original_frame).unwrap();
    let (payload, original_format) = decode_extent_chunk(&original_frame, coordinate).unwrap();
    assert_eq!(original_format, format);
    let BlobRecordV1::OriginalDropReserved(original) = decode_blob_record(payload).unwrap() else {
        panic!("selected control is the original drop reservation")
    };
    let mut changed_basis = original.source_basis_digest();
    changed_basis[0] ^= 0x80;
    let changed = OriginalDropReservedV1::new(
        original.store(),
        original.reclaim_attempt(),
        original.manifest_record(),
        original.manifest_frame_sha256(),
        changed_basis,
        original.manifest_selected_generation(),
        original.reserved_selected_generation(),
        original.request(),
    )
    .unwrap()
    .encode();
    assert_eq!(changed.len(), payload.len());
    let replacement = encode_extent_chunk(format, coordinate, &changed).unwrap();
    assert_eq!(replacement.len(), original_frame.len());
    assert!(matches!(
        decode_blob_record(decode_extent_chunk(&replacement, coordinate).unwrap().0),
        Ok(BlobRecordV1::OriginalDropReserved(_))
    ));
    file.seek(SeekFrom::Start(chunk_offset)).unwrap();
    file.write_all(&replacement).unwrap();
    file.sync_all().unwrap();
}
