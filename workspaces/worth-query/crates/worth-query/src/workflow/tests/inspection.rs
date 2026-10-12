use crate::harness::fixtures::execution_preflights;
use crate::harness::fixtures::relational_merge_inspection::{
    deleted_vs_modified_inspection_artifact, source_addition_inspection_artifact,
    topology_region_conflict_inspection_artifact,
};
use crate::workflow::{
    admit_query_workflow_declaration, bind_workflow_context, inspect_merge_conflicts,
    inspect_post_merge_outcome, lower_merge_workflow_declaration,
    lower_mutation_intent_declaration, shape_merge_authority_outcome,
    shape_mutation_authority_outcome, MergeClassAdmission, MergeLoweringInput,
    MutationLoweringInput, WorkflowAuthorityTargetFamily, WorkflowBindingSource,
    WorkflowBudgetClass, WorkflowCostClass, WorkflowDeclarationFamily, WorkflowDeclarationRequest,
    WorkflowFreshnessPolicy, WorkflowInspectionFailureClass,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn lowered_merge_declaration_can_be_inspected_in_query_shape() {
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

    let inspection_artifact = source_addition_inspection_artifact(request_execution);
    let inspection =
        inspect_merge_conflicts(&inspection_declaration, &lowered, &inspection_artifact)
            .expect("inspection should succeed");
    assert!(!inspection.rows().is_empty());
    let source_addition_row = inspection
        .rows()
        .iter()
        .find(|row| row.merge_class() == "source_only_addition")
        .expect("source-only addition row should be present");
    assert_eq!(
        source_addition_row.merge_class_admission(),
        &MergeClassAdmission::ExecutionAdmissible
    );
    assert_eq!(source_addition_row.merge_class(), "source_only_addition");
    assert_eq!(
        source_addition_row.authority_target_family(),
        &WorkflowAuthorityTargetFamily::RelationalMerge
    );
    assert_eq!(
        inspection.counters().workflow_executor_rediscovery_count(),
        0
    );
    assert_eq!(
        inspection.counters().workflow_conflict_inspection_count(),
        1
    );
    assert_eq!(
        inspection.counters().workflow_post_merge_inspection_count(),
        0
    );
    assert_eq!(
        inspection.prediction_report().predicted_inspection_width(),
        inspection.rows().len()
    );
    assert_eq!(
        inspection.declaration_digest(),
        inspection_declaration.report().declaration_digest()
    );
}

#[test]
fn lowered_merge_declaration_preserves_denied_merge_class_identity() {
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

    let deletion_denied_artifact = deleted_vs_modified_inspection_artifact(request_execution);
    let topology_denied_artifact = topology_region_conflict_inspection_artifact(request_execution);
    let deletion_denied =
        inspect_merge_conflicts(&inspection_declaration, &lowered, &deletion_denied_artifact)
            .expect("denied deletion inspection should succeed");
    let topology_denied =
        inspect_merge_conflicts(&inspection_declaration, &lowered, &topology_denied_artifact)
            .expect("denied topology inspection should succeed");
    let deletion_row = deletion_denied
        .rows()
        .iter()
        .find(|row| row.merge_class() == "deletion:deleted_vs_modified")
        .expect("deleted-vs-modified row should be present");
    let topology_row = topology_denied
        .rows()
        .iter()
        .find(|row| row.merge_class() == "topology_region_conflict")
        .expect("topology-region row should be present");

    assert_eq!(
        deletion_row.merge_class_admission(),
        &MergeClassAdmission::ExecutionDenied
    );
    assert_eq!(deletion_row.merge_class(), "deletion:deleted_vs_modified");
    assert_eq!(
        topology_row.merge_class_admission(),
        &MergeClassAdmission::ExecutionDenied
    );
    assert_eq!(topology_row.merge_class(), "topology_region_conflict");
    assert_eq!(
        deletion_denied
            .counters()
            .workflow_conflict_inspection_count(),
        1
    );
    assert_ne!(
        deletion_row.conflict_scope_digest(),
        topology_row.conflict_scope_digest()
    );
}

#[test]
fn post_merge_inspection_wraps_authoritative_merge_outcome_artifact() {
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
    let outcome = shape_merge_authority_outcome(&lowered);
    let inspection_declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::PostMergeInspectionNarrow,
            WorkflowAuthorityTargetFamily::QueryInspection,
            WorkflowCostClass::InspectionNarrow,
            WorkflowBudgetClass::InspectionBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("post-merge inspection declaration should admit");
    let inspection = inspect_post_merge_outcome(&inspection_declaration, &outcome)
        .expect("post-merge inspection should succeed");
    assert_eq!(inspection.rows().len(), 1);
    assert_eq!(
        inspection.rows()[0].authority_target_family(),
        &WorkflowAuthorityTargetFamily::RelationalMerge
    );
    assert_eq!(
        inspection.counters().workflow_post_merge_inspection_count(),
        1
    );
    assert_eq!(
        inspection.counters().workflow_conflict_inspection_count(),
        0
    );
    assert_eq!(
        inspection.origin_digest(),
        inspection_declaration.report().declaration_digest()
    );
}

#[test]
fn post_merge_inspection_denies_non_authoritative_mutation_outcomes() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let binding = bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
        .expect("runtime binding should admit");
    let declaration = admit_query_workflow_declaration(
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
    let lowered = lower_mutation_intent_declaration(
        &declaration,
        authority_binding_identity,
        MutationLoweringInput::IntentReconciliation {
            entity_id: EntityId::new(PartitionId(1), 41, 0),
            desired_aspect_fields:
                crate::aspect_field_authoring::single_native_string_aspect_field_patch(
                    "name", "name", "after",
                )
                .expect("name patch should lower"),
        },
    )
    .expect("mutation lowering should succeed");
    let outcome = shape_mutation_authority_outcome(&lowered);
    let inspection_declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::PostMergeInspectionNarrow,
            WorkflowAuthorityTargetFamily::QueryInspection,
            WorkflowCostClass::InspectionNarrow,
            WorkflowBudgetClass::InspectionBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("post-merge inspection declaration should admit");

    let error = inspect_post_merge_outcome(&inspection_declaration, &outcome)
        .expect_err("mutation-only outcome should not support post-merge inspection");
    assert_eq!(
        error.failure_class(),
        &WorkflowInspectionFailureClass::NonAuthoritativeOutcomeForbidden
    );
}
