pub(crate) use tempfile::tempdir;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::PhysicalEffectObligation,
    worth_store::physical_runtime::PhysicalExecutorCommand,
    worth_store::physical_runtime::PhysicalStoreCloseOutcome,
    worth_store::physical_runtime::PhysicalWorkCapacity,
    worth_store::physical_runtime::PhysicalWorkCapacityDimension,
    worth_store::physical_runtime::PhysicalWorkReadiness,
    worth_store::physical_runtime::PhysicalWorkSubmissionOutcome,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::family_locality_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};
pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_work,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::write_demand,
    worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt,
};

pub(crate) fn success(
    outcome: PhysicalWorkSubmissionOutcome,
) -> worth_store::physical_runtime::PhysicalWorkSubmissionReceipt {
    match outcome.into_raw() {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        outcome => panic!("physical work should submit: {outcome:?}"),
    }
}

pub(crate) fn await_signal_cleanup(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let observation = serving.physical_signal_observation().unwrap();
        if observation.active_locality_count() == 0 && observation.active_in_flight_count() == 0 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "dropped physical work retained Signal or locality capacity: locality={}, in_flight={}",
            observation.active_locality_count(),
            observation.active_in_flight_count(),
        );
        std::thread::yield_now();
    }
}
