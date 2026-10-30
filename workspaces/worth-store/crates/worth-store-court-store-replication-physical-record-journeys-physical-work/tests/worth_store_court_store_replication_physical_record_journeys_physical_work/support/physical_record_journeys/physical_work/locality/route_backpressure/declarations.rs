pub(crate) use tempfile::tempdir;
pub(crate) use worth_store_physical_format::RecordArtifactFile;
pub(crate) use {
    worth_store::physical_runtime::PhysicalSignalAspectBindingSet,
    worth_store::physical_runtime::PhysicalWorkCapacity,
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration,
    worth_store::physical_runtime::PhysicalWorkReadiness,
};

pub(crate) use {
    crate::physical_record_journeys::physical_work::locality::admit_read,
    crate::physical_record_journeys::physical_work::locality::projection_fact,
    crate::physical_record_journeys::physical_work::locality::read_dependency,
    crate::physical_record_journeys::physical_work::locality::request,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_named_contract,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
};

pub(crate) fn await_route_depth(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    route: worth_store::physical_runtime::PhysicalSignalAspectBindingDigest,
    expected: usize,
) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if serving.certification_physical_signal_route_depth(route) == Some(expected) {
            return true;
        }
        std::thread::yield_now();
    }
    false
}

pub(crate) fn await_signal_cleanup(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        let observation = serving.physical_signal_observation().unwrap();
        if observation.active_locality_count() == 0 && observation.active_in_flight_count() == 0 {
            return true;
        }
        std::thread::yield_now();
    }
    false
}
