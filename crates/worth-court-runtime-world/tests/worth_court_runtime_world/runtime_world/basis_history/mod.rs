#[test]
fn publication_phase_vocabulary_is_complete_for_diagnostics() {
    // Phase 1 intentionally fences only the public diagnostic vocabulary.
    // Real consuming token progression belongs to the later owner lanes.
    let phases = [
        worth_runtime_world::facade::RuntimeWorldPublicationPhase::CompositePublicationIntent,
        worth_runtime_world::facade::RuntimeWorldPublicationPhase::ResolvedExpectedProductHead,
        worth_runtime_world::facade::RuntimeWorldPublicationPhase::AdmittedCompositeRuntimeWorldBasis,
        worth_runtime_world::facade::RuntimeWorldPublicationPhase::LoweredOwnerComponentPlan,
        worth_runtime_world::facade::RuntimeWorldPublicationPhase::ReservedCompositePublicationAttempt,
        worth_runtime_world::facade::RuntimeWorldPublicationPhase::OwnerExecutionSettlement,
        worth_runtime_world::facade::RuntimeWorldPublicationPhase::CompositePublicationReady,
        worth_runtime_world::facade::RuntimeWorldPublicationPhase::RuntimeWorldPublicationOutcome,
    ];

    for (index, phase) in phases.iter().enumerate() {
        assert!(phases[..index].iter().all(|prior| prior != phase));
    }
}

#[test]
fn immutable_history_keeps_root_parentage_and_correlation_descriptive() {
    let parent = worth_runtime_world::facade::CompositeCommitParent::Root;
    assert!(matches!(
        parent,
        worth_runtime_world::facade::CompositeCommitParent::Root
    ));

    let correlation = worth_runtime_world::facade::CompositeCallerCorrelation::new(17);
    assert_eq!(
        correlation,
        worth_runtime_world::facade::CompositeCallerCorrelation::new(17)
    );
    assert_ne!(
        correlation,
        worth_runtime_world::facade::CompositeCallerCorrelation::new(18)
    );
}
