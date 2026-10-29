//! Publishing one generation and admitting its release, as the fixture
//! issuer does for every world of this family.

pub(crate) use {std::num::NonZeroU64};

pub(crate) use {worth_proof::AdmittedBlobReleaseProof};
pub(crate) use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobIngestSession, worth_store::physical_runtime::BlobObjectId, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobResumeToken, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};

pub(crate) use {crate::physical_blob_journeys::terminal_head_retirement::deadline, crate::physical_blob_journeys::terminal_head_retirement::CHUNK, crate::physical_blob_journeys::terminal_head_retirement::SCOPE};
pub(crate) use {worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement};

/// One published generation: its admitted release, and what its ended
/// session and its object can still be asked.
pub(crate) struct Published {
    pub(crate) proof: AdmittedBlobReleaseProof,
    pub(crate) token: BlobResumeToken,
    pub(crate) object: BlobObjectId,
}

pub(crate) fn read_limits() -> BlobReadLimits {
    worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap())
}

/// Declares one ingest of `chunks` whole chunks into `object`, resumable for
/// `horizon` completed checkpoints.
pub(crate) fn begin(
    serving: &ServingPhysicalRuntime,
    object: BlobObjectId,
    chunks: usize,
    horizon: u64,
) -> Result<BlobIngestSession<'_>, BlobIngestFailure> {
    let scope = admitted_blob_scope(SCOPE);
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (chunks * CHUNK) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(horizon).unwrap(),
        deadline(),
    )
    .unwrap();
    let blobs = serving.blobs().unwrap();
    blobs.begin_ingest(declaration, placement(), CHUNK as u64, read_limits())
}

/// Publishes one two-chunk generation and admits its release proof. Its
/// session stays resumable for `horizon` completed checkpoints.
pub(crate) fn publish_resumable(
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
pub(crate) fn publish_chunks(
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
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
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
pub(crate) fn again(proof: &AdmittedBlobReleaseProof) -> AdmittedBlobReleaseProof {
    worth_proof::AdmittedBlobReleaseProof::certification_admit(
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