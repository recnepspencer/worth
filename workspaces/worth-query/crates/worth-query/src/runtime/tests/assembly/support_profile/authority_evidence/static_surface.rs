use super::super::super::super::support::*;

#[test]
fn runtime_public_authority_evidence_support_report_freezes_surface() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.authority-evidence-support")
        .expect("task runtime should open a named workspace");
    let report = workspace.public_authoritative_mutation_evidence_support();

    assert_eq!(
        report.backend_posture(),
        WorthQueryRuntimeBackendPosture::Primary
    );
    assert_eq!(
        report.graph_composition_families(),
        &[
            "same_batch_entity_relation_identity_edges",
            "mixed_existing_and_symbolic_entity_identity_edges",
            "same_batch_symbolic_entity_followup_mutation",
            "same_batch_symbolic_relation_followup_mutation",
            "same_batch_symbolic_relation_retirement",
            "mixed_existing_target_followup_mutation",
            "mixed_existing_target_retarget",
            "mixed_existing_target_supersession",
            "mixed_existing_target_retirement",
            "mixed_existing_target_verified_followup_mutation",
            "mixed_existing_target_verified_retarget",
            "mixed_existing_target_verified_supersession",
            "mixed_existing_target_verified_retirement",
        ]
    );
    assert!(report
        .graph_composition_capability_support_rows()
        .iter()
        .any(|row| row.capability_family() == "same_batch_symbolic_relation_followup_mutation"));
    assert!(report
        .graph_composition_capability_support_rows()
        .iter()
        .any(|row| row.capability_family() == "mixed_existing_target_verified_supersession"));
    assert!(report
        .graph_composition_extension_hook_support_rows()
        .iter()
        .any(|row| {
            row.hook_family() == "domain_interpretation_hook"
                && row.boundary() == WorthQueryGraphCompositionExtensionHookBoundary::Interpretation
                && !row.semantic_bypass_allowed()
        }));
    assert!(!report.support_digest().is_empty());
}
