pub(crate) use {std::fs, std::sync::mpsc, std::sync::mpsc::TryRecvError, std::time::Duration, std::time::Instant};

pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::TemporalDuration};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint};
pub(crate) use {worth_store::physical_runtime::DataDispatchedPhysicalMutation, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointProgressPhase, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalDataSettlementOutcome, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store::physical_runtime::WalDurablePhysicalMutation};
pub(crate) use {worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) fn dispatch_dirty_data(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    payload: Vec<u8>,
) -> DataDispatchedPhysicalMutation {
    let submission = serving.certification_record_submission();
    let (durable, _end_lsn) = durable_wal(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x41; 32]),
        &payload,
    );
    match submission.dispatch_wal_durable_data(durable) {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        _ => panic!("siege setup requires real dirty data effects"),
    }
}

pub(crate) fn durable_wal(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
    payload: &[u8],
) -> (WalDurablePhysicalMutation, u64) {
    let prepared = worth_store_testkit_store::physical_record_journeys::durability_admission::wal_append::prepared(submission, placement, material, payload);
    let appended = match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(prepared, Vec::new())) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("foreground WAL mutation must append while checkpoint is paused"),
    };
    let end_lsn = appended.members()[0]
        .mutation()
        .reserved()
        .declaration()
        .lsn_range()
        .end_exclusive()
        .get();
    let durable = match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(durable) => durable
            .into_members()
            .into_vec()
            .pop()
            .expect("singleton WAL group derives one member"),
        _ => panic!("foreground WAL mutation must cross its barrier"),
    };
    (durable, end_lsn)
}

pub(crate) fn start_checkpoint(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> worth_store::physical_runtime::PhysicalCheckpointHandle {
    match serving
        .checkpoints()
        .start(checkpoint_request(key))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint siege requires an admitted handle"),
    }
}

pub(crate) fn checkpoint_request(key: u8) -> PhysicalCheckpointRequest {
    worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(worth_signal::facade::TemporalDuration::temporal_duration(10_000).unwrap()),
    )
}

pub(crate) fn mutation_material(index: u64) -> PhysicalMutationIdempotencyMaterial {
    let mut material = [0_u8; 32];
    material[..8].copy_from_slice(&index.to_le_bytes());
    material[8] = 0x9d;
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material)
}

pub(crate) fn assert_checkpoint_io(
    before: worth_store_physical_backend::MediaCounterSnapshot,
    after: worth_store_physical_backend::MediaCounterSnapshot,
    records: &[&[u8]],
) {
    const PHYSICAL_EFFECT_RECOVERY_RECORD_BYTES: u64 = 160;
    // Candidate and namespace synchronization are pure flushes: they journal
    // no recovery record unless their outcome is retained.
    const SYNCHRONIZATION_ONLY_ACTIONS: u64 = 2;

    let delta = |role| {
        after
            .attempts_for(role)
            .saturating_sub(before.attempts_for(role))
    };
    let checkpoint_actions = records.len() as u64 + 3;
    let journaled_actions = checkpoint_actions - SYNCHRONIZATION_ONLY_ACTIONS;
    assert_eq!(delta(worth_store_physical_backend::MediaOperationRole::CreateNew), journaled_actions + 1);
    assert_eq!(
        delta(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        records.len() as u64
    );
    assert_eq!(delta(worth_store_physical_backend::MediaOperationRole::Append), journaled_actions);
    assert_eq!(
        delta(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState),
        journaled_actions + 1
    );
    assert_eq!(delta(worth_store_physical_backend::MediaOperationRole::Delete), journaled_actions);
    assert_eq!(delta(worth_store_physical_backend::MediaOperationRole::AtomicReplace), 1);
    // Each journaled action: its record's barrier and its durable retirement.
    // Plus the published namespace's barrier.
    assert_eq!(
        delta(worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication),
        2 * journaled_actions + 1
    );
    assert_eq!(
        after.completed_bytes_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.completed_bytes_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        records
            .iter()
            .map(|record| record.len() as u64)
            .sum::<u64>()
    );
    assert_eq!(
        after.completed_bytes_for(worth_store_physical_backend::MediaOperationRole::Append)
            - before.completed_bytes_for(worth_store_physical_backend::MediaOperationRole::Append),
        journaled_actions * PHYSICAL_EFFECT_RECOVERY_RECORD_BYTES
    );
}

pub(crate) fn checkpoint_records(bytes: &[u8]) -> Vec<&[u8]> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        assert!(bytes.len() - offset >= 20);
        let payload = u32::from_le_bytes(bytes[offset + 12..offset + 16].try_into().unwrap());
        let end = offset + 16 + payload as usize + 4;
        assert!(end <= bytes.len());
        records.push(&bytes[offset..end]);
        offset = end;
    }
    records
}