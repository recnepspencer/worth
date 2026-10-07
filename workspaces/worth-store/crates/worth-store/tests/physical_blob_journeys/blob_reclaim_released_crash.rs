//! Real process kill after the released V3 drop descriptor WAL is durable,
//! before its root is published. C8 must roll the exact release forward.

use std::{
    num::{NonZeroU16, NonZeroU64},
    path::Path,
    thread,
    time::{Duration, Instant},
};

use worth_proof::AdmittedBlobReleaseProof;
use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobCheckpointLimit, BlobIngestDeclaration,
    BlobReadLimits, BlobReclaimLimits, BlobReclaimRequest, PhysicalMutationDeadline,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{BlobRecordKind, SelectedRecordContentClass};

use super::{
    blob_crash::{
        establish_recovery_frontier, kill_at, marker_path, recover_closed_store, write_marker,
    },
    fixture::{
        admitted_blob_scope, assert_released_open_requires_recovered_custody, configuration,
        placement, serving_from_initialization,
    },
};

#[path = "blob_reclaim_released_crash/derived_witness.rs"]
mod derived_witness;
#[path = "blob_reclaim_released_crash/repeat_c8.rs"]
mod repeat_c8;
#[path = "blob_reclaim_released_crash/retirement_witness.rs"]
mod retirement_witness;

const ROLE: &str = "crash-released-first-wal";
/// The same seam after a tier epoch is certified by the selected checkpoint.
pub(super) const TIER_ROLE: &str = "crash-released-tier-first-wal";
const SCOPE: &str = "c11.blob.released.redo.scope";
const CHUNK: usize = 64 << 10;

#[test]
fn durable_released_v3_descriptor_replays_exact_publication_drop_and_retirement() {
    let world = kill_at(ROLE, Duration::from_secs(240));
    let (before_root, format) = retirement_witness::selected_root(&world.root);
    let retired_directories = derived_witness::retired_inline_directory_slots(&world.root, format);
    let (publication_record, publication_extent) =
        retirement_witness::selected_publication_extent(&world.root, &before_root);
    recover_closed_store(&world.root);
    let (after_root, after_format) = retirement_witness::selected_root(&world.root);
    assert_eq!(after_format, format);
    assert!(
        !retirement_witness::range_published_free(
            &world.root,
            &after_root,
            publication_extent.arena_range(),
            before_root.generation(),
        ),
        "dropped native extent is still owed retirement, not already published free"
    );
    assert!(
        after_root.requires_maintenance_protocol(),
        "C8 drop root must retain displaced native extent retirement protocol"
    );
    // A released drop leaves a release-custody head, so the only path to
    // Serving is C8's handoff; an ordinary reopen must deny before effects.
    assert_released_open_requires_recovered_custody(&world.root, configuration().0);
    let placements = retirement_witness::selected_placements(&world.root, &after_root);
    for retired_directory in retired_directories {
        assert!(
            placements
                .iter()
                .all(|placement| placement.record() != retired_directory.record),
            "physically witnessed retired inline slot {:?}/{:?} must not become a live route",
            retired_directory.page,
            retired_directory.slot,
        );
    }
    let count = |kind| {
        placements
            .iter()
            .filter(|placement| placement.content_class() == SelectedRecordContentClass::Blob(kind))
            .count()
    };
    assert!(
        placements
            .iter()
            .all(|placement| placement.record() != publication_record),
        "redo must unroute the released publication"
    );
    assert_eq!(count(BlobRecordKind::GenerationPublished), 0);
    assert_eq!(
        (
            count(BlobRecordKind::Chunk),
            count(BlobRecordKind::TreeNode)
        ),
        (2, 1),
        "first bounded batch keeps payload selected"
    );
    assert_eq!(count(BlobRecordKind::DropSetManifestV3), 1);
    assert_eq!(
        count(BlobRecordKind::ReclaimDescriptorV3),
        1,
        "C8 must publish the durable V3 descriptor"
    );
    // The dropped publication was the indexed watermark: the same root binds a
    // replacement directory that no longer names it, rather than clearing it.
    let before_directory = before_root
        .derived_family_directory()
        .expect("released publication is indexed by the selected directory");
    assert_eq!(
        before_directory
            .indexed_through_blob_publication()
            .map(|publication| publication.record()),
        Some(publication_record)
    );
    let after_directory = after_root
        .derived_family_directory()
        .expect("C8 drop root binds the replacement directory");
    assert_ne!(
        after_directory.directory_record(),
        before_directory.directory_record()
    );
    assert_eq!(after_directory.indexed_through_blob_publication(), None);
    assert!(placements
        .iter()
        .any(|placement| placement.record() == after_directory.directory_record()));
    assert_eq!(after_root.latest_blob_publication(), None);
}

pub(super) fn child(root: &Path, role: &str) {
    assert!(role == ROLE || role == TIER_ROLE);
    let serving = serving_from_initialization(root);
    establish_recovery_frontier(&serving);
    if role == TIER_ROLE {
        super::clean_reopen_custody::certify_tier_epoch(&serving);
    }
    let scope = admitted_blob_scope(SCOPE);
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, limits)
        .unwrap();
    ingest.push(&vec![0x83; CHUNK]).unwrap();
    ingest.push(&vec![0x94; CHUNK]).unwrap();
    let session = ingest.session_id().bytes();
    let published = ingest.finish().unwrap();
    drop(blobs);
    let marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .unwrap();
    let make_proof = || {
        AdmittedBlobReleaseProof::certification_admit(
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
        BlobReclaimRequest::released(
            proof,
            placement(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            BlobReclaimLimits::new(
                NonZeroU64::new(128).unwrap(),
                NonZeroU64::new(32 << 20).unwrap(),
                NonZeroU16::new(1).unwrap(),
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

fn park_at_descriptor_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    root: &Path,
    object: [u8; 16],
    session: [u8; 16],
    request: BlobReclaimRequest,
) {
    let first = serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
    let ready = marker_path(root);
    thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(180);
            let mut arrived = false;
            while Instant::now() < deadline {
                if first.await_arrival() {
                    arrived = true;
                    break;
                }
            }
            assert!(arrived, "manifest WAL not reached");
            let second =
                serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
            first.release();
            arrived = false;
            while Instant::now() < deadline {
                if second.await_arrival() {
                    arrived = true;
                    break;
                }
            }
            assert!(arrived, "reservation WAL not reached");
            let third =
                serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
            second.release();
            arrived = false;
            while Instant::now() < deadline {
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
