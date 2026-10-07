use std::num::{NonZeroU16, NonZeroU64};

use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits,
    BlobReclaimDeferral, BlobReclaimDisposition, BlobReclaimFailure, BlobReclaimLimits,
    BlobReclaimReceipt, BlobReclaimRequest, BlobReclaimRetirement, BlobResumeToken,
    PhysicalMutationDeadline, PhysicalWorkCounterStage, PhysicalWorkOperationFamily,
    PhysicalWorkPressureClass, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{decode_blob_record, BlobRecordV1};

use super::{
    blob_abort::{deadline, limits as terminal_limits},
    blob_crash::{establish_recovery_frontier, recover_closed_store},
    blob_frontier::selected_blob_records,
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

const CHUNK: usize = 64 << 10;

#[cfg(feature = "certification-test-authority")]
#[path = "blob_reclaim/retirement_tests.rs"]
mod retirement_tests;

#[test]
fn held_reader_defers_failed_ingest_reclaim_and_one_record_batches_preserve_frontier() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.reclaim.failed.scope");
    let serving = serving_from_initialization(directory.path());
    establish_recovery_frontier(&serving);
    let token = abandoned_prefix(&serving, &scope);
    let before = selected_blob_records(&serving);
    let held = serving.records().unwrap();
    assert!(matches!(
        serving.blobs().unwrap().reclaim(request(token, &scope)),
        Err(BlobReclaimFailure::Deferred(
            BlobReclaimDeferral::ProtectedReader
        ))
    ));
    assert_eq!(
        selected_blob_records(&serving),
        before,
        "reader deferral has no publication effects"
    );
    drop(held);

    let cancelled = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap();
    assert_eq!(cancelled.cancel(), BlobReclaimDisposition::ProvenNoEffect);
    assert_eq!(
        selected_blob_records(&serving),
        before,
        "cancellation releases a pre-effect fence"
    );
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(receipt.dropped_records().len(), 1);
    assert_eq!(receipt.remaining_payload_records(), 2);
    assert_completed_retirement(&receipt);
    assert!(
        serving.physical_work_counters().count_under_pressure(
            PhysicalWorkOperationFamily::ArtifactPublication,
            PhysicalWorkPressureClass::BackgroundBlobReclaim,
            PhysicalWorkCounterStage::Terminal,
        ) > 0,
        "the selected reclaim control frame spent a dedicated background scheduler lease"
    );
    let after = selected_blob_records(&serving);
    assert_eq!(
        count_kind(&after, 2),
        2,
        "surviving prefix chunks are not the first drop"
    );
    assert_eq!(
        count_kind(&after, 5),
        0,
        "frontier is removed before its implicit chunk prefix"
    );
    assert_eq!(
        count_kind(&after, 1),
        1,
        "declaration remains continuation custody"
    );
    assert_eq!(
        count_kind(&after, 6),
        1,
        "Abandoned remains continuation custody"
    );
    serving.close();
    assert_intact(directory.path(), "frontier-removed");

    for remaining in [1, 0] {
        recover_closed_store(directory.path());
        let serving = serving_from_open(directory.path());
        let receipt = serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope))
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
        assert_eq!(receipt.dropped_records().len(), 1);
        assert_eq!(receipt.remaining_payload_records(), remaining);
        assert_completed_retirement(&receipt);
        assert_eq!(
            count_kind(&selected_blob_records(&serving), 2),
            remaining as usize
        );
        serving.close();
        assert_intact(
            directory.path(),
            if remaining == 0 {
                "empty-payload"
            } else {
                "one-chunk"
            },
        );
    }

    recover_closed_store(directory.path());
    let serving = serving_from_open(directory.path());
    let before = selected_blob_records(&serving);
    let repeated = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        repeated.disposition(),
        BlobReclaimDisposition::ProvenNoEffect
    );
    assert!(repeated.dropped_records().is_empty());
    assert_eq!(repeated.bytes_released(), 0);
    assert_eq!(selected_blob_records(&serving), before);
    serving.close();
}

#[test]
fn reclaim_inspection_bounds_and_nonterminal_custody_deny_before_effects() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reclaim.denial.scope");
    let ingest = super::blob_abort::unfinished(&serving, &scope);
    let token = ingest.resume_token();
    drop(ingest);
    let before = selected_blob_records(&serving);
    assert!(matches!(
        serving.blobs().unwrap().reclaim(request(token, &scope)),
        Err(BlobReclaimFailure::NotAbandoned)
    ));
    assert_eq!(selected_blob_records(&serving), before);
    serving
        .blobs()
        .unwrap()
        .abort_ingest(token, &scope, placement(), deadline(), terminal_limits())
        .unwrap();
    let before = selected_blob_records(&serving);
    for (records, bytes) in [(1, 8 << 20), (128, 1)] {
        let request = BlobReclaimRequest::abandoned(
            token,
            &scope,
            placement(),
            deadline(),
            BlobReclaimLimits::new(
                NonZeroU64::new(records).unwrap(),
                NonZeroU64::new(bytes).unwrap(),
                NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        );
        let result = serving.blobs().unwrap().reclaim(request);
        match (records, result) {
            (1, Err(BlobReclaimFailure::ScanBoundExhausted)) => {}
            (128, Err(BlobReclaimFailure::InspectedByteBoundExhausted)) => {}
            (_, Err(cause)) => panic!("wrong inspection denial: {cause:?}"),
            (_, Ok(_)) => panic!("exhausted inspection must not admit a handle"),
        }
        assert_eq!(selected_blob_records(&serving), before);
    }
    assert_eq!(
        serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope))
            .unwrap()
            .cancel(),
        BlobReclaimDisposition::ProvenNoEffect,
        "denials must release their session claim"
    );
    serving.close();
}

fn assert_completed_retirement(receipt: &BlobReclaimReceipt) {
    assert_eq!(
        receipt.retirement(),
        BlobReclaimRetirement::Completed,
        "{receipt:?}"
    );
    assert!(receipt.bytes_released() > 0);
    assert_eq!(
        receipt.bytes_released(),
        receipt
            .displaced_extents()
            .iter()
            .map(|extent| extent.range().length())
            .sum::<u64>()
    );
}

pub(super) fn abandoned_prefix(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
) -> BlobResumeToken {
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (CHUNK * 3) as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(32).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, limits)
        .unwrap();
    ingest.push(&[0x31; CHUNK]).unwrap();
    ingest.push(&[0x52; CHUNK]).unwrap();
    ingest.checkpoint().unwrap();
    let token = ingest.resume_token();
    drop(ingest);
    blobs
        .abort_ingest(token, scope, placement(), deadline(), terminal_limits())
        .unwrap();
    token
}

pub(super) fn request(token: BlobResumeToken, scope: &AdmittedBlobScope) -> BlobReclaimRequest<'_> {
    BlobReclaimRequest::abandoned(
        token,
        scope,
        placement(),
        deadline(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(8 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    )
}

fn count_kind(
    records: &[(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)],
    kind: u8,
) -> usize {
    records
        .iter()
        .filter(|(_, bytes)| bytes.starts_with(b"WRC11BLB"))
        .filter(|(_, bytes)| match decode_blob_record(bytes).unwrap() {
            BlobRecordV1::SessionDeclared(_) => kind == 1,
            BlobRecordV1::Chunk(_) => kind == 2,
            BlobRecordV1::SessionFrontier(_) => kind == 5,
            BlobRecordV1::SessionAbandoned(_) => kind == 6,
            _ => false,
        })
        .count()
}

fn assert_intact(root: &std::path::Path, scenario: &str) {
    let report = observe_closed_store_named(root, "c11-blob-reclaim", scenario);
    assert_eq!(report["completeness"], "complete", "{report}");
    let arena_rows: Vec<_> = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["family"] == "extent_arena_frame")
        .map(|row| format!("{}: {}", row["generation"], row["outcome"]))
        .collect();
    let arena_file_exists = root
        .join("families/records/arenas/arena-0000000000000001.data")
        .exists();
    for artifact in report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["family"]
                .as_str()
                .is_some_and(|family| family.starts_with("blob_"))
        })
    {
        assert_eq!(
            artifact["outcome"]["posture"],
            "intact",
            "scenario={scenario} arena_file_exists={arena_file_exists} arenas={arena_rows:?} artifact={artifact}"
        );
    }
}
