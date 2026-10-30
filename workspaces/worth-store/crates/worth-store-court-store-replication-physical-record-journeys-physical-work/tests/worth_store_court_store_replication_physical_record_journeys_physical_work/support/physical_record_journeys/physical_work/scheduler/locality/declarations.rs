pub(crate) use tempfile::tempdir;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_io_scheduler::QueueLocalityRelation;
pub(crate) use {
    worth_foundational::aspects, worth_foundational::FoundationalPerformanceWorkClass,
};
pub(crate) use {
    worth_store::aspect_native::StoreAspectAuthorityInput,
    worth_store::aspect_native::StoreAspectBoundaryFact,
    worth_store::physical_runtime::PhysicalReadWorkRequest,
    worth_store::physical_runtime::PhysicalSchedulerDemand,
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration,
    worth_store::physical_runtime::PhysicalWorkScope,
    worth_store::physical_runtime::PhysicalWorkSemanticBasis,
};
pub(crate) use {
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_contract,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::validated_value,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_read_work,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand,
    worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt_for,
};

pub(crate) fn scheduler_admit(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: PhysicalReadWorkRequest,
) -> worth_store::physical_runtime::ResourceAdmittedPhysicalWork {
    let ready = ready_read_work(serving, request);
    let demand = worth_store::physical_runtime::PhysicalSchedulerDemand::foreground(
        ready,
        worth_store_testkit_store::physical_work::scheduler_reservation::reserved_buffered_file_read(serving),
        None,
    )
    .unwrap();
    let work = demand.queue_work();
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let demand = secure_demand(demand, &backend);
    serving
        .admit_physical_scheduler_demand(
            demand,
            &backend,
            policy_receipt_for(
                requested_budget,
                0,
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeRead,
            ),
        )
        .unwrap()
}

pub(crate) fn coordinate(
    artifact: RecordArtifactFile,
    offset: u64,
    length: u32,
) -> RecordFrameCoordinate {
    worth_store_physical_format::RecordFrameCoordinate::new(artifact, offset, length).unwrap()
}
