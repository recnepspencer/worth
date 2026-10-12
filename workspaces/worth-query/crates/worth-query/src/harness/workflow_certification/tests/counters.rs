use crate::aspect_field_authoring::single_native_string_aspect_field_patch;
use crate::harness::fixtures::{
    execution_preflights, relational_merge_inspection::deleted_vs_modified_inspection_artifact,
};
use crate::workflow::{
    admit_query_workflow_declaration, bind_workflow_context, lower_merge_workflow_declaration,
    lower_mutation_intent_declaration, lower_query_writeback_declaration, MergeLoweringInput,
    MutationLoweringInput, WorkflowAuthorityTargetFamily, WorkflowBindingSource,
    WorkflowBudgetClass, WorkflowCostClass, WorkflowDeclarationFamily, WorkflowDeclarationRequest,
    WorkflowFreshnessPolicy, WritebackLoweringInput,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn workflow_certification_lane_specific_counters_are_exercised() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let preflight = execution_preflights::direct_runtime_preflight();
    let binding = bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
        .expect("runtime binding should admit");

    let mutation_declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::MutationLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMutation,
            WorkflowCostClass::MutationLoweringNarrow,
            WorkflowBudgetClass::AuthorityTargetBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("mutation declaration should admit");
    let authority_binding_identity = binding.basis_identity();
    let mutation_lowered = lower_mutation_intent_declaration(
        &mutation_declaration,
        authority_binding_identity,
        MutationLoweringInput::IntentReconciliation {
            entity_id: EntityId::new(PartitionId(1), 41, 0),
            desired_aspect_fields: single_native_string_aspect_field_patch("name", "name", "after")
                .expect("name patch should lower"),
        },
    )
    .expect("mutation lowering should succeed");
    assert_eq!(
        mutation_lowered
            .counters()
            .workflow_mutation_lowering_count(),
        1
    );
    assert_eq!(
        mutation_lowered.counters().workflow_merge_lowering_count(),
        0
    );

    let merge_declaration = admit_query_workflow_declaration(
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
    let merge_lowered = lower_merge_workflow_declaration(
        &merge_declaration,
        MergeLoweringInput::reconcile_into_target(
            BranchId("main".to_string()),
            BranchId("candidate".to_string()),
        ),
    )
    .expect("merge lowering should succeed");
    assert_eq!(merge_lowered.counters().workflow_merge_lowering_count(), 1);
    assert_eq!(
        merge_lowered.counters().workflow_mutation_lowering_count(),
        0
    );

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
    .expect("inspection declaration should admit");
    let inspection = crate::workflow::inspect_merge_conflicts(
        &inspection_declaration,
        &merge_lowered,
        &deleted_vs_modified_inspection_artifact(request_execution),
    )
    .expect("inspection should succeed");
    assert_eq!(
        inspection.counters().workflow_conflict_inspection_count(),
        1
    );
    assert_eq!(
        inspection.counters().workflow_post_merge_inspection_count(),
        0
    );

    let writeback_declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::WritebackLoweringNarrow,
            WorkflowAuthorityTargetFamily::BridgeWriteback,
            WorkflowCostClass::WritebackLoweringNarrow,
            WorkflowBudgetClass::AuthorityTargetBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("writeback declaration should admit");
    let writeback_lowered = lower_query_writeback_declaration(
        &writeback_declaration,
        WritebackLoweringInput::projected_state_diff(),
    )
    .expect("writeback lowering should succeed");
    assert_eq!(
        writeback_lowered
            .counters()
            .workflow_writeback_declaration_count(),
        1
    );
    assert_eq!(
        writeback_lowered
            .counters()
            .workflow_writeback_causality_binding_count(),
        1
    );
}
