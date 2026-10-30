pub use {crate::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::produced_rewrite_payloads};
pub use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint, worth_store::physical_runtime::PreparedPhysicalMutation, worth_proof::TransitionOutcome, crate::physical_record_journeys::configuration, crate::physical_record_journeys::durability_admission::managed_mutation::prepare, crate::physical_record_journeys::serving_from_initialization};
pub use {worth_signal::facade::TemporalDuration};
pub use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits};
pub use {worth_store_physical_backend::MediaOperationRole};

pub fn prepare_rewrite(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
) -> PreparedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    match submission
        .rewrite_selected_inline_segment(
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::at(
                    worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
                ),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("selected segment rewrite preparation must return a prepared mutation")
        }
        worth_proof::TransitionOutcome::Denied(_) => {
            panic!("selected segment rewrite preparation was denied")
        }
        worth_proof::TransitionOutcome::Deferred(_) => {
            panic!("selected segment rewrite preparation was deferred")
        }
        worth_proof::TransitionOutcome::Stale(_) => {
            panic!("selected segment rewrite preparation was stale")
        }
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("selected segment rewrite preparation required a rebind")
        }
        worth_proof::TransitionOutcome::Failed(_) => {
            panic!("selected segment rewrite preparation failed")
        }
    }
}

pub fn completed(
    outcome: PhysicalMutationOutcome,
) -> worth_store::physical_runtime::CompletedPhysicalMutation {
    match outcome {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(completed) => completed,
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("rewrite journey proved no effect: {:?}", fate.cause())
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite journey became indeterminate at {:?}", fate.stage())
        }
    }
}
