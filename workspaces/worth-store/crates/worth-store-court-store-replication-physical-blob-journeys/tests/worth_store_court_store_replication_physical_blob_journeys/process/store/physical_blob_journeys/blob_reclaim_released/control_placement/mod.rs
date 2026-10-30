//! Genuine released-drop controls must acquire their complete arena window,
//! including the replacement directory frame when the dropped publication is
//! the directory watermark, before the first control's WAL or root effect.

use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::configuration,
    worth_store_testkit_store::fixture::serving_from_initialization_with_placement,
};

#[test]
fn released_control_arena_window_denies_before_effect_and_completes_when_admitted() {
    // The ingest extent holds one range; Manifest, Reservation, Descriptor and
    // the replacement directory frame need four more.
    for maximum_ranges in [4_u32, 5] {
        let directory = tempfile::tempdir().unwrap();
        let (format, _, _) = configuration();
        let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
            .manifest_capacity(
                worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap(),
            )
            // Existing owner charges 1KiB base + 4KiB per index entry.
            .arena_index_bytes(
                worth_store::physical_runtime::RecordByteLimit::new(1024 + 4096 * maximum_ranges)
                    .unwrap(),
            )
            .admit(format)
            .unwrap();
        let serving = serving_from_initialization_with_placement(directory.path(), placement);
        let scope = admitted_blob_scope("c11.blob.release.control-window.scope");
        let limits = worth_store::physical_runtime::BlobReadLimits::new(
            std::num::NonZeroU64::new(128).unwrap(),
        );
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(64 << 10).unwrap(),
            64 << 10,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement, 32 << 10, limits)
            .unwrap();
        ingest.push(&vec![0x35; 32 << 10]).unwrap();
        ingest.push(&vec![0x35; 32 << 10]).unwrap();
        let published = ingest.finish().unwrap();
        let marker = serving
            .certification_selected_latest_blob_publication()
            .unwrap()
            .unwrap();
        let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
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
            .reclaim(worth_store::physical_runtime::BlobReclaimRequest::released(
                proof,
                placement,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
                worth_store::physical_runtime::BlobReclaimLimits::new(
                    std::num::NonZeroU64::new(256).unwrap(),
                    std::num::NonZeroU64::new(64 << 20).unwrap(),
                    std::num::NonZeroU16::new(1).unwrap(),
                )
                .unwrap(),
            ))
            .expect("genuine publication/release authority must reach execution")
            .wait();
        if maximum_ranges == 4 {
            let Err(worth_store::physical_runtime::BlobReclaimFailure::Publication {
                stage: worth_store::physical_runtime::BlobReclaimPublicationStage::Manifest,
                manifest_record: None,
                cause: worth_store::physical_runtime::BlobAppendFailure::Preparation(outcome),
            }) = result
            else {
                panic!("the directory claim must reject before Manifest effects: {result:?}")
            };
            assert!(matches!(
                outcome.into_raw(),
                worth_proof::TransitionOutcome::Denied(worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
                    worth_store::physical_runtime::RecordAppendDenial::ArenaAllocationUnavailable(
                        worth_store::physical_runtime::ArenaAllocationDenial::RangeBudget {
                            required: 5,
                            maximum: 4,
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
                result.expect("the exact five-entry window must complete the genuine drop");
            assert_eq!(
                receipt.disposition(),
                worth_store::physical_runtime::BlobReclaimDisposition::Dropped
            );
            assert_eq!(receipt.dropped_records(), &[marker.record()]);
            assert_eq!(receipt.remaining_payload_records(), 2);
            assert_eq!(
                receipt.retirement(),
                worth_store::physical_runtime::BlobReclaimRetirement::Completed
            );
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
