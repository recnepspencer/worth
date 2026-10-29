//! Advance an unrelated root past the original reservation's lease and WAL.

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalWalReclamationObservation, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_format::PersistedRecordIdentity};

pub(crate) use {worth_store_testkit_store::fixture::placement};
pub(crate) use {crate::physical_blob_journeys::blob_reclaim_crash::persisted, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records};

pub(crate) fn age_and_prune_original_reservation(
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
        let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
            worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
            worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(120_000).unwrap(),
        );
        let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw()
        else {
            panic!("post-expiry checkpoint {ordinal} must admit");
        };
        let worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) = handle.wait() else {
            panic!("post-expiry checkpoint {ordinal} must complete");
        };
        reclaimed |= matches!(completed.wal_reclamation(),
            worth_store::physical_runtime::PhysicalWalReclamationObservation::Reclaimed(value) if value.reclaimed_segments() > 0);
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

pub(crate) fn append_unrelated_padding(serving: &ServingPhysicalRuntime, ordinal: u8) {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(
            [0xd0 + ordinal; 32],
        ))
        .unwrap();
    let frame = [ordinal; 64 * 1024];
    let batch = worth_store::physical_runtime::RecordAppendBatch::try_from_iter([frame.as_slice()]).unwrap();
    let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                batch,
                placement(),
                worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                    key,
                    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("unrelated WAL-rotation append must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
}