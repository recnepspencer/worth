use crate::effect_lifecycle::{
    scope_admitted_effect_plan, EffectAuthorityLane, EffectAuthorityOwner, EffectConflictFootprint,
    EffectFamily, EffectInvariantScope, EffectLoweringDenialKind, EffectPermittedLoweringFamily,
    EffectPolicyPosture, EffectPreviewPosture, EffectStrategyIdentityTarget,
    LoweredEffectExecutionArtifact,
};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::workflow::WorkflowStalenessClass;

use super::support::{
    admitted_branch_merge_effect, admitted_invalid_merge_effect, admitted_mutation_effect,
    admitted_tenant_writeback_effect,
};

#[test]
fn admitted_mutation_plan_lowers_into_one_executable_effect_artifact() {
    let lowered = scope_admitted_effect_plan(admitted_mutation_effect())
        .lower()
        .expect("mutation plan should lower");

    assert!(matches!(
        lowered.artifact(),
        LoweredEffectExecutionArtifact::Mutation(_)
    ));
    assert!(lowered.as_mutation().is_some());
    assert!(lowered.as_merge().is_none());
    assert!(lowered.as_writeback().is_none());
    assert_eq!(
        lowered.authority_scoped_plan().authority_owner(),
        EffectAuthorityOwner::WorthRelational
    );
    assert_eq!(lowered.family(), EffectFamily::Mutation);
    assert_eq!(lowered.authority_lane(), EffectAuthorityLane::Relational);
    assert_eq!(
        lowered.authority_owner(),
        EffectAuthorityOwner::WorthRelational
    );
    assert_eq!(
        lowered.invariant_scope(),
        EffectInvariantScope::EntityScopedMutation
    );
    assert_eq!(
        lowered.preview_posture(),
        EffectPreviewPosture::NotPreviewBound
    );
    assert_eq!(lowered.policy_posture(), EffectPolicyPosture::Unmasked);
    assert_eq!(
        lowered.permitted_lowering_family(),
        EffectPermittedLoweringFamily::MutationIntentDeclaration
    );
    assert_eq!(
        lowered.conflict_footprint(),
        EffectConflictFootprint::EntityMutation
    );
    assert_eq!(
        lowered.strategy_identity_target(),
        EffectStrategyIdentityTarget::NativeStrategyCommitRequest
    );
    assert_eq!(lowered.counters().lowered_effect_count(), 1);
    assert!(!lowered
        .lowered_effect_execution_plan_for_reporting()
        .is_empty());

    let mutation = lowered
        .as_mutation()
        .expect("mutation lowered artifact should be present");
    assert_eq!(mutation.counters().workflow_mutation_lowering_count(), 1);
    assert_eq!(
        lowered.counters().effect_lowering_width(),
        mutation.counters().workflow_lowering_width()
    );
    assert_eq!(
        lowered.counters().effect_executor_rediscovery_count(),
        mutation.counters().workflow_executor_rediscovery_count()
    );
    assert_eq!(lowered.staleness_class(), mutation.staleness_class());
    assert_eq!(
        mutation
            .strategy_request()
            .caller_provenance()
            .actor_identity
            .as_deref(),
        Some("worth-query")
    );
}

#[test]
fn admitted_merge_plan_lowers_into_one_executable_effect_artifact() {
    let lowered = scope_admitted_effect_plan(admitted_branch_merge_effect())
        .lower()
        .expect("merge plan should lower");

    assert!(matches!(
        lowered.artifact(),
        LoweredEffectExecutionArtifact::Merge(_)
    ));
    assert!(lowered.as_mutation().is_none());
    assert!(lowered.as_merge().is_some());

    let merge = lowered
        .as_merge()
        .expect("merge lowered artifact should be present");
    assert_eq!(merge.counters().workflow_merge_lowering_count(), 1);
    assert_eq!(
        lowered.counters().effect_lowering_width(),
        merge.counters().workflow_lowering_width()
    );
    assert_eq!(
        lowered.counters().effect_executor_rediscovery_count(),
        merge.counters().workflow_executor_rediscovery_count()
    );
    assert_eq!(lowered.staleness_class(), merge.staleness_class());
    assert_eq!(
        merge.staleness_class(),
        &WorkflowStalenessClass::AuthorityValidationRequired
    );
}

#[test]
fn admitted_writeback_plan_lowers_into_one_executable_effect_artifact() {
    let lowered = scope_admitted_effect_plan(admitted_tenant_writeback_effect())
        .lower()
        .expect("writeback plan should lower");

    assert!(matches!(
        lowered.artifact(),
        LoweredEffectExecutionArtifact::Writeback(_)
    ));
    assert!(lowered.as_writeback().is_some());

    let writeback = lowered
        .as_writeback()
        .expect("writeback lowered artifact should be present");
    assert_eq!(
        writeback.counters().workflow_writeback_declaration_count(),
        1
    );
    let binding = lowered
        .authority_scoped_plan()
        .admitted()
        .workflow_declaration()
        .binding();
    let expected_basis_identity = WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority,
    )
    .field_shape(WorthQueryEvidenceTag::new("role"), "writeback_basis")
    .field_shape(
        WorthQueryEvidenceTag::new("basis_family"),
        binding.basis_family().as_str(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("basis"),
        binding.basis_identity(),
    )
    .seal();
    assert_eq!(
        writeback.causality_binding().basis_identity(),
        &expected_basis_identity
    );
    assert_eq!(
        lowered.counters().effect_lowering_width(),
        writeback.counters().workflow_lowering_width()
    );
    assert_eq!(
        lowered.counters().effect_executor_rediscovery_count(),
        writeback.counters().workflow_executor_rediscovery_count()
    );
    assert_eq!(lowered.staleness_class(), writeback.staleness_class());
}

#[test]
fn lowering_denials_stay_typed_for_admitted_but_invalid_merge_pairs() {
    let denial = scope_admitted_effect_plan(admitted_invalid_merge_effect())
        .lower()
        .expect_err("invalid merge pair should deny during lowering");

    assert_eq!(
        denial.denial_kind(),
        EffectLoweringDenialKind::InvalidMergeBranchPairing
    );
    assert_eq!(
        denial.staleness_class(),
        &WorkflowStalenessClass::ExactBasisPreserved
    );
    assert_eq!(denial.counters().lowering_denied_count(), 1);
    assert_eq!(denial.counters().effect_lowering_width(), 1);
    assert!(!denial
        .denial_for_reporting()
        .expect("fixture digest must fit")
        .is_empty());
}
