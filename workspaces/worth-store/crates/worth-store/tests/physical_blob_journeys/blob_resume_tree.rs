use std::{
    fs,
    num::NonZeroU64,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobIngestSession, BlobReadLimits, BlobResumeFailure,
    BlobResumeLimits, BlobResumeToken, PhysicalMutationDeadline, PhysicalRecordId,
    ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, BlobTreeNodeKind, BlobTreeNodeV1, PersistedRecordIdentity,
};

use super::{
    blob_crash::{
        kill_at, marker_path, recover_closed_store, resume_token_path, write_marker, SCOPE_KEY,
    },
    blob_frontier::selected_blob_records,
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, placement, serving_from_open},
};

const CHUNK: usize = 64 * 1024;

/// The gate is installed only after both chunks selected. `finish` writes its
/// partial leaf first; the child is killed after that leaf's root replacement,
/// before it can start the generation-publication mutation.
pub(super) fn park_after_leaf_root(
    serving: &ServingPhysicalRuntime,
    ingest: BlobIngestSession<'_>,
    root: &Path,
    object: [u8; 16],
    session: [u8; 16],
) -> ! {
    let gate = serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterRootReplacement);
    let marker = marker_path(root);
    thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(180);
            while Instant::now() < deadline {
                if gate.await_arrival() {
                    write_marker(&marker, object, session);
                    return;
                }
            }
            panic!("partial leaf never reached selected-root replacement");
        });
        let _ = ingest.finish();
        panic!("generation publication escaped selected-leaf pause before kill");
    })
}

#[test]
fn killed_partial_finish_leaf_is_reused_as_published_root() {
    let world = kill_at("crash-leaf-selected", Duration::from_secs(210));
    let token = token_from_sidecar(&world.root);
    recover_closed_store(&world.root);
    let serving = serving_from_open(&world.root);
    let before = selected_nodes(&serving, world.session);
    assert_eq!(before.len(), 1, "C8 must select exactly the partial leaf");
    assert_eq!(before[0].1.occurrence().kind(), BlobTreeNodeKind::Leaf);
    assert_eq!(before[0].1.occurrence().index(), 0);
    assert_eq!(before[0].1.entries().len(), 2);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let blobs = serving.blobs().unwrap();
    let resumed = blobs
        .resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            resume_limits(),
        )
        .unwrap_or_else(|error| panic!("selected partial leaf must readmit: {error:?}"));
    assert_eq!(resumed.frontier().next_chunk_ordinal(), 2);
    assert_eq!(resumed.frontier().bytes(), (2 * CHUNK) as u64);
    let published = resumed.finish().unwrap();
    let after = selected_nodes(&serving, world.session);
    assert_eq!(
        after, before,
        "finish must reuse, not append, the selected leaf"
    );
    let selected_publications = selected_blob_records(&serving)
        .into_iter()
        .filter_map(|(_, bytes)| match decode_blob_record(&bytes) {
            Ok(BlobRecordV1::GenerationPublished(value)) if value.session() == world.session => {
                Some(value)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(selected_publications.len(), 1);
    let root_record = selected_publications[0].root_record();
    assert_record_binding(before[0].0, root_record);
    assert_eq!(published.object().bytes(), world.object);
    drop(blobs);
    serving.close();

    let reopened = serving_from_open(&world.root);
    let blobs = reopened.blobs().unwrap();
    let read_limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let resolved = blobs
        .resolve_publication(world.object, 1, &scope, read_limits)
        .unwrap();
    assert_eq!(resolved, published);
    let mut read = blobs
        .read(resolved, &scope, 0, (2 * CHUNK) as u64, read_limits)
        .unwrap();
    let mut actual = Vec::new();
    let mut frame = [0_u8; 4096];
    loop {
        let count = read.read_next(&mut frame).unwrap();
        if count == 0 {
            break;
        }
        actual.extend_from_slice(&frame[..count]);
    }
    assert_eq!(actual.len(), 2 * CHUNK);
    for (index, byte) in actual.iter().enumerate() {
        let expected = ((index / CHUNK) as u8)
            .wrapping_mul(17)
            .wrapping_add((index % CHUNK) as u8);
        assert_eq!(*byte, expected, "fresh read byte {index}");
    }
    drop(read);
    drop(blobs);
    reopened.close();

    let report = observe_closed_store_named(&world.root, "c11-blob-resume", "selected-leaf");
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    for (family, count) in [
        ("blob_resume_session", 1),
        ("blob_chunk_frame", 2),
        ("blob_tree_node", 1),
        ("blob_generation_publication", 1),
    ] {
        let matching = artifacts
            .iter()
            .filter(|row| row["family"] == family)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), count, "{family}: {report}");
        assert!(
            matching
                .iter()
                .all(|row| row["outcome"]["posture"] == "intact"),
            "{report}"
        );
    }
}

#[test]
fn non_prefix_selected_node_denies_before_any_resumed_write() {
    let world = kill_at("crash-leaf-selected", Duration::from_secs(210));
    let token = token_from_sidecar(&world.root);
    recover_closed_store(&world.root);
    let serving = serving_from_open(&world.root);
    let leaf = selected_nodes(&serving, world.session);
    assert_eq!(
        leaf.len(),
        1,
        "C8 must select the producer's real partial leaf"
    );
    assert_eq!(leaf[0].1.occurrence().index(), 0);
    let encoded = leaf[0].1.encode();
    let record = leaf[0].0;
    serving.close();
    reseal_selected_leaf_index(&world.root, record, &encoded);

    let serving = serving_from_open(&world.root);
    let changed = selected_nodes(&serving, world.session);
    assert_eq!(
        changed.len(),
        1,
        "resealed leaf stays selected and decodable"
    );
    assert_eq!(changed[0].0, record);
    assert_eq!(changed[0].1.occurrence().index(), 1);
    assert_eq!(changed[0].1.entries().len(), 2);
    let root_before = selected_root(&serving);
    let media_before = serving.media_counters();
    let scope = admitted_blob_scope(SCOPE_KEY);
    let blobs = serving.blobs().unwrap();
    assert!(matches!(
        blobs.resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            resume_limits(),
        ),
        Err(BlobResumeFailure::TreeConflict)
    ));
    assert_eq!(selected_root(&serving), root_before);
    let media_after = serving.media_counters();
    assert_eq!(
        media_after.append_attempts(),
        media_before.append_attempts()
    );
    assert_eq!(
        media_after.positioned_write_attempts(),
        media_before.positioned_write_attempts(),
        "dry tree validation must not cross any C5 media-write boundary"
    );
    drop(blobs);
    serving.close();
}

fn reseal_selected_leaf_index(root: &Path, record: PhysicalRecordId, old_frame: &[u8]) {
    const C5_HEADER: usize = 48;
    const EXTENT_METADATA: usize = 64;
    const C11_HEADER: usize = 48;
    let mut locations = Vec::new();
    for entry in fs::read_dir(root.join("families/records/arenas")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "data") {
            continue;
        }
        let bytes = fs::read(&path).unwrap();
        if bytes.len() < C5_HEADER + EXTENT_METADATA + old_frame.len() {
            continue;
        }
        for inner_offset in C5_HEADER + EXTENT_METADATA..=bytes.len() - old_frame.len() {
            if &bytes[inner_offset..inner_offset + 8] != b"WRC11BLB" || bytes[inner_offset + 8] != 3
            {
                continue;
            }
            let outer_offset = inner_offset - C5_HEADER - EXTENT_METADATA;
            let outer = &bytes[outer_offset..];
            if &outer[..8] != b"WRC5FRM\0" || outer[8] != 4 {
                continue;
            }
            let payload_len = u32::from_le_bytes(outer[24..28].try_into().unwrap()) as usize;
            let frame_len = C5_HEADER + payload_len;
            if outer_offset + frame_len > bytes.len()
                || payload_len != EXTENT_METADATA + old_frame.len()
                || &outer[C5_HEADER..C5_HEADER + 16] != record.allocation_epoch().as_slice()
                || u64::from_le_bytes(outer[C5_HEADER + 16..C5_HEADER + 24].try_into().unwrap())
                    != record.ordinal()
                || u32::from_le_bytes(outer[C5_HEADER + 56..C5_HEADER + 60].try_into().unwrap())
                    as usize
                    != old_frame.len()
            {
                continue;
            }
            assert_eq!(
                &bytes[inner_offset..inner_offset + old_frame.len()],
                old_frame
            );
            locations.push((path.clone(), outer_offset, frame_len));
        }
    }
    assert_eq!(
        locations.len(),
        1,
        "exact selected leaf has one C5 arena frame"
    );
    let (path, outer_offset, frame_len) = locations.pop().unwrap();
    let mut bytes = fs::read(&path).unwrap();
    let outer = &mut bytes[outer_offset..outer_offset + frame_len];
    {
        let inner = &mut outer[C5_HEADER + EXTENT_METADATA..];
        assert_eq!(&inner[..8], b"WRC11BLB");
        assert_eq!(inner[8], 3, "only the real blob tree node may be changed");
        assert_eq!(u64::from_le_bytes(inner[80..88].try_into().unwrap()), 0);
        inner[80..88].copy_from_slice(&1_u64.to_le_bytes());
        let mut hash = Sha256::new();
        hash.update(&inner[..16]);
        hash.update(&inner[C11_HEADER..]);
        inner[16..C11_HEADER].copy_from_slice(&hash.finalize());
        let BlobRecordV1::TreeNode(decoded) = decode_blob_record(inner).unwrap() else {
            panic!("resealed inner frame must stay a valid tree node");
        };
        assert_eq!(decoded.occurrence().index(), 1);
    }
    let checksum = crc32c(&outer[..44], &outer[C5_HEADER..]);
    outer[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    fs::write(path, bytes).unwrap();
}

fn crc32c(prefix: &[u8], payload: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in prefix.iter().chain(payload) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}

fn selected_nodes(
    serving: &ServingPhysicalRuntime,
    session: [u8; 16],
) -> Vec<(PhysicalRecordId, BlobTreeNodeV1)> {
    selected_blob_records(serving)
        .into_iter()
        .filter_map(|(record, bytes)| match decode_blob_record(&bytes) {
            Ok(BlobRecordV1::TreeNode(node)) if node.occurrence().session() == session => {
                Some((record, node))
            }
            _ => None,
        })
        .collect()
}

fn token_from_sidecar(root: &Path) -> BlobResumeToken {
    BlobResumeToken::decode(&fs::read(resume_token_path(root)).unwrap()).unwrap()
}

fn resume_limits() -> BlobResumeLimits {
    BlobResumeLimits::new(
        NonZeroU64::new(128).unwrap(),
        NonZeroU64::new(1024 * 1024).unwrap(),
    )
}

fn selected_root(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

fn assert_record_binding(actual: PhysicalRecordId, expected: PersistedRecordIdentity) {
    assert_eq!(actual.allocation_epoch(), expected.allocation_epoch());
    assert_eq!(actual.ordinal(), expected.ordinal());
}
