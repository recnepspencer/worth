pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::completed, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) fn positioned_writes(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
}

pub(crate) fn assert_denied_before_effects(
    serving: &ServingPhysicalRuntime,
    outcome: PhysicalMutationOutcome,
    writes: u64,
    journey: &str,
) {
    match outcome {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::RetentionPressure
        ),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {
            panic!("{journey} must be denied before WAL effects")
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("{journey} must prove no effect before WAL effects")
        }
    }
    assert_eq!(serving.certification_pending_publication_count(), 0);
    assert_eq!(positioned_writes(serving), writes);
}
