use std::fs;

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalCheckpointStartDenial, PhysicalMutationIdempotencyMaterial,
    PhysicalWalGroupAppendOutcome, PhysicalWalGroupBarrierOutcome,
};

use super::super::{configuration, serving_from_initialization};

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
        TransitionOutcome::Denied(PhysicalCheckpointStartDenial::NoDurableWalSource)
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
    let prepared = super::wal_append::prepared(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([101; 32]),
        b"checkpoint-redo",
    );
    let appended = match submission.append_prepared_wal_group(NonEmpty::new(prepared, Vec::new())) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("checkpoint setup requires one exact appended WAL member"),
    };
    let expected_wal = appended.members()[0]
        .mutation()
        .reserved()
        .declaration()
        .lsn_range();
    match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::Durable(_) => {}
        _ => panic!("checkpoint setup requires the exact WAL barrier"),
    }

    let handle = match serving
        .checkpoints()
        .start(checkpoint_request(2))
        .into_raw()
    {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint admission did not produce a handle"),
    };
    let published = match handle.wait() {
        PhysicalCheckpointOutcome::Completed(published) => published,
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
    let bytes = fs::read(&artifact).unwrap();
    assert_eq!(published.encoded_bytes(), bytes.len() as u64);
    let records = checkpoint_records(&bytes);
    let compaction_index = records
        .iter()
        .position(|record| record[9] == 3)
        .expect("checkpoint carries one binding-compaction header");
    assert_eq!(published.dirty_records() as usize, compaction_index - 1);
    assert_eq!(
        published.binding_compaction().binding_count() as usize,
        records.len() - compaction_index - 2
    );
    assert_eq!(published.binding_compaction().generation().get(), 1);
    assert_eq!(published.binding_compaction().unresolved_binding_count(), 1);
    assert_eq!(published.binding_compaction().terminal_binding_count(), 0);
    let issued_after_first = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([102; 32]))
        .unwrap();
    assert_eq!(issued_after_first.lease().issuance_generation().get(), 1);

    let joined = match serving
        .checkpoints()
        .start(checkpoint_request(2))
        .into_raw()
    {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("same checkpoint key must join the completed attempt"),
    };
    let joined = match joined.wait() {
        PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("joined checkpoint observation changed fate: {other:?}"),
    };
    assert_eq!(joined.basis().identity(), basis.identity());
    assert_eq!(joined.binding_compaction().generation().get(), 1);
    let issued_after_join = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([103; 32]))
        .unwrap();
    assert_eq!(issued_after_join.lease().issuance_generation().get(), 1);

    let second = match serving
        .checkpoints()
        .start(checkpoint_request(3))
        .into_raw()
    {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("a distinct checkpoint key must start the next generation"),
    };
    let second = match second.wait() {
        PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("second checkpoint failed: {other:?}"),
    };
    assert_eq!(second.binding_compaction().generation().get(), 2);
    let issued_after_second = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([104; 32]))
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

fn checkpoint_request(key: u8) -> PhysicalCheckpointRequest {
    PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::at(
            TemporalDuration::temporal_duration(1_000).expect("deadline is positive"),
        ),
    )
}

fn checkpoint_records(bytes: &[u8]) -> Vec<&[u8]> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        assert!(
            bytes.len() - offset >= 20,
            "checkpoint record prefix is truncated"
        );
        let payload = u32::from_le_bytes(bytes[offset + 12..offset + 16].try_into().unwrap());
        let end = offset + 16 + payload as usize + 4;
        assert!(end <= bytes.len(), "checkpoint record exceeds the artifact");
        records.push(&bytes[offset..end]);
        offset = end;
    }
    assert!(
        records.len() >= 2,
        "checkpoint requires a header and footer"
    );
    records
}
