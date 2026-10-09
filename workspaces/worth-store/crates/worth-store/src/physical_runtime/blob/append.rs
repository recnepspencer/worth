use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::{BlobRecordKind, PersistedRecordIdentity};

use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, ServingPhysicalRuntime,
};

use super::BlobSessionId;

/// The failure retains the physical fate rather than converting an uncertain
/// root publication into an apparently retryable, effect-free blob operation.
pub enum BlobAppendFailure {
    Idempotency(PhysicalMutationIdempotencyIssuanceDenial),
    Preparation(PhysicalMutationPreparationOutcome),
    ProvenNoEffect(crate::physical_runtime::ProvenNoEffectPhysicalMutation),
    Indeterminate(crate::physical_runtime::IndeterminatePhysicalMutation),
    MissingRecordIdentity,
    ExtraRecordIdentities,
}

/// Exact selected C.5 completion, without a later live-root reacquisition.
pub(super) struct CompletedBlobAppend {
    pub(super) record: PersistedRecordIdentity,
    pub(super) root_generation: u64,
}

impl std::fmt::Debug for BlobAppendFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idempotency(_) => f.write_str("IdempotencyIssuance"),
            Self::Preparation(_) => f.write_str("Preparation"),
            Self::ProvenNoEffect(fate) => f
                .debug_struct("ProvenNoEffect")
                .field("cause", &fate.cause())
                .field("admission_detail", &fate.admission_detail())
                .finish(),
            Self::Indeterminate(fate) => f
                .debug_struct("Indeterminate")
                .field("stage", &fate.stage())
                .field("completed_effects", &fate.completed_effect_count())
                .field("root_preparation_failure", &fate.root_preparation_failure())
                .finish(),
            Self::MissingRecordIdentity => f.write_str("MissingRecordIdentity"),
            Self::ExtraRecordIdentities => f.write_str("ExtraRecordIdentities"),
        }
    }
}

pub(super) fn append_blob_record(
    runtime: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    session: BlobSessionId,
    kind: BlobRecordKind,
    ordinal: u64,
    deadline: PhysicalMutationDeadline,
    encoded: Vec<u8>,
) -> Result<PersistedRecordIdentity, BlobAppendFailure> {
    append_blob_record_with_root(
        runtime, placement, session, kind, ordinal, deadline, encoded,
    )
    .map(|completed| completed.record)
}

pub(super) fn append_blob_record_with_root(
    runtime: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    session: BlobSessionId,
    kind: BlobRecordKind,
    ordinal: u64,
    deadline: PhysicalMutationDeadline,
    encoded: Vec<u8>,
) -> Result<CompletedBlobAppend, BlobAppendFailure> {
    append_blob_record_selected(
        runtime, placement, session, kind, ordinal, deadline, encoded, None,
    )
}

pub(super) fn append_blob_reuse_claim(
    runtime: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    session: BlobSessionId,
    ordinal: u64,
    deadline: PhysicalMutationDeadline,
    encoded: Vec<u8>,
    declaration_record: PersistedRecordIdentity,
    declaration_digest: [u8; 32],
) -> Result<PersistedRecordIdentity, BlobAppendFailure> {
    append_blob_record_selected(
        runtime,
        placement,
        session,
        BlobRecordKind::ChunkReuseClaimV2,
        ordinal,
        deadline,
        encoded,
        Some((declaration_record, declaration_digest)),
    )
    .map(|completed| completed.record)
}

pub(super) fn append_blob_dedupe_quarantine(
    runtime: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    session: BlobSessionId,
    ordinal: u64,
    deadline: PhysicalMutationDeadline,
    encoded: Vec<u8>,
    declaration_record: PersistedRecordIdentity,
    declaration_digest: [u8; 32],
) -> Result<PersistedRecordIdentity, BlobAppendFailure> {
    append_blob_record_selected(
        runtime,
        placement,
        session,
        BlobRecordKind::DedupeQuarantine,
        ordinal,
        deadline,
        encoded,
        Some((declaration_record, declaration_digest)),
    )
    .map(|completed| completed.record)
}

fn append_blob_record_selected(
    runtime: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    session: BlobSessionId,
    kind: BlobRecordKind,
    ordinal: u64,
    deadline: PhysicalMutationDeadline,
    encoded: Vec<u8>,
    reuse_declaration: Option<(PersistedRecordIdentity, [u8; 32])>,
) -> Result<CompletedBlobAppend, BlobAppendFailure> {
    let submission = runtime.record_submission();
    let material = mutation_material(session, kind, ordinal);
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .map_err(BlobAppendFailure::Idempotency)?;
    let request = PhysicalMutationRequest::platform_durable(key, deadline);
    let preparation = match reuse_declaration {
        Some((record, digest)) if kind == BlobRecordKind::ChunkReuseClaimV2 => {
            submission.prepare_blob_reuse_claim_append(encoded, record, digest, placement, request)
        }
        Some((record, digest)) if kind == BlobRecordKind::DedupeQuarantine => submission
            .prepare_blob_dedupe_quarantine_append(encoded, record, digest, placement, request),
        Some(_) => {
            unreachable!("specialized declaration basis is only used for claim or quarantine")
        }
        None => submission.prepare_blob_record_append(encoded, placement, request),
    };
    let completed = match preparation.into_raw() {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            match prepared.execute() {
                PhysicalMutationOutcome::Completed(completed) => completed,
                PhysicalMutationOutcome::ProvenNoEffect(fate) => {
                    return Err(BlobAppendFailure::ProvenNoEffect(fate));
                }
                PhysicalMutationOutcome::Indeterminate(fate) => {
                    return Err(BlobAppendFailure::Indeterminate(fate));
                }
            }
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(completed)) => {
            completed
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(fate)) => {
            return Err(BlobAppendFailure::ProvenNoEffect(fate));
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(fate)) => {
            return Err(BlobAppendFailure::Indeterminate(fate));
        }
        other => return Err(BlobAppendFailure::Preparation(other.into())),
    };
    match completed.persisted_records() {
        [record] => {
            // Only this completed append's settled data frames are reloadable
            // here; unrelated warm records retain their cache residency.
            runtime.release_blob_ingest_clean_frames(completed.completed_data_frame_coordinates());
            Ok(CompletedBlobAppend {
                record: *record,
                root_generation: completed.completed_breadth().current_root_generation(),
            })
        }
        [] => Err(BlobAppendFailure::MissingRecordIdentity),
        _ => Err(BlobAppendFailure::ExtraRecordIdentities),
    }
}

fn mutation_material(session: BlobSessionId, kind: BlobRecordKind, ordinal: u64) -> [u8; 32] {
    let mut sha = Sha256::new();
    sha.update(b"worth.store.blob.mutation.v1");
    sha.update(session.bytes());
    sha.update([match kind {
        BlobRecordKind::SessionDeclared => 1,
        BlobRecordKind::Chunk => 2,
        BlobRecordKind::TreeNode => 3,
        BlobRecordKind::GenerationPublished => 4,
        BlobRecordKind::SessionFrontier => 5,
        BlobRecordKind::SessionAbandoned => 6,
        BlobRecordKind::DropSetManifest => 7,
        BlobRecordKind::ReclaimDescriptor => 8,
        BlobRecordKind::DropSetManifestV2 => 9,
        BlobRecordKind::OriginalDropReserved => 10,
        BlobRecordKind::ChunkReuseClaim => 11,
        BlobRecordKind::ChunkReuseClaimV2 => 15,
        BlobRecordKind::DedupeQuarantine => 12,
        BlobRecordKind::DropSetManifestV3 => 13,
        BlobRecordKind::ReclaimDescriptorV2 => 14,
        BlobRecordKind::ReclaimDescriptorV3 => 16,
    }]);
    sha.update(ordinal.to_le_bytes());
    sha.finalize().into()
}
