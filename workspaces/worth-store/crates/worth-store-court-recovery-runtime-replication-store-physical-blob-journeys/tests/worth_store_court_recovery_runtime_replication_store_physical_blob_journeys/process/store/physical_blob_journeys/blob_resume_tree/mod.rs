#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_resume_tree::*;
use {
    std::fs, std::num::NonZeroU64, std::path::Path, std::thread, std::time::Duration,
    std::time::Instant,
};

use {sha2::Digest, sha2::Sha256};
use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::BlobIngestSession,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobResumeFailure,
    worth_store::physical_runtime::BlobResumeLimits,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::BlobTreeNodeKind, worth_store_physical_format::BlobTreeNodeV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_crash::marker_path,
    crate::physical_blob_journeys::blob_crash::resume_token_path,
    crate::physical_blob_journeys::blob_crash::write_marker,
    crate::physical_blob_journeys::blob_crash::SCOPE_KEY,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

#[test]
fn killed_partial_finish_leaf_is_reused_as_published_root() {
    let world = kill_at("crash-leaf-selected", std::time::Duration::from_secs(210));
    let token = token_from_sidecar(&world.root);
    recover_closed_store(&world.root);
    let serving = serving_from_open(&world.root);
    let before = selected_nodes(&serving, world.session);
    assert_eq!(before.len(), 1, "C8 must select exactly the partial leaf");
    assert_eq!(
        before[0].1.occurrence().kind(),
        worth_store_physical_format::BlobTreeNodeKind::Leaf
    );
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
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
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
            Ok(worth_store_physical_format::BlobRecordV1::GenerationPublished(value))
                if value.session() == world.session =>
            {
                Some(value)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(selected_publications.len(), 1);
    let root_record = selected_publications[0].root_record();
    assert_record_binding(before[0].0, root_record);
    assert_eq!(published.object().bytes(), world.object);
    serving.close();

    let reopened = serving_from_open(&world.root);
    let blobs = reopened.blobs().unwrap();
    let read_limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
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
    let world = kill_at("crash-leaf-selected", std::time::Duration::from_secs(210));
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
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            resume_limits(),
        ),
        Err(worth_store::physical_runtime::BlobResumeFailure::TreeConflict)
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
    serving.close();
}
