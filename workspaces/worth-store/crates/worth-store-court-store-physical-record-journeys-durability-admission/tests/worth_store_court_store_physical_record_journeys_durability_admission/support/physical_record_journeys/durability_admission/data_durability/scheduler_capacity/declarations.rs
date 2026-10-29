pub(crate) use {std::sync::mpsc};
pub(crate) use {std::time::Duration};

pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint};
pub(crate) use {worth_store::physical_runtime::PhysicalDataDispatchFailureCause, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalRecordMutationFailureCause, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::PhysicalWorkCapacity, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::WalDurablePhysicalMutation};
pub(crate) use {worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial, worth_store_io_scheduler::foreground_reservation::ForegroundReservationResourceShortfall, worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration};
pub(crate) use {worth_store_testkit_store::durable_publication::wal_data::prepare_single, worth_store_testkit_store::durable_publication::root::publish_single};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

pub(crate) static SEED: [u8; 20_000] = [0x61; 20_000];
pub(crate) const LEFT: &[u8] = b"scheduler capacity left";
pub(crate) const RIGHT: &[u8] = b"scheduler capacity right";
pub(crate) const RETRY: &[u8] = b"scheduler capacity retry";
pub(crate) const PAGE_GRANT_BYTES: u64 = 16_384;
pub(crate) const SCHEDULER_BANDWIDTH_BYTES: u64 = 44 * 1_024;
pub(crate) const BANDWIDTH_AFTER_TWO_DATA_EFFECTS: u64 = SCHEDULER_BANDWIDTH_BYTES - (PAGE_GRANT_BYTES * 2);

pub(crate) fn prepared(
    submission: &worth_store::physical_runtime::PhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    idempotency: [u8; 32],
    bytes: &[u8],
) -> PreparedPhysicalMutation {
    match prepare_single(
        submission,
        placement,
        worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(idempotency),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([bytes]).unwrap(),
    )
    .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("canonical preparation must succeed"),
    }
}

pub(crate) fn exact_bandwidth_denial(outcome: PhysicalDataDispatchOutcome) -> WalDurablePhysicalMutation {
    let worth_store::physical_runtime::PhysicalDataDispatchOutcome::NotStarted {
        durable,
        cause: worth_store::physical_runtime::PhysicalDataDispatchFailureCause::Canonical(evidence),
    } = outcome
    else {
        panic!("scheduler exhaustion must be an explicit canonical pre-effect denial");
    };
    assert!(matches!(
        evidence.cause(),
        worth_store::physical_runtime::PhysicalRecordMutationFailureCause::SchedulerReservationDenied(
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial::Foreground(
                worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial::InsufficientCapacity(
                    worth_store_io_scheduler::foreground_reservation::ForegroundReservationResourceShortfall::BandwidthToken {
                        requested: PAGE_GRANT_BYTES,
                        available: BANDWIDTH_AFTER_TWO_DATA_EFFECTS,
                    }
                )
            )
        )
    ));
    durable
}

pub(crate) fn dispatched(
    outcome: PhysicalDataDispatchOutcome,
) -> worth_store::physical_runtime::DataDispatchedPhysicalMutation {
    match outcome {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        _ => panic!("the WAL-durable member must dispatch"),
    }
}