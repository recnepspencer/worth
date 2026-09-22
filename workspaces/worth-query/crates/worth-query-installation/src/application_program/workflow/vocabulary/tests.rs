use super::{
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryWorkflowHistoryReconstructionBudget,
};
use worth_query_declaration::facade::application_program::ApplicationWorkflowComponentLimits;

#[test]
fn workflow_resource_ceiling_covers_every_definition_limit() {
    let component_limits = ApplicationWorkflowComponentLimits::new(6, 4, 30, 40, 12).unwrap();
    let ceiling = WorthQueryApplicationWorkflowResourceCeiling::new(
        10,
        20,
        3,
        component_limits,
        4096,
        8,
        32,
        1024,
    )
    .expect("all workflow ceilings are nonzero");
    assert_eq!(ceiling.maximum_definition_nodes(), 10);
    assert_eq!(ceiling.maximum_definition_connections(), 20);
    assert_eq!(ceiling.maximum_definition_effects(), 3);
    assert_eq!(ceiling.maximum_component_depth(), 4);
    assert_eq!(ceiling.component_limits(), component_limits);
    assert_eq!(ceiling.maximum_canonical_bytes(), 4096);
    assert_eq!(
        ceiling.history_reconstruction_budget(),
        WorthQueryWorkflowHistoryReconstructionBudget::standard()
    );
    let custom = WorthQueryWorkflowHistoryReconstructionBudget::new(5, 32_768).unwrap();
    assert_eq!(
        ceiling
            .with_history_reconstruction_budget(custom)
            .history_reconstruction_budget(),
        custom
    );
}

#[test]
fn zero_canonical_or_runtime_ceiling_is_rejected() {
    assert!(WorthQueryWorkflowHistoryReconstructionBudget::new(0, 1024).is_none());
    assert!(WorthQueryWorkflowHistoryReconstructionBudget::new(1, 0).is_none());
    let components = ApplicationWorkflowComponentLimits::new(6, 4, 30, 40, 12).unwrap();
    assert!(WorthQueryApplicationWorkflowResourceCeiling::new(
        10, 20, 3, components, 0, 8, 32, 1024
    )
    .is_none());
    assert!(WorthQueryApplicationWorkflowResourceCeiling::new(
        10, 20, 3, components, 4096, 8, 0, 1024
    )
    .is_none());
}
