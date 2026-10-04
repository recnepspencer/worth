//! Publishing one generation and admitting its release, as the fixture
//! issuer does for every world of this family.

use std::num::NonZeroU64;

use worth_proof::AdmittedBlobReleaseProof;
use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobIngestFailure, BlobIngestSession, BlobObjectId,
    BlobReadLimits, BlobResumeToken, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::{deadline, CHUNK, SCOPE};
use crate::fixture::{admitted_blob_scope, placement};

/// One published generation: its admitted release, and what its ended
/// session and its object can still be asked.
pub(super) struct Published {
    pub(super) proof: AdmittedBlobReleaseProof,
    pub(super) token: BlobResumeToken,
    pub(super) object: BlobObjectId,
}

fn read_limits() -> BlobReadLimits {
    BlobReadLimits::new(NonZeroU64::new(128).unwrap())
}

/// Declares one ingest of `chunks` whole chunks into `object`, resumable for
/// `horizon` completed checkpoints.
pub(super) fn begin(
    serving: &ServingPhysicalRuntime,
    object: BlobObjectId,
    chunks: usize,
    horizon: u64,
) -> Result<BlobIngestSession<'_>, BlobIngestFailure> {
    let scope = admitted_blob_scope(SCOPE);
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (chunks * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(horizon).unwrap(),
        deadline(),
    )
    .unwrap();
    let blobs = serving.blobs().unwrap();
    blobs.begin_ingest(declaration, placement(), CHUNK as u64, read_limits())
}

/// Publishes one two-chunk generation and admits its release proof. Its
/// session stays resumable for `horizon` completed checkpoints.
pub(super) fn publish_resumable(
    serving: &ServingPhysicalRuntime,
    seed: u8,
    horizon: u64,
) -> Published {
    publish_chunks(
        serving,
        &[vec![seed; CHUNK], vec![!seed; CHUNK]],
        &[],
        horizon,
    )
}

/// Publishes one generation of the given whole chunks. The ingest checkpoints
/// a resume frontier at each of the `frontier_prefixes` chunk counts, besides
/// the frontier it checkpoints by itself every 64 chunks.
pub(super) fn publish_chunks(
    serving: &ServingPhysicalRuntime,
    chunks: &[Vec<u8>],
    frontier_prefixes: &[usize],
    horizon: u64,
) -> Published {
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(read_limits()).unwrap();
    let mut ingest = begin(serving, object, chunks.len(), horizon).unwrap();
    let token = ingest.resume_token();
    for (pushed, chunk) in chunks.iter().enumerate() {
        ingest.push(chunk).unwrap();
        if frontier_prefixes.contains(&(pushed + 1)) {
            ingest.checkpoint().unwrap();
        }
    }
    let published = ingest.finish().unwrap();
    let marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("published generation has a selected marker");
    let record = marker.record();
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        record.allocation_epoch(),
        record.ordinal(),
        marker.encoded_digest(),
        [0x71; 32],
    )
    .unwrap();
    Published {
        proof,
        token,
        object,
    }
}

/// The fixture issuer admits the same release again for a repeated request.
pub(super) fn again(proof: &AdmittedBlobReleaseProof) -> AdmittedBlobReleaseProof {
    AdmittedBlobReleaseProof::certification_admit(
        proof.store(),
        proof.object(),
        proof.generation(),
        proof.publication_allocation_epoch(),
        proof.publication_record_ordinal(),
        proof.publication_frame_sha256(),
        proof.issuer_evidence_sha256(),
    )
    .unwrap()
}
