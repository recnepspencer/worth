#[allow(unused_imports)]
use crate::shared_support as support;

use support::compile_fail::run_compile_fail_bundle;
use support::compile_pass::run_compile_pass_bundle;
use support::dx;

#[test]
fn pleasant_lane_dx_certification() {
    let compile_fail_bundle = dx::compile_fail_bundle();
    assert_eq!(
        compile_fail_bundle.suite(),
        "pleasant_lane_capability_overclaim_compile_boundaries_hold"
    );
    assert_eq!(compile_fail_bundle.cases().len(), 3);

    let compile_pass_bundle = dx::compile_pass_bundle();
    assert_eq!(
        compile_pass_bundle.suite(),
        "pleasant_lane_representative_workflows_compile_cleanly"
    );
    assert_eq!(compile_pass_bundle.cases().len(), 6);

    let proof_shape_digest = dx::proof_shape_digest();
    assert_eq!(proof_shape_digest.suite(), "pleasant_lane_surface");
    assert!(proof_shape_digest
        .entries()
        .contains(&"escape_hatch:raw_module_reexports_semantic_substrate"));

    let transition_digest = dx::transition_digest();
    assert_eq!(
        transition_digest.suite(),
        "pleasant_lane_representative_workflows"
    );
    assert!(transition_digest
        .entries()
        .contains(&"workflow:checked_progression_and_boundary_resume"));

    let failure_digest = dx::failure_digest();
    assert_eq!(
        failure_digest.suite(),
        "pleasant_lane_compile_time_boundaries"
    );
    assert!(failure_digest
        .entries()
        .iter()
        .any(|entry| entry == "compile_fail:pleasant_lane_cannot_skip_progression"));

    let codegen_honesty_report = dx::codegen_honesty_report();
    assert_eq!(
        codegen_honesty_report.suite(),
        "pleasant_lane_hot_path_honesty"
    );
    assert_eq!(
        codegen_honesty_report.verified_scope(),
        "size_layout_and_drop_only"
    );
    assert!(codegen_honesty_report
        .checks()
        .iter()
        .all(|check| check.matches()));
    assert!(!codegen_honesty_report.hidden_dynamic_lookup());
    assert!(!codegen_honesty_report.hidden_virtual_dispatch());
    assert!(!codegen_honesty_report.mandatory_allocation_introduced());

    let docs_audit = dx::documentation_default_path_audit();
    assert_eq!(
        docs_audit.suite(),
        "pleasant_lane_documentation_default_path_audit"
    );
    assert!(docs_audit.readme_teaches_pleasant_first());
    assert!(docs_audit.readme_teaches_raw_escape_hatch());
    assert!(docs_audit.readme_includes_scoped_default_lane());
    assert!(docs_audit.happy_path_workflow_includes_raw_equivalent());
    assert!(docs_audit.happy_path_workflow_uses_raw_import());
    assert!(docs_audit.checked_workflow_teaches_pleasant_and_raw());
    assert!(docs_audit.runtime_readmission_workflow_teaches_pleasant_and_raw());
    assert!(docs_audit.fixed_join_workflow_teaches_pleasant_and_raw());
    assert!(docs_audit.family_lowering_workflow_teaches_pleasant_and_raw());
    assert!(docs_audit.authoring_workflow_teaches_pleasant_and_raw());
    assert!(docs_audit.low_level_workflow_names_raw_escape_hatch());
    assert!(docs_audit.low_level_workflow_uses_raw_import());
    assert!(docs_audit.recipes_feature_teaches_pleasant_and_raw());
    assert!(docs_audit.checked_transitions_feature_teaches_pleasant_and_raw());
    assert!(docs_audit.runtime_readmission_feature_teaches_pleasant_and_raw());
    assert!(docs_audit.ready_join_feature_teaches_pleasant_and_raw());
    assert!(docs_audit.family_lowering_feature_teaches_pleasant_and_raw());
    assert!(docs_audit.boundary_readmission_feature_teaches_pleasant_and_raw());
    assert!(docs_audit.artifact_feature_declares_dx_posture());
    assert!(docs_audit.assumption_basis_feature_declares_dx_posture());
    assert!(docs_audit.freshness_feature_declares_dx_posture());
    assert!(docs_audit.family_symbol_resolution_feature_declares_dx_posture());
    assert!(docs_audit.family_lifecycle_actions_feature_declares_dx_posture());
    assert!(docs_audit.fixed_shape_collections_feature_declares_dx_posture());
    assert!(docs_audit.fork_and_join_feature_declares_dx_posture());
    assert!(docs_audit.readiness_gates_feature_declares_dx_posture());
    assert!(docs_audit.proof_markers_feature_declares_dx_posture());
    assert!(docs_audit.proven_vectors_feature_declares_dx_posture());
    assert!(docs_audit.structural_facts_feature_declares_dx_posture());
    assert!(docs_audit.transition_outcomes_feature_declares_dx_posture());
    assert!(docs_audit.witnesses_feature_declares_dx_posture());
    assert!(docs_audit.authority_and_workflow_contracts_are_explicit());

    let residual_debt_report = dx::residual_debt_report();
    assert_eq!(residual_debt_report.suite(), "pleasant_lane_closeout_debt");
    assert_eq!(residual_debt_report.items().len(), 2);

    run_compile_fail_bundle(&compile_fail_bundle);
    run_compile_pass_bundle(&compile_pass_bundle);
}
