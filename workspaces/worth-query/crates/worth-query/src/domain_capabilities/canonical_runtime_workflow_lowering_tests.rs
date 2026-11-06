use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use super::test_support::{
    admitted_plan_target_parts, declaration_target, ready, ready_payload, success,
};
use super::{
    materialize_lowered_merge_workflow_declaration,
    materialize_lowered_mutation_intent_declaration, materialize_query_writeback_lowering,
    WorthQueryDomainCapabilityProgressionDenialKind, WorthQueryWorkflowContributionAuthoring,
    WorthQueryWorkflowContributionPayload, WorthQueryWorkflowContributionPosture,
    WorthQueryWorkflowLoweringSemantics, WorthQueryWorkflowRuntimeBindingSemantics,
    WorthQueryWorkflowRuntimeSemantics,
};
use crate::target_binding::WorthQueryBindingTargetWitness;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_runtime_bridge::facade::BridgeRequestKind;

#[test]
fn mutation_lowering_materializer_builds_runtime_lowered_declaration(
) -> Result<(), IdentityDerivationDenial> {
    let authority_binding_identity = crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::WorkflowMutationLowering,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("test_authority_binding"),
        "mutation-lowering",
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("binding"),
        "authority-binding:42",
    )
    .seal();
    let lowered = success(materialize_lowered_mutation_intent_declaration(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_mutation_reconciliation(
            "spatial.workflow.mutation",
            "runtime preflight should lower a relational mutation intent",
            crate::memory_workspace::admit_external_snapshot_label("runtime-snapshot:42"),
            authority_binding_identity.clone(),
            EntityId::new(PartitionId(1), 41, 0),
            crate::aspect_field_authoring::single_native_string_aspect_field_patch(
                "name", "name", "after",
            )
            .expect("name patch should build"),
        )?
        .bind_to_declaration_target(declaration_target("intent-workflow-mutation")),
    )));

    assert_eq!(
        lowered.strategy_request().strategy_name().as_str(),
        "strategy.intent.reconcile"
    );
    assert_eq!(
        lowered.authority_binding().binding_digest(),
        authority_binding_identity.as_str()
    );
    assert_eq!(
        lowered
            .authority_binding()
            .runtime_snapshot_identity()
            .map(|identity| identity.evidence_identity()),
        Some(
            crate::memory_workspace::admit_external_snapshot_label("runtime-snapshot:42",)
                .evidence_identity()
        )
    );
    assert_eq!(lowered.counters().workflow_mutation_lowering_count(), 1);

    Ok(())
}

#[test]
fn merge_lowering_materializer_builds_preview_bound_lowering(
) -> Result<(), IdentityDerivationDenial> {
    let lowered = success(materialize_lowered_merge_workflow_declaration(ready(
        WorthQueryWorkflowContributionAuthoring::promotion_eligible_merge_reconciliation(
            "spatial.workflow.merge",
            "preview promotion should lower a merge workflow declaration",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:77",
            ),
            BranchId("main".to_string()),
            BranchId("candidate".to_string()),
        )?
        .bind_to_admitted_plan_target(admitted_plan_target_parts(
            "plan-workflow-merge",
            "request-merge",
            "eligibility-merge",
            "decision-merge",
        )?),
    )));

    assert_eq!(
        lowered.merge_request().target_branch(),
        &BranchId("main".to_string())
    );
    assert_eq!(
        lowered.merge_request().source_branch(),
        &BranchId("candidate".to_string())
    );
    assert_eq!(
        lowered.freshness_binding().as_str(),
        "preview_session_bound"
    );
    assert_eq!(
        lowered.staleness_class().as_str(),
        "authority_validation_required"
    );

    Ok(())
}

#[test]
fn writeback_lowering_materializer_builds_bridge_writeback_declaration(
) -> Result<(), IdentityDerivationDenial> {
    let lowered = success(materialize_query_writeback_lowering(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_writeback_projected_state_diff(
            "spatial.workflow.writeback",
            "runtime preflight should lower a bridge writeback declaration",
            crate::memory_workspace::admit_external_snapshot_label(
                "runtime-snapshot:58",
            ),
        )?
        .bind_to_declaration_target(declaration_target("intent-workflow-writeback")),
    )));

    assert_eq!(
        lowered.causality_binding().request_kind(),
        BridgeRequestKind::Authoritative
    );
    assert_eq!(
        lowered.bridge_declaration().request_kind(),
        BridgeRequestKind::Authoritative
    );
    assert_eq!(lowered.counters().workflow_writeback_declaration_count(), 1);

    Ok(())
}

#[test]
fn workflow_lowering_materializer_denies_missing_lowering_semantics(
) -> Result<(), IdentityDerivationDenial> {
    let denied = materialize_lowered_mutation_intent_declaration(ready(
        super::proof_integration::create_requested_domain_capability_contribution(
            declaration_target("intent-workflow-missing-lowering"),
            WorthQueryWorkflowContributionPayload::with_runtime_semantics(
                WorthQueryWorkflowContributionPosture::ConfirmationRequired,
                "spatial.workflow.mutation",
                "missing lowering semantics should deny",
                Some(WorthQueryWorkflowRuntimeSemantics::new(
                    WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_snapshot_identity(
                        crate::memory_workspace::admit_external_snapshot_label(
                            "runtime-snapshot:42",
                        ),
                    ),
                    crate::workflow::WorkflowDeclarationFamily::MutationLoweringNarrow,
                    crate::workflow::WorkflowAuthorityTargetFamily::RelationalMutation,
                    crate::workflow::WorkflowCostClass::MutationLoweringNarrow,
                    crate::workflow::WorkflowBudgetClass::AuthorityTargetBounded,
                    crate::workflow::WorkflowFreshnessPolicy::ExactBasis,
                )),
            )?,
        ),
    ));

    assert!(matches!(
        denied,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics
    ));

    Ok(())
}

#[test]
fn workflow_lowering_materializer_preserves_runtime_stale_posture(
) -> Result<(), IdentityDerivationDenial> {
    let target = declaration_target("intent-workflow-writeback-stale");
    let stale = materialize_query_writeback_lowering(ready_payload(
        target.clone(),
        WorthQueryWorkflowContributionPayload::with_runtime_and_lowering_semantics(
            WorthQueryWorkflowContributionPosture::PromotionEligible,
            "spatial.workflow.writeback.stale",
            "preview promotion writeback should preserve stale posture at lowering time",
            Some(WorthQueryWorkflowRuntimeSemantics::new(
                WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                    crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                        "preview-session:88",
                    ),
                    crate::workflow::WorkflowPreviewEvaluationClass::PromotionEligible,
                ),
                crate::workflow::WorkflowDeclarationFamily::WritebackLoweringNarrow,
                crate::workflow::WorkflowAuthorityTargetFamily::BridgeWriteback,
                crate::workflow::WorkflowCostClass::WritebackLoweringNarrow,
                crate::workflow::WorkflowBudgetClass::AuthorityTargetBounded,
                crate::workflow::WorkflowFreshnessPolicy::ExactBasis,
            )),
            Some(WorthQueryWorkflowLoweringSemantics::writeback(
                crate::workflow::WritebackLoweringInput::projected_state_diff(),
            )),
        )?,
    ));

    match stale {
        TransitionOutcome::Stale(stale) => {
            assert_eq!(stale.category(), "workflow-preview");
            assert_eq!(
                stale.bound_target_for_reporting(),
                target.target_identity().as_str()
            );
        }
        other => panic!("expected stale outcome, got {other:?}"),
    }

    Ok(())
}

#[test]
fn discard_required_writeback_lowering_preserves_preview_stale_posture(
) -> Result<(), IdentityDerivationDenial> {
    let target = declaration_target("intent-workflow-writeback-discard");
    let stale = materialize_query_writeback_lowering(ready(
        WorthQueryWorkflowContributionAuthoring::discard_required_writeback_projected_state_diff(
            "spatial.workflow.writeback.discard",
            "discard-required preview writeback should stay stale until authoritative revalidation",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:91",
            ),
        )?
        .bind_to_declaration_target(target.clone()),
    ));

    match stale {
        TransitionOutcome::Stale(stale) => {
            assert_eq!(stale.category(), "workflow-preview");
            assert_eq!(
                stale.bound_target_for_reporting(),
                target.target_identity().as_str()
            );
        }
        other => panic!("expected stale outcome, got {other:?}"),
    }

    Ok(())
}

#[test]
fn workflow_lowering_materializer_preserves_runtime_rebind_posture(
) -> Result<(), IdentityDerivationDenial> {
    let target = declaration_target("intent-workflow-writeback-rebind");
    let rebind = materialize_query_writeback_lowering(ready_payload(
        target.clone(),
        WorthQueryWorkflowContributionPayload::with_runtime_and_lowering_semantics(
            WorthQueryWorkflowContributionPosture::PromotionEligible,
            "spatial.workflow.writeback.rebind",
            "preview-scoped writeback should preserve explicit rebind posture",
            Some(WorthQueryWorkflowRuntimeSemantics::new(
                WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                    crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                        "preview-session:89",
                    ),
                    crate::workflow::WorkflowPreviewEvaluationClass::PromotionEligible,
                ),
                crate::workflow::WorkflowDeclarationFamily::WritebackLoweringNarrow,
                crate::workflow::WorkflowAuthorityTargetFamily::BridgeWriteback,
                crate::workflow::WorkflowCostClass::WritebackLoweringNarrow,
                crate::workflow::WorkflowBudgetClass::AuthorityTargetBounded,
                crate::workflow::WorkflowFreshnessPolicy::AllowExplicitRebind,
            )),
            Some(WorthQueryWorkflowLoweringSemantics::writeback(
                crate::workflow::WritebackLoweringInput::projected_state_diff(),
            )),
        )?,
    ));

    match rebind {
        TransitionOutcome::RebindRequired(rebind) => {
            assert_eq!(rebind.category(), "workflow-preview");
            assert_eq!(
                rebind.bound_target_for_reporting(),
                target.target_identity().as_str()
            );
        }
        other => panic!("expected rebind-required outcome, got {other:?}"),
    }

    Ok(())
}
