//! Genuine released-drop controls must acquire their complete arena window
//! before the first control's WAL or root effect.

use std::num::{NonZeroU16, NonZeroU64};

use worth_proof::{AdmittedBlobReleaseProof, TransitionOutcome};
use worth_store::physical_runtime::{
    ArenaAllocationDenial, BlobAppendFailure, BlobCheckpointLimit, BlobIngestDeclaration,
    BlobReadLimits, BlobReclaimDisposition, BlobReclaimFailure, BlobReclaimLimits,
    BlobReclaimPublicationStage, BlobReclaimRequest, BlobReclaimRetirement, ManifestEntryCapacity,
    PhysicalMutationDeadline, PhysicalMutationPreparationDenial, PhysicalRecordPlacementPolicy,
    RecordAppendDenial, RecordByteLimit,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::super::fixture::{
    admitted_blob_scope, configuration, serving_from_initialization_with_placement,
};

#[test]
fn three_control_arena_window_denies_before_effect_and_completes_when_admitted() {
    for maximum_ranges in [3_u32, 4] {
        let directory = tempfile::tempdir().unwrap();
        let (format, _, _) = configuration();
        let placement = PhysicalRecordPlacementPolicy::builder()
            .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
            // Existing owner charges 1KiB base + 4KiB per index entry.
            .arena_index_bytes(RecordByteLimit::new(1024 + 4096 * maximum_ranges).unwrap())
            .admit(format)
            .unwrap();
        let serving = serving_from_initialization_with_placement(directory.path(), placement);
        let scope = admitted_blob_scope("c11.blob.release.control-window.scope");
        let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(64 << 10).unwrap(),
            64 << 10,
            &scope,
            BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement, 32 << 10, limits)
            .unwrap();
        ingest.push(&vec![0x35; 32 << 10]).unwrap();
        ingest.push(&vec![0x35; 32 << 10]).unwrap();
        let published = ingest.finish().unwrap();
        drop(blobs);
        let marker = serving
            .certification_selected_latest_blob_publication()
            .unwrap()
            .unwrap();
        let proof = AdmittedBlobReleaseProof::certification_admit(
            serving.store_identity().bytes(),
            object.bytes(),
            published.generation().sequence(),
            marker.record().allocation_epoch(),
            marker.record().ordinal(),
            marker.encoded_digest(),
            [0x73; 32],
        )
        .unwrap();
        let before_root = serving
            .records()
            .unwrap()
            .protected_root()
            .root()
            .generation();
        let before_append = serving.media_counters().append_attempts();
        let result = serving
            .blobs()
            .unwrap()
            .reclaim(BlobReclaimRequest::released(
                proof,
                placement,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                BlobReclaimLimits::new(
                    NonZeroU64::new(256).unwrap(),
                    NonZeroU64::new(64 << 20).unwrap(),
                    NonZeroU16::new(1).unwrap(),
                )
                .unwrap(),
            ))
            .expect("genuine publication/release authority must reach execution")
            .wait();
        if maximum_ranges == 3 {
            let Err(BlobReclaimFailure::Publication {
                stage: BlobReclaimPublicationStage::Manifest,
                manifest_record: None,
                cause: BlobAppendFailure::Preparation(outcome),
            }) = result
            else {
                panic!("third control must reject before Manifest effects: {result:?}")
            };
            assert!(matches!(
                outcome.into_raw(),
                TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
                    RecordAppendDenial::ArenaAllocationUnavailable(
                        ArenaAllocationDenial::RangeBudget {
                            required: 4,
                            maximum: 3,
                        }
                    )
                ))
            ));
            assert_eq!(
                serving
                    .records()
                    .unwrap()
                    .protected_root()
                    .root()
                    .generation(),
                before_root
            );
            assert_eq!(serving.media_counters().append_attempts(), before_append);
            assert_eq!(
                serving
                    .certification_selected_latest_blob_publication()
                    .unwrap()
                    .unwrap()
                    .record(),
                marker.record()
            );
        } else {
            let receipt =
                result.expect("the exact four-entry window must complete the genuine drop");
            assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
            assert_eq!(receipt.dropped_records(), &[marker.record()]);
            assert_eq!(receipt.remaining_payload_records(), 2);
            assert_eq!(receipt.retirement(), BlobReclaimRetirement::Completed);
            assert_eq!(receipt.displaced_extents().len(), 1);
            assert_eq!(
                receipt.bytes_released(),
                receipt.displaced_extents()[0].range().length()
            );
            assert_eq!(
                serving
                    .records()
                    .unwrap()
                    .protected_root()
                    .root()
                    .generation()
                    .get(),
                // Manifest, Reservation, Descriptor, then the native owner's
                // durable free-range publication for the retired source.
                before_root.get() + 4
            );
            assert!(serving.media_counters().append_attempts() > before_append);
        }
        serving.close();
    }
}
