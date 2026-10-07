use std::{num::NonZeroU64, time::Duration};

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{BlobReadLimits, BlobReadOpenFailure, PhysicalRecordId};
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, BlobSessionFrontierV1};

use super::{
    blob_crash::{kill_at, recover_closed_store, SCOPE_KEY},
    blob_frontier::selected_blob_records,
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, serving_from_open},
};

const CHUNK_BYTES: usize = 64 << 10;

#[test]
fn durable_frontier_survives_kill_with_partial_next_chunk_absent() {
    let world = kill_at("crash-frontier-partial", Duration::from_secs(120));
    recover_closed_store(&world.root);

    let serving = serving_from_open(&world.root);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(1024).unwrap());
    assert!(matches!(
        blobs.resolve_publication(world.object, 1, &scope, limits),
        Err(BlobReadOpenFailure::PublicationNotFound)
    ));
    drop(blobs);

    let selected = selected_blob_records(&serving)
        .into_iter()
        .filter(|(_, bytes)| bytes.starts_with(b"WRC11BLB"))
        .collect::<Vec<_>>();
    assert_eq!(
        selected.len(),
        3,
        "only declaration, full chunk and frontier may be selected"
    );
    let mut declaration = None;
    let mut chunk = None;
    let mut frontier = None;
    for (record, bytes) in selected {
        match decode_blob_record(&bytes).expect("selected blob frame") {
            BlobRecordV1::SessionDeclared(value) => {
                let full_digest: [u8; 32] = Sha256::digest(&bytes).into();
                assert!(declaration.replace((record, full_digest, value)).is_none());
            }
            BlobRecordV1::Chunk(value) => {
                assert_eq!(value.occurrence().ordinal(), 0);
                assert_eq!(value.bytes(), &[0x3c; CHUNK_BYTES]);
                assert!(chunk.replace((record, value.stored_digest())).is_none());
            }
            BlobRecordV1::SessionFrontier(value) => {
                assert!(frontier.replace(value).is_none());
            }
            other => panic!("killed prepublication session selected unexpected frame: {other:?}"),
        }
    }
    let (declaration_record, declaration_digest, declared) =
        declaration.expect("durable declaration survived C8");
    let (chunk_record, chunk_digest) = chunk.expect("one full chunk survived C8");
    let frontier: BlobSessionFrontierV1 = frontier.expect("durable frontier survived C8");
    assert_eq!(declared.object(), world.object);
    assert_eq!(declared.session(), world.session);
    assert_eq!(declared.declared_bytes(), (2 * CHUNK_BYTES) as u64);
    assert_eq!(frontier.store(), declared.store());
    assert_eq!(frontier.session(), world.session);
    assert_record(frontier.declaration_record(), declaration_record);
    assert_eq!(frontier.declaration_digest(), declaration_digest);
    assert_record(frontier.last_chunk_record(), chunk_record);
    assert_eq!(frontier.last_chunk_digest(), chunk_digest);
    assert_eq!(
        (frontier.next_chunk_ordinal(), frontier.durable_bytes()),
        (1, CHUNK_BYTES as u64)
    );
    serving.close();

    let report =
        observe_closed_store_named(&world.root, "c11-blob-frontier-crash", "frontier-partial");
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    let count = |family: &str| {
        artifacts
            .iter()
            .filter(|row| row["family"] == family)
            .count()
    };
    assert_eq!(count("blob_resume_session"), 2, "{report}");
    assert_eq!(count("blob_chunk_frame"), 1, "{report}");
    assert_eq!(count("blob_tree_node"), 0, "{report}");
    assert_eq!(count("blob_generation_publication"), 0, "{report}");
    for artifact in artifacts.iter().filter(|row| {
        row["family"]
            .as_str()
            .is_some_and(|family| family.starts_with("blob_"))
    }) {
        assert_eq!(artifact["outcome"]["posture"], "intact", "{artifact}");
    }
}

fn assert_record(
    selected: worth_store_physical_format::PersistedRecordIdentity,
    record: PhysicalRecordId,
) {
    assert_eq!(selected.allocation_epoch(), record.allocation_epoch());
    assert_eq!(selected.ordinal(), record.ordinal());
}
