//! A non-global-tip A head survives native WAL cutover and authorizes A's successor.

#[allow(unused_imports)]
use crate::production_entry::release_reopen::per_object_pruned_successor::*;
use {
    crate::production_entry::release_reopen::per_object_pruned_successor::released_world_with_wal_segment_bytes,
    std::num::NonZeroU16, std::num::NonZeroU64, worth_proof::AdmittedBlobReleaseProof,
    worth_proof::TransitionOutcome, worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::two_generations,
    worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle,
    worth_store_testkit_recovery_runtime::selected_tag7_pruned_wal,
};
use {std::fmt::Write, std::fs, std::path::Path, std::process::Command};
use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
};
use {
    worth_store_physical_format::decode_blob_record,
    worth_store_physical_format::release_checkpoint_batch_records_digest_v1,
    worth_store_physical_format::BlobReclaimSourceBasisV1,
    worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::ReleaseCustodyHeadKeyV1,
    worth_store_physical_format::ReleasedGenerationReclaimBasisV1,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

#[test]
fn non_global_a_head_survives_pruned_wal_and_fresh_successor() {
    let wal_bytes = std::num::NonZeroU64::new(WAL_SEGMENT_BYTES).unwrap();
    let (world, first, first_publication, _) = released_world_with_wal_segment_bytes(1, wal_bytes);
    assert!(
        first.remaining_payload_records() > 0,
        "A remains nonterminal"
    );
    let (a_batches, a_accumulator) = selected_release_certificates(&world);
    assert_eq!(a_batches.len(), 1);
    let first_heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            world.root(),
            a_accumulator,
        );
    let [a_head] = first_heads.as_slice() else {
        panic!("first checkpoint must select exactly A's keyed head");
    };
    assert!(!a_head.terminal());
    let a_key = a_head.key();
    let a_wal = a_batches[0].fate();
    let c1 = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();

    let (b_accumulator, b_key) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::two_generations::release_second_object(&world, first_publication, a_accumulator);
    assert_ne!(a_key, b_key);
    assert_eq!(
        b_accumulator.base().tip().descriptor_record(),
        selected_release_certificates(&world).0[0].descriptor_record()
    );
    let b_heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            world.root(),
            b_accumulator,
        );
    assert_eq!(b_heads.len(), 2);
    assert_eq!(
        b_heads.iter().find(|head| head.key() == a_key),
        Some(a_head)
    );
    let b_head = *b_heads.iter().find(|head| head.key() == b_key).unwrap();
    let c2 = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    assert_ne!(c2, c1, "B must publish a successor checkpoint");

    worth_store_testkit_recovery_runtime::selected_tag7_pruned_wal::rotate_completed_wal(
        world.serving(),
    );
    checkpoint(world.serving(), [0xa3; 32]);
    let c3 = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    assert_ne!(
        c3, c2,
        "native checkpoint replacement supersedes B's checkpoint"
    );
    let (idle_batches, idle_accumulator) = selected_release_certificates(&world);
    assert!(idle_batches.is_empty());
    assert_eq!(idle_accumulator.base().tip(), b_accumulator.base().tip());
    assert_eq!(
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            world.root(),
            idle_accumulator
        ),
        b_heads,
    );
    assert!(
        !worth_store_testkit_recovery_runtime::selected_tag7_pruned_wal::retained_wal_interval(
            world.root(),
            a_wal.lsn_start(),
            a_wal.lsn_end_exclusive(),
        ),
        "native checkpoint cutover must physically prune A's old C9 member"
    );

    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROOT_ENV, &root)
        .env(OBJECT_ENV, encode_object(a_key.object()))
        .env(GENERATION_ENV, a_key.generation().to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fresh A successor process: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let c4 = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    assert_ne!(c4, c3, "A successor must publish its own checkpoint");
    let (successor_batches, successor_accumulator) = selected_release_certificates_from_bytes(&c4);
    let [successor] = successor_batches.as_slice() else {
        panic!("A successor checkpoint must contain one new Batch");
    };
    let predecessor = successor.predecessor().expect("A successor predecessor");
    assert_eq!(predecessor.descriptor_record(), a_head.descriptor_record());
    assert_eq!(
        predecessor.descriptor_frame_sha256(),
        a_head.descriptor_frame_sha256()
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
        successor_accumulator.base().tip(),
        successor.tip_provenance().unwrap()
    );
    assert_eq!(
        successor_accumulator.base().cumulative_dropped(),
        successor.cumulative_dropped(),
    );
    assert_eq!(
        successor_accumulator.base().cumulative_digest(),
        successor.cumulative_digest(),
    );
    assert_eq!(
        successor_accumulator.base().batch_records_digest(),
        release_checkpoint_batch_records_digest_v1(&successor_batches).unwrap(),
    );
    assert_eq!(
        successor_accumulator.prior_head_count(),
        idle_accumulator.head_count()
    );
    assert_eq!(
        successor_accumulator.prior_head_roster_digest(),
        idle_accumulator.head_roster_digest(),
    );
    let final_heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            &root,
            successor_accumulator,
        );
    assert_eq!(final_heads.len(), 2);
    assert_eq!(
        final_heads.iter().find(|head| head.key() == b_key),
        Some(&b_head)
    );
    let a_next = final_heads.iter().find(|head| head.key() == a_key).unwrap();
    assert_eq!(a_next.descriptor_record(), successor.descriptor_record());
    assert_eq!(
        a_next.descriptor_frame_sha256(),
        successor.descriptor_frame_sha256(),
    );
    assert_eq!(a_next.cumulative_dropped(), a_head.cumulative_dropped() + 1,);
    assert_eq!(a_next.source_basis_digest(), a_head.source_basis_digest());

    let serving = recover_serving(&root, wal_bytes);
    crate::release_reopen::per_object_pruned_successor::selected_controls::assert_selected_successor_controls(
        &serving,
        *a_next,
        *successor,
        idle_accumulator,
    );
    checkpoint(&serving, [0xa4; 32]);
    serving.close();
    let (carry_batches, carry_accumulator) = selected_release_certificates_from_bytes(
        &std::fs::read(root.join("families/checkpoint.current")).unwrap(),
    );
    assert!(carry_batches.is_empty());
    assert_eq!(
        carry_accumulator.base().tip(),
        successor_accumulator.base().tip()
    );
    assert_eq!(
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            &root,
            carry_accumulator
        ),
        final_heads
    );
    recover_serving(&root, wal_bytes).close();
}

#[test]
fn fresh_child_continues_a() {
    let Some(root) = std::env::var_os(ROOT_ENV) else {
        return;
    };
    let root = std::path::Path::new(&root);
    let object = decode_object(&std::env::var(OBJECT_ENV).unwrap());
    let generation = std::env::var(GENERATION_ENV).unwrap().parse().unwrap();
    let a_key =
        worth_store_physical_format::ReleaseCustodyHeadKeyV1::new(object, generation).unwrap();
    let wal_bytes = std::num::NonZeroU64::new(WAL_SEGMENT_BYTES).unwrap();
    let serving = recover_serving(root, wal_bytes);
    let (_, accumulator) = selected_release_certificates_from_bytes(
        &std::fs::read(root.join("families/checkpoint.current")).unwrap(),
    );
    let heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            root,
            accumulator,
        );
    let a_head = heads.iter().find(|head| head.key() == a_key).unwrap();
    assert!(!a_head.terminal());
    let source = selected_source_for_key(&serving, a_key);
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
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(request)
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        receipt.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert_eq!(receipt.dropped_records().len(), 1);
    serving.close();
}
