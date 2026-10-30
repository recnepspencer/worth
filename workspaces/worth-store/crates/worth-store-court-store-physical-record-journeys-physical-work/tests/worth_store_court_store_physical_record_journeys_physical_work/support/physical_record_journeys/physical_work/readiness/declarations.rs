pub(crate) use tempfile::tempdir;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::PhysicalWorkPreEffectDenial,
    worth_store::physical_runtime::PhysicalWorkReadiness,
    worth_store::physical_runtime::PhysicalWorkSubmissionOutcome,
    worth_store::physical_runtime::PhysicalWorkSubmissionReceipt,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::family_locality_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::matching_aspect_delta,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::EXPECTED_NATIVE_RECORD_BINDING_COUNT,
};

pub(crate) fn success(outcome: PhysicalWorkSubmissionOutcome) -> PhysicalWorkSubmissionReceipt {
    match outcome.into_raw() {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        outcome => panic!("physical work should be declared: {outcome:?}"),
    }
}

pub(crate) fn ready_read(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: worth_store::physical_runtime::PhysicalReadWorkRequest,
) -> worth_store::physical_runtime::ReadyPhysicalWork {
    let admitted = serving
        .admit_physical_work(success(serving.physical_read_submission().submit(request)))
        .unwrap();
    match serving.request_physical_work(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
        worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(blocked) => {
            panic!(
                "physical work unexpectedly blocked: {:?}",
                blocked.condition()
            )
        }
    }
}

pub(crate) fn readiness_descriptors(
    ready: &worth_store::physical_runtime::ReadyPhysicalWork,
) -> (String, String, String) {
    (
        ready.capability_registry_digest().to_owned(),
        ready.capability_bundle_digest().to_owned(),
        ready.payload_contract_digest().to_owned(),
    )
}

pub(crate) fn await_no_active_signal(
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
            "dropped readiness retained Signal state"
        );
        std::thread::yield_now();
    }
}
