//! A same-object V3 successor survives a selected idle accumulator-only checkpoint.

#[allow(unused_imports)]
use crate::production_entry::accumulator_only_pending_wal::*;
use {
    std::fs, std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::path::PathBuf,
    std::process::Command, std::process::Stdio, std::sync::atomic::AtomicBool,
    std::sync::atomic::Ordering, std::thread, std::time::Duration, std::time::Instant,
};

use {sha2::Digest, sha2::Sha256};
use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
};
use {
    worth_store_physical_format::decode_blob_record,
    worth_store_physical_format::release_checkpoint_batch_records_digest_v1,
    worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::ReleasedDropCumulativeEvidenceV1,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

use {
    crate::production_entry::accumulator_only_pending_wal::selected_media::selected_blob_control,
    crate::production_entry::accumulator_only_pending_wal::selected_media::selected_certificates,
    crate::production_entry::accumulator_only_pending_wal::selected_media::selected_released_source,
};

#[test]
fn same_object_pending_v3_follows_idle_accumulator_tip() {
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first();
    let root = world.root();
    let serving = recover_serving(root);
    serving
        .retire_displaced_segment()
        .expect("first recovered V3 extent retires through C1 checkpoint");
    let (first_batches, first_accumulator) = selected_certificates(root);
    assert_eq!(first_batches.len(), 1, "C1 folds the real first V3 WAL");
    assert_eq!(first_accumulator.base().batch_count(), 1);
    assert_eq!(
        first_accumulator.base().tip(),
        first_batches[0].tip_provenance().unwrap()
    );
    assert!(
        !first_accumulator.base().terminal(),
        "first drop remains nonterminal"
    );
    let first_heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            root,
            first_accumulator,
        );
    assert_eq!(first_heads.len(), 1);
    assert!(!first_heads[0].terminal());

    checkpoint(&serving, [0xc2; 32]);
    let (idle_batches, idle_accumulator) = selected_certificates(root);
    assert!(idle_batches.is_empty(), "C2 has no new released Batch");
    assert_eq!(idle_accumulator.base().batch_count(), 0);
    assert_eq!(
        idle_accumulator.base().tip(),
        first_accumulator.base().tip()
    );
    assert_eq!(
        idle_accumulator.base().cumulative_dropped(),
        first_accumulator.base().cumulative_dropped()
    );
    assert_eq!(
        idle_accumulator.base().cumulative_digest(),
        first_accumulator.base().cumulative_digest()
    );
    assert!(!idle_accumulator.base().terminal());
    assert_eq!(
        idle_accumulator.prior_head_count(),
        first_accumulator.head_count()
    );
    assert_eq!(
        idle_accumulator.prior_head_roster_digest(),
        first_accumulator.head_roster_digest()
    );
    let idle_heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            root,
            idle_accumulator,
        );
    assert_eq!(idle_heads, first_heads);
    serving.close();

    kill_successor_after_descriptor_wal(root);
    let serving = recover_serving(root);
    checkpoint(&serving, [0xc3; 32]);
    serving.close();
    let (successor_batches, successor_accumulator) = selected_certificates(root);
    assert_eq!(successor_batches.len(), 1, "C3 folds the pending V3");
    assert_eq!(successor_accumulator.base().batch_count(), 1);
    let predecessor = successor_batches[0]
        .predecessor()
        .expect("same-object successor names the first descriptor");
    assert_eq!(
        predecessor.descriptor_record(),
        idle_heads[0].descriptor_record()
    );
    assert_eq!(
        predecessor.descriptor_frame_sha256(),
        idle_heads[0].descriptor_frame_sha256()
    );
    assert_eq!(
        successor_accumulator.base().tip(),
        successor_batches[0].tip_provenance().unwrap()
    );
    assert!(
        successor_accumulator.base().cumulative_dropped()
            > idle_accumulator.base().cumulative_dropped()
    );
    assert_eq!(
        successor_accumulator.base().prior_cumulative_dropped(),
        idle_accumulator.base().cumulative_dropped()
    );
    assert_eq!(
        successor_accumulator.base().prior_cumulative_digest(),
        idle_accumulator.base().cumulative_digest()
    );
    assert_eq!(
        successor_accumulator.base().batch_records_digest(),
        release_checkpoint_batch_records_digest_v1(&successor_batches).unwrap()
    );
    let successor_heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            root,
            successor_accumulator,
        );
    assert_eq!(successor_heads.len(), 1);
    assert_eq!(successor_heads[0].key(), idle_heads[0].key());
    assert_eq!(
        successor_heads[0].descriptor_record(),
        successor_batches[0].descriptor_record()
    );
    assert!(successor_heads[0].cumulative_dropped() > idle_heads[0].cumulative_dropped());

    let fresh = recover_serving(root);
    let descriptor_bytes = selected_blob_control(&fresh, successor_batches[0].descriptor_record());
    let worth_store_physical_format::BlobRecordV1::ReclaimDescriptorV3(descriptor) =
        decode_blob_record(&descriptor_bytes).unwrap()
    else {
        panic!("C3 selected descriptor must be the genuine V3 control")
    };
    let manifest_bytes = selected_blob_control(&fresh, descriptor.base().manifest_record());
    let worth_store_physical_format::BlobRecordV1::DropSetManifestV3(manifest) =
        decode_blob_record(&manifest_bytes).unwrap()
    else {
        panic!("C3 selected manifest must be the genuine V3 control")
    };
    let batch = successor_batches[0];
    assert_eq!(
        <[u8; 32]>::from(sha2::Sha256::digest(descriptor.encode())),
        batch.descriptor_frame_sha256()
    );
    assert_eq!(descriptor.custody_digest(), batch.custody_digest());
    assert_eq!(manifest.count(), descriptor.base().manifest_count());
    let evidence = worth_store_physical_format::ReleasedDropCumulativeEvidenceV1::new(
        batch.descriptor_record(),
        batch.descriptor_frame_sha256(),
        descriptor.custody_digest(),
        batch.reservation_record(),
        batch.reservation_frame_sha256(),
        batch.fate(),
        batch.candidate_root_generation(),
        batch.candidate_root_sha256(),
        batch.predecessor(),
        manifest.count(),
        descriptor.base().terminal(),
    )
    .unwrap();
    let (expected_count, expected_digest) = evidence
        .advance(
            idle_accumulator.base().cumulative_dropped(),
            idle_accumulator.base().cumulative_digest(),
        )
        .unwrap();
    assert_eq!(batch.cumulative_dropped(), expected_count);
    assert_eq!(batch.cumulative_digest(), expected_digest);
    assert_eq!(
        successor_accumulator.base().cumulative_dropped(),
        expected_count
    );
    assert_eq!(
        successor_accumulator.base().cumulative_digest(),
        expected_digest
    );
    fresh.close();
}

#[test]
fn successor_child() {
    let Some(root) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let marker = std::path::PathBuf::from(std::env::var_os(MARKER_ENV).expect("successor marker"));
    let serving = recover_serving(std::path::Path::new(&root));
    let source = selected_released_source(&serving);
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        source.object(),
        source.generation(),
        source.publication_record().allocation_epoch(),
        source.publication_record().ordinal(),
        source.publication_frame_sha256(),
        source.issuer_evidence_sha256(),
    )
    .unwrap();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(
            worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
                worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
                    .admit()
                    .unwrap(),
            ),
        )
        .unwrap();
    let request = worth_store::physical_runtime::BlobReclaimRequest::released(
        proof,
        placement,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(128).unwrap(),
            std::num::NonZeroU64::new(32 << 20).unwrap(),
            std::num::NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    park_at_descriptor_wal(&serving, request, &marker);
}
