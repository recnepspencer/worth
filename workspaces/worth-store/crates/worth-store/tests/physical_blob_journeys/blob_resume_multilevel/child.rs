use std::{
    fs,
    num::NonZeroU64,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobCheckpointLimit, BlobIngestDeclaration,
    BlobIngestSession, BlobReadLimits, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalMutationDeadline, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::super::{
    blob_crash::{
        establish_recovery_frontier, marker_path, resume_token_path, write_marker, SCOPE_KEY,
    },
    fixture::{admitted_blob_scope, placement, serving_from_initialization},
};

pub(super) const CHUNK_BYTES: usize = 64 * 1024;
pub(super) const CHUNKS: usize = 4097;
pub(super) const TOTAL_BYTES: usize = CHUNK_BYTES * CHUNKS;
const PHYSICAL_CHECKPOINT_INTERVAL: usize = 64;

pub(in super::super) fn run(root: &Path) {
    let serving = serving_from_initialization(root);
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let limits = BlobReadLimits::new(NonZeroU64::new(8192).unwrap());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        TOTAL_BYTES as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(1024).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(super::SCHEDULED_WORKLOAD_MILLIS).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
        .expect("multilevel declaration must be selected before source input");
    let token = resume_token_path(root);
    let pending = token.with_extension("pending");
    fs::write(&pending, ingest.resume_token().encode()).unwrap();
    fs::rename(pending, token).unwrap();
    let session = ingest.session_id().bytes();
    let mut source = vec![0; CHUNK_BYTES];
    for ordinal in 0..CHUNKS {
        fill_source(ordinal, &mut source);
        ingest
            .push(&source)
            .unwrap_or_else(|error| panic!("multilevel chunk {ordinal}: {error:?}"));
        if (ordinal + 1) % PHYSICAL_CHECKPOINT_INTERVAL == 0 {
            checkpoint(&serving, (ordinal + 1) as u64);
        }
    }
    assert_eq!(ingest.frontier().next_chunk_ordinal(), CHUNKS as u64);
    park_after_interior_root(&serving, ingest, root, object.bytes(), session);
}

fn checkpoint(serving: &ServingPhysicalRuntime, completed_chunks: u64) {
    let mut key = [0xd4; 32];
    key[..8].copy_from_slice(&completed_chunks.to_le_bytes());
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(120_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("physical checkpoint admission after {completed_chunks} chunks failed");
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "physical checkpoint after {completed_chunks} chunks: {outcome:?}"
    );
}

fn park_after_interior_root(
    serving: &ServingPhysicalRuntime,
    ingest: BlobIngestSession<'_>,
    root: &Path,
    object: [u8; 16],
    session: [u8; 16],
) -> ! {
    // The full first leaf was selected during push. Finish emits the partial
    // second leaf and then the interior root, each through ordinary C5.
    let leaf = serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterRootReplacement);
    thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(180);
            while Instant::now() < deadline {
                if leaf.await_arrival() {
                    let interior = serving.pause_physical_mutation_at(
                        PhysicalMutationCheckpoint::AfterRootReplacement,
                    );
                    leaf.release();
                    while Instant::now() < deadline {
                        if interior.await_arrival() {
                            write_marker(&marker_path(root), object, session);
                            return;
                        }
                    }
                    panic!("interior root did not reach selected-root pause");
                }
            }
            panic!("partial second leaf did not reach selected-root pause");
        });
        let _ = ingest.finish();
        panic!("generation publication escaped selected-interior pause before kill");
    })
}

pub(super) fn fill_source(ordinal: usize, target: &mut [u8]) {
    for (offset, byte) in target.iter_mut().enumerate() {
        let position = (ordinal * CHUNK_BYTES + offset) as u64;
        *byte = (position.wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ (position >> 17)
            ^ (position >> 18).wrapping_mul(37)) as u8;
    }
}
