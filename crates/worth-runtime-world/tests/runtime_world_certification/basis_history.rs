use worth_runtime_world::facade::{
    CompositeCallerCorrelation, CompositeCommitParent, RuntimeWorldPublicationPhase,
};

#[test]
fn publication_phase_vocabulary_is_complete_for_diagnostics() {
    // Phase 1 intentionally fences only the public diagnostic vocabulary.
    // Real consuming token progression belongs to the later owner lanes.
    let phases = [
        RuntimeWorldPublicationPhase::CompositePublicationIntent,
        RuntimeWorldPublicationPhase::ResolvedExpectedProductHead,
        RuntimeWorldPublicationPhase::AdmittedCompositeRuntimeWorldBasis,
        RuntimeWorldPublicationPhase::LoweredOwnerComponentPlan,
        RuntimeWorldPublicationPhase::ReservedCompositePublicationAttempt,
        RuntimeWorldPublicationPhase::OwnerExecutionSettlement,
        RuntimeWorldPublicationPhase::CompositePublicationReady,
        RuntimeWorldPublicationPhase::RuntimeWorldPublicationOutcome,
    ];

    for (index, phase) in phases.iter().enumerate() {
        assert!(phases[..index].iter().all(|prior| prior != phase));
    }
}

#[test]
fn immutable_history_keeps_root_parentage_and_correlation_descriptive() {
    let parent = CompositeCommitParent::Root;
    assert!(matches!(parent, CompositeCommitParent::Root));

    let correlation = CompositeCallerCorrelation::new(17);
    assert_eq!(correlation, CompositeCallerCorrelation::new(17));
    assert_ne!(correlation, CompositeCallerCorrelation::new(18));
}
