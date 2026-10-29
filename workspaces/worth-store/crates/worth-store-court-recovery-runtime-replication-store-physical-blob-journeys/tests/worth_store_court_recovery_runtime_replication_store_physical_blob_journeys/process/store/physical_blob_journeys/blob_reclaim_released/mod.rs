#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reclaim_released::*;
use {std::num::NonZeroU16, std::num::NonZeroU64};

use worth_proof::AdmittedBlobReleaseProof;
use worth_proof::TransitionOutcome;
use worth_store_blob_chunks::BlobChunkSize;
use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReclaimDeferral,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalPageSizeClass,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

use {
    crate::physical_blob_journeys::blob_ingest_process::observed_without_damage,
    crate::physical_blob_journeys::blob_ingest_process::selected_root,
};
use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
};

#[test]
fn released_generation_one_record_batches_keep_post_order_custody_and_end_in_no_effect() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.release.post_order.scope");
    let blobs = serving.blobs().unwrap();
    let read_limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
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
        .begin_ingest(declaration, placement(), CHUNK as u64, read_limits)
        .unwrap();
    ingest.push(&vec![0x31; CHUNK]).unwrap();
    ingest.push(&vec![0x52; CHUNK]).unwrap();
    let abandoned_token = ingest.resume_token();
    let published = ingest.finish().unwrap();

    let marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("published generation has selected C5 marker");
    let release_limits = || {
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(256).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
            std::num::NonZeroU16::new(1).unwrap(),
        )
        .unwrap()
    };
    let proof = |store, frame_sha256| {
        let record = marker.record();
        worth_proof::AdmittedBlobReleaseProof::certification_admit(
            store,
            object.bytes(),
            published.generation().sequence(),
            record.allocation_epoch(),
            record.ordinal(),
            frame_sha256,
            [0x71; 32],
        )
        .unwrap()
    };
    let request = || {
        worth_store::physical_runtime::BlobReclaimRequest::released(
            proof(serving.store_identity().bytes(), marker.encoded_digest()),
            placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
            release_limits(),
        )
    };

    let before_denials =
        worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records(
            &serving,
        );
    let held = serving.records().unwrap();
    assert!(matches!(
        serving.blobs().unwrap().reclaim(request()),
        Err(worth_store::physical_runtime::BlobReclaimFailure::Deferred(
            worth_store::physical_runtime::BlobReclaimDeferral::ProtectedReader
        ))
    ));
    drop(held);
    assert!(matches!(
        serving.blobs().unwrap().reclaim(
            worth_store::physical_runtime::BlobReclaimRequest::abandoned(
                abandoned_token,
                &scope,
                placement(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
                release_limits(),
            )
        ),
        Err(worth_store::physical_runtime::BlobReclaimFailure::AlreadyPublished)
    ));
    assert!(matches!(
        serving.blobs().unwrap().reclaim(
            worth_store::physical_runtime::BlobReclaimRequest::released(
                proof(serving.store_identity().bytes(), [0x91; 32]),
                placement(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
                release_limits(),
            )
        ),
        Err(worth_store::physical_runtime::BlobReclaimFailure::DeclarationMismatch)
    ));
    assert!(matches!(
        serving.blobs().unwrap().reclaim(
            worth_store::physical_runtime::BlobReclaimRequest::released(
                proof([0x92; 16], marker.encoded_digest()),
                placement(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
                release_limits(),
            )
        ),
        Err(worth_store::physical_runtime::BlobReclaimFailure::ForeignStore)
    ));
    assert_eq!(
        worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records(
            &serving
        ),
        before_denials,
        "all negative admissions must leave the selected root unchanged"
    );

    let first = serving
        .blobs()
        .unwrap()
        .reclaim(request())
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        first.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert_eq!(first.dropped_records(), &[marker.record()]);
    assert!(first.remaining_payload_records() > 0);
    assert!(
        !worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records(
            &serving
        )
        .iter()
        .any(|(record, _)| {
            record.allocation_epoch() == marker.record().allocation_epoch()
                && record.ordinal() == marker.record().ordinal()
        }),
        "completed first batch must remove publication from the selected C5 root"
    );
    let mut remaining = first.remaining_payload_records();
    for _ in 0..8 {
        let receipt = serving
            .blobs()
            .unwrap()
            .reclaim(request())
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(
            receipt.disposition(),
            worth_store::physical_runtime::BlobReclaimDisposition::Dropped
        );
        assert_eq!(receipt.dropped_records().len(), 1);
        assert!(receipt.remaining_payload_records() < remaining);
        remaining = receipt.remaining_payload_records();
        if remaining == 0 {
            break;
        }
    }
    assert_eq!(
        remaining, 0,
        "all exclusive publication-tree custody was dropped"
    );
    let repeated = serving
        .blobs()
        .unwrap()
        .reclaim(request())
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        repeated.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::ProvenNoEffect
    );
    assert!(repeated.dropped_records().is_empty());
    let released_descriptors =
        worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records(
            &serving,
        )
        .iter()
        .filter(|(_, bytes)| {
            matches!(
                decode_blob_record(bytes),
                Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptorV3(_))
            )
        })
        .count();
    assert!(
        released_descriptors > 0,
        "the completed release keeps its descriptor selected"
    );
    serving.close();
    // What the offline observer says of a completed release, which is less
    // than "this store is healthy": the observation is complete, no artifact it
    // reaches is damaged, and the release certificate is intact.
    let artifacts =
        observed_without_damage(directory.path(), "c11-blob-release", "post-order-released");
    let mut release_certificates = artifacts
        .iter()
        .filter(|artifact| artifact["family"] == "checkpoint_release_certificate")
        .peekable();
    assert!(
        release_certificates.peek().is_some(),
        "the checkpoint that completed the release certifies it"
    );
    for certificate in release_certificates {
        assert_eq!(certificate["outcome"]["posture"], "intact", "{certificate}");
    }
    // A completed release selects a head-bound root (envelope schema 10). The
    // offline observer does not read that schema yet, so it cannot say this
    // root's routing is intact; it must say so rather than guess. When the
    // observer learns the schema, this becomes the intact assertion.
    let selected = selected_root(&artifacts);
    let manifest = &selected[0]["outcome"];
    assert_eq!(manifest["posture"], "unsupported", "{manifest}");
    assert_eq!(manifest["axis"], "envelope_schema", "{manifest}");
    assert_eq!(manifest["observed"], 10, "{manifest}");
    assert_eq!(
        selected.len(),
        1,
        "no routing block of the unread root is observed: {selected:?}"
    );
    // Under a root it cannot read, the observer reaches no selected blob record,
    // not even the release descriptor counted above, and still reports a
    // complete observation. "Without damage" therefore says nothing about the
    // blob rows of a released store. This pins that gap; it does not excuse it.
    let blob_rows: Vec<_> = artifacts
        .iter()
        .filter(|artifact| {
            artifact["family"]
                .as_str()
                .is_some_and(|family| family.starts_with("blob_"))
        })
        .collect();
    assert!(blob_rows.is_empty(), "{blob_rows:?}");
}

#[test]
fn shared_source_release_requires_recovery_before_reopen() {
    crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::source_first();
}

#[test]
fn cancelled_pre_effect_release_restores_checkpoint_and_reclaim_admission() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.release.cancel.scope");
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
    ingest.push(&vec![0x31; CHUNK]).unwrap();
    ingest.push(&vec![0x52; CHUNK]).unwrap();
    let published = ingest.finish().unwrap();
    let marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .unwrap();
    let request = || {
        let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
            serving.store_identity().bytes(),
            object.bytes(),
            published.generation().sequence(),
            marker.record().allocation_epoch(),
            marker.record().ordinal(),
            marker.encoded_digest(),
            [0x72; 32],
        )
        .unwrap();
        worth_store::physical_runtime::BlobReclaimRequest::released(
            proof,
            placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
            worth_store::physical_runtime::BlobReclaimLimits::new(
                std::num::NonZeroU64::new(256).unwrap(),
                std::num::NonZeroU64::new(64 << 20).unwrap(),
                std::num::NonZeroU16::new(16).unwrap(),
            )
            .unwrap(),
        )
    };
    let handle = serving.blobs().unwrap().reclaim(request()).unwrap();
    drop(handle);
    let checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x73; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(checkpoint).into_raw()
    else {
        panic!("pre-effect cancellation must restore checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    let retry = serving
        .blobs()
        .unwrap()
        .reclaim(request())
        .expect("pre-effect cancellation must restore reclaim admission");
    drop(retry);
    serving.close();
}
#[path = "shared_reuse/mod.rs"]
pub(crate) mod shared_reuse;
