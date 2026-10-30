//! Real process kill after the released V3 drop descriptor WAL is durable,
//! before its root is published. C8 must roll the exact release forward.

pub(crate) use {
    std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::thread, std::time::Duration,
    std::time::Instant,
};

pub(crate) use worth_proof::AdmittedBlobReleaseProof;
pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};
pub(crate) use {
    worth_store_physical_format::BlobRecordKind,
    worth_store_physical_format::SelectedRecordContentClass,
};

pub(crate) use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_crash::marker_path,
    crate::physical_blob_journeys::blob_crash::write_marker,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::assert_released_open_requires_recovered_custody,
    worth_store_testkit_store::fixture::configuration,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
};

pub(crate) const ROLE: &str = "crash-released-first-wal";
/// The same seam after a tier epoch is certified by the selected checkpoint.
pub(crate) const TIER_ROLE: &str = "crash-released-tier-first-wal";
pub(crate) const SCOPE: &str = "c11.blob.released.redo.scope";
pub(crate) const CHUNK: usize = 64 << 10;

pub(crate) fn child(root: &Path, role: &str) {
    assert!(role == ROLE || role == TIER_ROLE);
    let serving = serving_from_initialization(root);
    establish_recovery_frontier(&serving);
    if role == TIER_ROLE {
        crate::physical_blob_journeys::clean_reopen_custody::certify_tier_epoch(&serving);
    }
    let scope = admitted_blob_scope(SCOPE);
    let blobs = serving.blobs().unwrap();
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, limits)
        .unwrap();
    ingest.push(&vec![0x83; CHUNK]).unwrap();
    ingest.push(&vec![0x94; CHUNK]).unwrap();
    let session = ingest.session_id().bytes();
    let published = ingest.finish().unwrap();
    let marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .unwrap();
    let make_proof = || {
        worth_proof::AdmittedBlobReleaseProof::certification_admit(
            serving.store_identity().bytes(),
            object.bytes(),
            published.generation().sequence(),
            marker.record().allocation_epoch(),
            marker.record().ordinal(),
            marker.encoded_digest(),
            [0x71; 32],
        )
        .unwrap()
    };
    let request = |proof| {
        worth_store::physical_runtime::BlobReclaimRequest::released(
            proof,
            placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            worth_store::physical_runtime::BlobReclaimLimits::new(
                std::num::NonZeroU64::new(128).unwrap(),
                std::num::NonZeroU64::new(32 << 20).unwrap(),
                std::num::NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        )
    };
    park_at_descriptor_wal(
        &serving,
        root,
        object.bytes(),
        session,
        request(make_proof()),
    );
}

pub(crate) fn park_at_descriptor_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    root: &Path,
    object: [u8; 16],
    session: [u8; 16],
    request: BlobReclaimRequest,
) {
    let first = serving.pause_physical_mutation_at(
        worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability,
    );
    let ready = marker_path(root);
    std::thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
            let mut arrived = false;
            while std::time::Instant::now() < deadline {
                if first.await_arrival() {
                    arrived = true;
                    break;
                }
            }
            assert!(arrived, "manifest WAL not reached");
            let second =
                serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability);
            first.release();
            arrived = false;
            while std::time::Instant::now() < deadline {
                if second.await_arrival() {
                    arrived = true;
                    break;
                }
            }
            assert!(arrived, "reservation WAL not reached");
            let third =
                serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability);
            second.release();
            arrived = false;
            while std::time::Instant::now() < deadline {
                if third.await_arrival() {
                    arrived = true;
                    break;
                }
            }
            assert!(
                arrived,
                "released V3 descriptor WAL not reached before {deadline:?}"
            );
            write_marker(&ready, object, session);
        });
        let _ = serving.blobs().unwrap().reclaim(request).unwrap().wait();
        panic!("released drop escaped WAL-durable pause before process kill");
    });
}
