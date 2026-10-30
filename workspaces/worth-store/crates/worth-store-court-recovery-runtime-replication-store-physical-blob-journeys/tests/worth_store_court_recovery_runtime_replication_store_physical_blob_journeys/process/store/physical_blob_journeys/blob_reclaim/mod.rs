#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reclaim::*;
use {std::num::NonZeroU16, std::num::NonZeroU64};

use worth_store_blob_chunks::BlobChunkSize;
use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReclaimDeferral,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimReceipt,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::BlobReclaimRetirement,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalWorkCounterStage,
    worth_store::physical_runtime::PhysicalWorkOperationFamily,
    worth_store::physical_runtime::PhysicalWorkPressureClass,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_abort::deadline,
    crate::physical_blob_journeys::blob_abort::limits as terminal_limits,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

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
        Err(worth_store::physical_runtime::BlobReclaimFailure::Deferred(
            worth_store::physical_runtime::BlobReclaimDeferral::ProtectedReader
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
    assert_eq!(
        cancelled.cancel(),
        worth_store::physical_runtime::BlobReclaimDisposition::ProvenNoEffect
    );
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
    assert_eq!(
        receipt.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert_eq!(receipt.dropped_records().len(), 1);
    assert_eq!(receipt.remaining_payload_records(), 2);
    assert_completed_retirement(&receipt);
    assert!(
        serving.physical_work_counters().count_under_pressure(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactPublication,
            worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundBlobReclaim,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
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
        assert_eq!(
            receipt.disposition(),
            worth_store::physical_runtime::BlobReclaimDisposition::Dropped
        );
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
        worth_store::physical_runtime::BlobReclaimDisposition::ProvenNoEffect
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
    let ingest = crate::physical_blob_journeys::blob_abort::unfinished(&serving, &scope);
    let token = ingest.resume_token();
    drop(ingest);
    let before = selected_blob_records(&serving);
    assert!(matches!(
        serving.blobs().unwrap().reclaim(request(token, &scope)),
        Err(worth_store::physical_runtime::BlobReclaimFailure::NotAbandoned)
    ));
    assert_eq!(selected_blob_records(&serving), before);
    serving
        .blobs()
        .unwrap()
        .abort_ingest(token, &scope, placement(), deadline(), terminal_limits())
        .unwrap();
    let before = selected_blob_records(&serving);
    for (records, bytes) in [(1, 8 << 20), (128, 1)] {
        let request = worth_store::physical_runtime::BlobReclaimRequest::abandoned(
            token,
            &scope,
            placement(),
            deadline(),
            worth_store::physical_runtime::BlobReclaimLimits::new(
                std::num::NonZeroU64::new(records).unwrap(),
                std::num::NonZeroU64::new(bytes).unwrap(),
                std::num::NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        );
        let result = serving.blobs().unwrap().reclaim(request);
        match (records, result) {
            (1, Err(worth_store::physical_runtime::BlobReclaimFailure::ScanBoundExhausted)) => {}
            (
                128,
                Err(worth_store::physical_runtime::BlobReclaimFailure::InspectedByteBoundExhausted),
            ) => {}
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
        worth_store::physical_runtime::BlobReclaimDisposition::ProvenNoEffect,
        "denials must release their session claim"
    );
    serving.close();
}
#[cfg(all())]
#[path = "retirement_tests/mod.rs"]
pub(crate) mod retirement_tests;
