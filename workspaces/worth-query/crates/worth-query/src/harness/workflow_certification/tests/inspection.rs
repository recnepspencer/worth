use crate::harness::fixtures::{
    execution_preflights, relational_merge_inspection::deleted_vs_modified_inspection_artifact,
};
use crate::workflow::{
    admit_query_workflow_declaration, bind_workflow_context, inspect_merge_conflicts,
    lower_merge_workflow_declaration, MergeLoweringInput, WorkflowAuthorityTargetFamily,
    WorkflowBindingSource, WorkflowBudgetClass, WorkflowCostClass, WorkflowDeclarationFamily,
    WorkflowDeclarationRequest, WorkflowFreshnessPolicy,
};
use worth_relational::facade::history::BranchId;

#[test]
fn workflow_certification_conflict_inspection_preserves_lower_authority_merge_class() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let preflight = execution_preflights::direct_runtime_preflight();
    let binding = bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
        .expect("runtime binding should admit");
    let declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::MergeLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMerge,
            WorkflowCostClass::MergeLoweringNarrow,
            WorkflowBudgetClass::AuthorityTargetBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("merge declaration should admit");
    let lowered = lower_merge_workflow_declaration(
        &declaration,
        MergeLoweringInput::reconcile_into_target(
            BranchId("main".to_string()),
            BranchId("candidate".to_string()),
        ),
    )
    .expect("merge lowering should succeed");
    let inspection_declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::ConflictInspectionNarrow,
            WorkflowAuthorityTargetFamily::QueryInspection,
            WorkflowCostClass::InspectionNarrow,
            WorkflowBudgetClass::InspectionBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("conflict inspection declaration should admit");

    let inspection = inspect_merge_conflicts(
        &inspection_declaration,
        &lowered,
        &deleted_vs_modified_inspection_artifact(request_execution),
    )
    .expect("inspection should succeed");
    let deletion_row = inspection
        .rows()
        .iter()
        .find(|row| row.merge_class() == "deletion:deleted_vs_modified")
        .expect("deleted-vs-modified row should be present");

    assert_eq!(deletion_row.merge_class(), "deletion:deleted_vs_modified");
    assert_eq!(
        deletion_row.merge_class_admission(),
        &crate::workflow::MergeClassAdmission::ExecutionDenied
    );
}
