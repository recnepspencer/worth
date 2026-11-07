use crate::aspect_field_authoring::single_native_string_aspect_field_patch;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::harness::fixtures::execution_preflights;
use crate::workflow::{
    admit_query_workflow_declaration, bind_workflow_context, lower_merge_workflow_declaration,
    lower_mutation_intent_declaration, lower_query_writeback_declaration, MergeLoweringInput,
    MutationLoweringInput, WorkflowAuthorityTargetFamily, WorkflowBindingSource,
    WorkflowBudgetClass, WorkflowCostClass, WorkflowDeclarationFamily, WorkflowDeclarationRequest,
    WorkflowFreshnessPolicy, WritebackLoweringInput,
};
use worth_relational::facade::commit_strategies::{
    IntentReconciliationInput, StrategyCallerProvenance, StrategyRequestOrigin,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::merge::{MergeExecutionRequest, MergeIntent};
use worth_runtime_bridge::facade::{
    BridgeRequestKind, BridgeWritebackDeclaration, BridgeWritebackDeclarationIdentity,
    BridgeWritebackEffectClass, BridgeWritebackFamilyKind, BridgeWritebackIdempotenceClass,
    BridgeWritebackRequestMode, BridgeWritebackStrategyClass,
};

#[test]
fn workflow_certification_mutation_lowering_matches_direct_relational_control() {
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
            desired_aspect_fields: single_native_string_aspect_field_patch("name", "name", "after")
                .expect("name patch should lower"),
        },
    )
    .expect("mutation lowering should succeed");

    let control = IntentReconciliationInput {
        entity_id: EntityId::new(PartitionId(1), 41, 0),
        desired_aspect_fields: single_native_string_aspect_field_patch("name", "name", "after")
            .expect("control field patch"),
    }
    .into_native_canonical_request(StrategyCallerProvenance {
        request_origin: StrategyRequestOrigin::Api,
        actor_identity: Some("worth-query".to_string()),
        correlation_id: Some(declaration.report().declaration_digest().to_string()),
    })
    .expect("control request should encode");

    assert_eq!(
        lowered.strategy_request().strategy_name(),
        control.strategy_name()
    );
    assert_eq!(
        lowered.strategy_request().input_bytes(),
        control.input_bytes()
    );
    assert_eq!(
        lowered.strategy_request().caller_provenance(),
        control.caller_provenance()
    );
}

#[test]
fn workflow_certification_merge_lowering_matches_direct_relational_control() {
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

    let control = MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("candidate".to_string()),
        MergeIntent::ReconcileIntoTarget,
    );
    assert_eq!(lowered.merge_request(), &control);
}

#[test]
fn workflow_certification_writeback_lowering_matches_direct_bridge_control() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let binding = bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
        .expect("runtime binding should admit");
    let declaration = admit_query_workflow_declaration(
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
    let lowered = lower_query_writeback_declaration(
        &declaration,
        WritebackLoweringInput::projected_state_diff(),
    )
    .expect("writeback lowering should succeed");

    assert_eq!(
        lowered.bridge_declaration().request_kind(),
        BridgeRequestKind::Authoritative,
    );
    assert_eq!(
        lowered.bridge_declaration().request_mode(),
        BridgeWritebackRequestMode::WritebackCapable,
    );
    assert_eq!(
        lowered.bridge_declaration().family_kind(),
        Some(BridgeWritebackFamilyKind::ProjectedStateDiff),
    );
    assert_eq!(
        lowered.bridge_declaration().effect_class(),
        BridgeWritebackEffectClass::ProjectedStateDiff,
    );
    assert_eq!(
        lowered.bridge_declaration().strategy_class(),
        Some(BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation),
    );
    assert_eq!(
        lowered.bridge_declaration().idempotence_class(),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    assert_eq!(
        lowered.bridge_declaration().digest(),
        BridgeWritebackDeclaration::writeback_capable(
            BridgeWritebackDeclarationIdentity::from_bridge_evidence(
                &WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::WorkflowMutationLowering,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "workflow_writeback_bridge_declaration_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("declaration"),
                    declaration.report().declaration_identity(),
                )
                .seal()
                .expect("fixture digest must fit")
                .bridge_external_identity_evidence(),
            ),
            BridgeRequestKind::Authoritative,
            BridgeWritebackFamilyKind::ProjectedStateDiff,
            BridgeWritebackEffectClass::ProjectedStateDiff,
            BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
            BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
        )
        .digest(),
    );
}
