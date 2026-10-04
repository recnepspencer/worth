//! The next released batch of an object that is already under release
//! custody. A successor child recovers the world and parks that batch at its
//! durable descriptor WAL without retirement and without a new checkpoint.

use std::{
    num::{NonZeroU16, NonZeroU64},
    path::Path,
};

use worth_proof::AdmittedBlobReleaseProof;
use worth_store::physical_runtime::{
    BlobReclaimLimits, BlobReclaimRequest, PhysicalMutationDeadline, RecordByteLimit,
    RecordCountLimit, RecordScanOutcome, RecordScanRequest, ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobRecordV1, ReleasedGenerationReclaimBasisV1,
};

use super::park_at_descriptor_wal;

/// Issuer evidence of the object the first child released.
pub(super) const FIRST_OBJECT: [u8; 32] = [0x71; 32];
/// Issuer evidence of the object a distinct child released.
pub(super) const DISTINCT_OBJECT: [u8; 32] = [0x72; 32];
/// The whole manifest capacity. A remainder that fits leaves in one batch.
pub(super) const FULL_BATCH: u16 = 64;

pub(super) fn child(root: &Path, marker: &Path, issuer_evidence: [u8; 32], batch_records: u16) {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        super::first::World::of_child_root().recovery_request(root),
    );
    let handoff = match outcome {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
            "successor C8 blocked at {:?}: denial={:?}, artifact={:?}",
            block.kind,
            block.evidence().planning_denial,
            block.evidence().artifact,
        ),
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
            indeterminate,
        ) => panic!(
            "successor C8 publication indeterminate: handoff={:?}, reopen={:?}",
            indeterminate.handoff_failure(),
            indeterminate.reopen_failure(),
        ),
        other => panic!("successor process could not recover the prior batch: {other:?}"),
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("prior batch C8 custody seal");
    let serving = super::super::certified_release_serving::admit_serving_with_seal(root, seal);
    park_next_batch(
        &serving,
        marker,
        |source| source.issuer_evidence_sha256() == issuer_evidence,
        batch_records,
    );
}

/// Parks the next batch of the one released object that `selects` admits.
pub(super) fn park_next_batch(
    serving: &ServingPhysicalRuntime,
    marker: &Path,
    selects: impl Fn(&ReleasedGenerationReclaimBasisV1) -> bool,
    batch_records: u16,
) {
    let sources = selected_release_sources(serving);
    let mut matching = sources.iter().filter(|source| selects(source));
    let (Some(source), None) = (matching.next(), matching.next()) else {
        panic!(
            "exactly one selected V3 source must match the successor object, found {} sources",
            sources.len()
        )
    };
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        source.object(),
        source.generation(),
        source.publication_record().allocation_epoch(),
        source.publication_record().ordinal(),
        source.publication_frame_sha256(),
        source.issuer_evidence_sha256(),
    )
    .unwrap();
    let request = BlobReclaimRequest::released(
        proof,
        worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
            .manifest_capacity(
                worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap(),
            )
            .admit(
                worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
                    worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
                        .admit()
                        .unwrap(),
                ),
            )
            .unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(32 << 20).unwrap(),
            NonZeroU16::new(batch_records).unwrap(),
        )
        .unwrap(),
    );
    park_at_descriptor_wal(serving, request, marker, b"second-descriptor-wal");
}

/// One source per released object that still has a selected V3 manifest.
fn selected_release_sources(
    serving: &ServingPhysicalRuntime,
) -> Vec<ReleasedGenerationReclaimBasisV1> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(8).unwrap())
                .with_payload_limit(RecordByteLimit::new(4096).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 8192];
    let mut sources: Vec<ReleasedGenerationReclaimBasisV1> = Vec::new();
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for index in 0..batch.records().len() {
            let Some(bytes) = batch.payload(index) else {
                continue;
            };
            if let Ok(BlobRecordV1::DropSetManifestV3(value)) = decode_blob_record(bytes) {
                if let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = value.source_basis() {
                    if sources.iter().all(|known| {
                        known.object() != source.object()
                            || known.generation() != source.generation()
                    }) {
                        sources.push(source);
                    }
                }
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    sources
}
