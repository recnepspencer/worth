#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::checkpoint_capture::*;
use {std::fs};

use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {worth_signal::facade::TemporalDuration};
use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalCheckpointStartDenial, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome};
use {worth_store_physical_format::decode_checkpoint_certificate, worth_store_physical_format::CheckpointCertificateKind, worth_store_physical_format::ReleaseCheckpointCertificateV1};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};



#[test]
fn checkpoint_without_durable_wal_is_rejected_before_candidate_creation() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_from_initialization(&store_root);

    assert!(matches!(
        serving
            .checkpoints()
            .start(checkpoint_request(1))
            .into_raw(),
        worth_proof::TransitionOutcome::Denied(worth_store::physical_runtime::PhysicalCheckpointStartDenial::NoDurableWalSource)
    ));
    assert!(!store_root.join("families/checkpoint.current").exists());
    assert!(!store_root
        .join("staging/checkpoint-0000000000000001.candidate")
        .exists());

    serving.close();
}

#[test]
fn durable_wal_captures_through_the_ordinary_facade_into_verified_checkpoint_bytes() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_from_initialization(&store_root);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let prepared = worth_store_testkit_store::physical_record_journeys::durability_admission::wal_append::prepared(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([101; 32]),
        b"checkpoint-redo",
    );
    let appended = match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(prepared, Vec::new())) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("checkpoint setup requires one exact appended WAL member"),
    };
    let expected_wal = appended.members()[0]
        .mutation()
        .reserved()
        .declaration()
        .lsn_range();
    match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(_) => {}
        _ => panic!("checkpoint setup requires the exact WAL barrier"),
    }

    let handle = match serving
        .checkpoints()
        .start(checkpoint_request(2))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint admission did not produce a handle"),
    };
    let published = match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(published) => published,
        other => panic!("checkpoint capture failed: {other:?}"),
    };
    let basis = published.basis();
    assert_eq!(basis.identity().store_identity(), serving.store_identity());
    assert_eq!(
        basis.source().wal().admitted_begin_lsn(),
        expected_wal.start().get()
    );
    assert_eq!(
        basis.source().wal().covered_end_lsn_exclusive(),
        expected_wal.end_exclusive().get()
    );
    assert_eq!(published.footer().identity(), basis.identity());

    let artifact = store_root.join("families/checkpoint.current");
    let bytes = std::fs::read(&artifact).unwrap();
    assert_eq!(published.encoded_bytes(), bytes.len() as u64);
    let records = checkpoint_records(&bytes);
    let compaction_index = records
        .iter()
        .position(|record| record[9] == 3)
        .expect("checkpoint carries one binding-compaction header");
    assert_eq!(published.dirty_records() as usize, compaction_index - 1);
    assert_eq!(
        published.binding_compaction().binding_count() as usize,
        records.iter().filter(|record| record[9] == 4).count()
    );
    let certificates: Vec<_> = records.iter().filter(|record| record[9] == 7).collect();
    let [certificate] = certificates.as_slice() else {
        panic!("fresh-genesis checkpoint must attest exactly one NoRelease marker");
    };
    let (kind, payload) = decode_checkpoint_certificate(certificate).unwrap();
    assert_eq!(kind, worth_store_physical_format::CheckpointCertificateKind::ReleasedDrop);
    let worth_store_physical_format::ReleaseCheckpointCertificateV1::NoRelease(marker) =
        worth_store_physical_format::ReleaseCheckpointCertificateV1::decode(payload).unwrap()
    else {
        panic!("a world without released drops must carry positive NoRelease custody");
    };
    assert_eq!(marker.checkpoint(), basis.identity());
    assert_eq!(marker.root_generation(), basis.source().root().generation());
    assert_eq!(published.binding_compaction().generation().get(), 1);
    assert_eq!(published.binding_compaction().unresolved_binding_count(), 1);
    assert_eq!(published.binding_compaction().terminal_binding_count(), 0);
    let issued_after_first = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([102; 32]))
        .unwrap();
    assert_eq!(issued_after_first.lease().issuance_generation().get(), 1);

    let joined = match serving
        .checkpoints()
        .start(checkpoint_request(2))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("same checkpoint key must join the completed attempt"),
    };
    let joined = match joined.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("joined checkpoint observation changed fate: {other:?}"),
    };
    assert_eq!(joined.basis().identity(), basis.identity());
    assert_eq!(joined.binding_compaction().generation().get(), 1);
    let issued_after_join = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([103; 32]))
        .unwrap();
    assert_eq!(issued_after_join.lease().issuance_generation().get(), 1);

    let second = match serving
        .checkpoints()
        .start(checkpoint_request(3))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("a distinct checkpoint key must start the next generation"),
    };
    let second = match second.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("second checkpoint failed: {other:?}"),
    };
    assert_eq!(second.binding_compaction().generation().get(), 2);
    let issued_after_second = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([104; 32]))
        .unwrap();
    assert_eq!(issued_after_second.lease().issuance_generation().get(), 2);
    assert!(!store_root
        .join(format!(
            "staging/checkpoint-{:016x}.candidate",
            basis.identity().sequence().get()
        ))
        .exists());

    serving.close();
}
#[path = "residency_denial/mod.rs"]
pub(crate) mod residency_denial;
