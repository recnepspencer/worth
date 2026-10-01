//! Advance an unrelated root past the original reservation's lease and WAL.

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalWalReclamationObservation, RecordAppendBatch, ServingPhysicalRuntime,
};
use worth_store_physical_format::PersistedRecordIdentity;

use super::super::fixture::placement;
use super::{persisted, selected_blob_records};

pub(super) fn age_and_prune_original_reservation(
    serving: &ServingPhysicalRuntime,
    lease_expiry: u64,
    manifest: PersistedRecordIdentity,
    reserved: PersistedRecordIdentity,
) {
    let original_segment = serving
        .record_submission()
        .wal_observation()
        .unwrap()
        .segment();
    for ordinal in 0..8_u8 {
        append_unrelated_padding(serving, ordinal);
    }
    let mut reclaimed = false;
    let mut expired_at_checkpoint = None;
    for ordinal in 0..16_u8 {
        let mut key = [0xd9; 32];
        key[0] = ordinal;
        let request = PhysicalCheckpointRequest::fuzzy(
            PhysicalCheckpointIdempotencyKey::new(key),
            PhysicalCheckpointDeadline::after_milliseconds(120_000).unwrap(),
        );
        let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw()
        else {
            panic!("post-expiry checkpoint {ordinal} must admit");
        };
        let PhysicalCheckpointOutcome::Completed(completed) = handle.wait() else {
            panic!("post-expiry checkpoint {ordinal} must complete");
        };
        reclaimed |= matches!(completed.wal_reclamation(),
            PhysicalWalReclamationObservation::Reclaimed(value) if value.reclaimed_segments() > 0);
        let retained_first = completed
            .retained_wal_tail()
            .segments()
            .first()
            .expect("checkpoint retains an active WAL segment")
            .artifact()
            .segment()
            .get();
        if completed.footer().identity().sequence().get() >= lease_expiry {
            expired_at_checkpoint = Some((
                completed.footer().identity().sequence().get(),
                retained_first,
            ));
            break;
        }
    }
    let (selected_checkpoint, retained_first) = expired_at_checkpoint
        .expect("bounded completed checkpoints must cross original lease expiry");
    assert!(
        selected_checkpoint >= lease_expiry,
        "lease expiry is in namespace-durable checkpoint coordinates"
    );
    assert!(
        retained_first > original_segment && reclaimed,
        "checkpoint must prune the original reservation's WAL segment"
    );
    let selected = selected_blob_records(serving);
    assert!(selected
        .iter()
        .any(|(record, _)| persisted(*record) == manifest));
    assert!(selected
        .iter()
        .any(|(record, _)| persisted(*record) == reserved));
}

fn append_unrelated_padding(serving: &ServingPhysicalRuntime, ordinal: u8) {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
            [0xd0 + ordinal; 32],
        ))
        .unwrap();
    let frame = [ordinal; 64 * 1024];
    let batch = RecordAppendBatch::try_from_iter([frame.as_slice()]).unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                batch,
                placement(),
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("unrelated WAL-rotation append must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
}
