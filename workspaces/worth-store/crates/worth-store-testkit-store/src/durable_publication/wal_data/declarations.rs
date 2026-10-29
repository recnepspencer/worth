pub use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub use {worth_signal::facade::TemporalDuration};
pub use {worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission};
pub use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::DataSettledPhysicalMutation, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalDataSettlementOutcome, worth_store::physical_runtime::PhysicalDurabilityGroupBasis, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordSubmission, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::RecordAppendBatch};

pub struct SettledSingleton {
    pub basis: PhysicalDurabilityGroupBasis,
    pub member: DataSettledPhysicalMutation,
}

pub fn settle_single(
    submission: &CertificationPhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
    batch: RecordAppendBatch,
) -> SettledSingleton {
    settle_single_with_manifest_capacity_transition(
        submission,
        placement,
        worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
        material,
        batch,
    )
}

pub fn prepare_single(
    submission: &PhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
    manifest_capacity_transition: PhysicalManifestCapacityTransition,
    material: PhysicalMutationIdempotencyMaterial,
    batch: RecordAppendBatch,
) -> PhysicalMutationPreparationOutcome {
    let key = submission
        .issue_idempotency_key(material)
        .expect("the explicit test identity must be admitted");
    submission.prepare_durable_append_with_manifest_capacity_transition(
        batch,
        placement,
        manifest_capacity_transition,
        worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
            key,
            worth_store::physical_runtime::PhysicalMutationDeadline::at(
                worth_signal::facade::TemporalDuration::temporal_duration(1_000)
                    .expect("the fixed test deadline is nonzero"),
            ),
        ),
    )
}

pub fn settle_single_with_manifest_capacity_transition(
    submission: &CertificationPhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
    manifest_capacity_transition: PhysicalManifestCapacityTransition,
    material: PhysicalMutationIdempotencyMaterial,
    batch: RecordAppendBatch,
) -> SettledSingleton {
    let prepared = match prepare_single(
        submission,
        placement,
        manifest_capacity_transition,
        material,
        batch,
    )
    .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("the canonical durable preparation must succeed"),
    };
    let appended = match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(prepared, Vec::new())) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("the canonical WAL append must succeed"),
    };
    let basis = appended.basis();
    let durable = match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(durable) => durable
            .into_members()
            .into_vec()
            .pop()
            .expect("a singleton group must derive one durable member"),
        _ => panic!("the canonical WAL barrier must succeed"),
    };
    let dispatched = match submission.dispatch_wal_durable_data(durable) {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        _ => panic!("the exact WAL-durable member must dispatch"),
    };
    let member = match dispatched.settle_exact_effects() {
        worth_store::physical_runtime::PhysicalDataSettlementOutcome::Settled(settled) => settled,
        worth_store::physical_runtime::PhysicalDataSettlementOutcome::InspectionRequired { cause, .. } => {
            panic!("the exact data effects must settle: {cause:?}")
        }
    };
    SettledSingleton { basis, member }
}