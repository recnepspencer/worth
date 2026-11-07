use crate::basis_lifecycle::BasisFamily;
use crate::workflow::{
    MutationLoweringInput, WorkflowAuthorityTargetFamily, WorkflowDeclarationFamily,
    WorkflowFreshnessPolicy,
};
use worth_relational::facade::identity::{EntityId, PartitionId};

use crate::effect_lifecycle::{
    discover_effect_lifecycle_support, effect_lifecycle_support_matrix,
    evaluate_effect_eligibility, normalize_raw_effect_intent, DeniedEffectEligibilityKind,
    EffectAuthoringBasis, EffectDeferredNeighborFamily, EffectEligibilityOutcome, EffectFamily,
    EffectLoweredArtifactKind, EffectReceiptArtifactKind, EffectSupportCause, EffectSupportPosture,
    RawEffectIntent,
};

use super::support::{
    native_name_patch, preview_derived_inspection_advisory, runtime_workflow_binding,
    workflow_request,
};

#[test]
fn support_matrix_includes_admitted_rebind_and_deferred_rows() {
    let matrix = effect_lifecycle_support_matrix().expect("fixture digest must fit");

    assert!(matrix.rows().iter().any(|row| {
        row.basis_family() == BasisFamily::CurrentHead
            && row.effect_family() == EffectFamily::Mutation
            && row.posture() == EffectSupportPosture::Admitted
    }));
    assert!(matrix.rows().iter().any(|row| {
        row.basis_family() == BasisFamily::Preview
            && row.effect_family() == EffectFamily::Mutation
            && row.posture() == EffectSupportPosture::RebindRequired
    }));
    assert!(matrix.rows().iter().any(|row| {
        row.basis_family() == BasisFamily::Preview
            && row.effect_family() == EffectFamily::Writeback
            && row.posture() == EffectSupportPosture::RebindRequired
    }));
    assert!(matrix.rows().iter().any(|row| {
        row.basis_family() == BasisFamily::StoreBacked
            && row.effect_family() == EffectFamily::Writeback
            && row.posture() == EffectSupportPosture::Deferred
            && row.cause() == EffectSupportCause::StoreBackedExecutionDeferred
    }));
    assert!(matrix.rows().iter().any(|row| {
        row.basis_family() == BasisFamily::DurableReload
            && row.effect_family() == EffectFamily::Writeback
            && row.posture() == EffectSupportPosture::Deferred
            && row.cause() == EffectSupportCause::DurableReplayDeferred
    }));
}

#[test]
fn support_discovery_and_eligibility_agree_for_deferred_neighbors() {
    let store_backed =
        discover_effect_lifecycle_support(BasisFamily::StoreBacked, EffectFamily::Writeback)
            .expect("fixture digest must fit");
    assert_eq!(store_backed.posture(), EffectSupportPosture::Deferred);
    assert_eq!(
        store_backed.cause(),
        EffectSupportCause::StoreBackedExecutionDeferred
    );

    let durable =
        discover_effect_lifecycle_support(BasisFamily::DurableReload, EffectFamily::Writeback)
            .expect("fixture digest must fit");
    assert_eq!(durable.posture(), EffectSupportPosture::Deferred);
    assert_eq!(durable.cause(), EffectSupportCause::DurableReplayDeferred);
}

#[test]
fn support_discovery_exposes_caller_contract_for_runtime_writeback() {
    let support =
        discover_effect_lifecycle_support(BasisFamily::CurrentHead, EffectFamily::Writeback)
            .expect("fixture digest must fit");

    assert_eq!(support.posture(), EffectSupportPosture::Admitted);
    assert_eq!(
        support.authority_owner(),
        Some(crate::effect_lifecycle::EffectAuthorityOwner::WorthRuntimeBridge)
    );
    assert_eq!(
        support.supported_lowering(),
        Some(EffectLoweredArtifactKind::QueryWritebackDeclaration)
    );
    assert_eq!(
        support.receipt_family(),
        Some(EffectReceiptArtifactKind::WorthQueryWriteReceipt)
    );
    assert!(!support.requires_rebind());
    assert_eq!(support.denial_kinds(), &[]);
    assert_eq!(
        support.deferred_neighbors(),
        &[
            EffectDeferredNeighborFamily::StoreBackedExecutionParity,
            EffectDeferredNeighborFamily::DurableReplayAndRestartStableEnvelope,
        ]
    );
}

#[test]
fn support_discovery_exposes_denial_and_rebind_expectations() {
    let denied = discover_effect_lifecycle_support(BasisFamily::TenantScoped, EffectFamily::Merge)
        .expect("fixture digest must fit");
    assert_eq!(denied.posture(), EffectSupportPosture::Denied);
    assert_eq!(
        denied.supported_lowering(),
        Some(EffectLoweredArtifactKind::LoweredMergeWorkflowDeclaration)
    );
    assert_eq!(
        denied.receipt_family(),
        Some(EffectReceiptArtifactKind::WorthQueryIntentExecution)
    );
    assert_eq!(
        denied.denial_kinds(),
        &[DeniedEffectEligibilityKind::BranchAuthorityRequired]
    );

    let rebind = discover_effect_lifecycle_support(BasisFamily::Preview, EffectFamily::Mutation)
        .expect("fixture digest must fit");
    assert_eq!(rebind.posture(), EffectSupportPosture::RebindRequired);
    assert!(rebind.requires_rebind());
    assert_eq!(
        rebind.denial_kinds(),
        &[DeniedEffectEligibilityKind::PreviewRebindRequired]
    );
    assert_eq!(
        rebind.supported_lowering(),
        Some(EffectLoweredArtifactKind::LoweredMutationIntentDeclaration)
    );
}

#[test]
fn unsupported_support_discovery_fails_closed_without_artifacts() {
    let support = discover_effect_lifecycle_support(BasisFamily::Preview, EffectFamily::Merge)
        .expect("fixture digest must fit");

    assert_eq!(support.posture(), EffectSupportPosture::Unsupported);
    assert_eq!(support.authority_owner(), None);
    assert_eq!(support.supported_lowering(), None);
    assert_eq!(support.receipt_family(), None);
    assert!(!support.requires_rebind());
    assert_eq!(
        support.denial_kinds(),
        &[DeniedEffectEligibilityKind::UnsupportedForBasisFamily]
    );
    assert_eq!(support.deferred_neighbors(), &[]);
}

#[test]
fn preview_derived_mutation_returns_real_advisory_effect_posture() {
    let normalized = normalize_raw_effect_intent(
        &EffectAuthoringBasis::from(preview_derived_inspection_advisory()),
        RawEffectIntent::Mutation {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::MutationLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMutation,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: MutationLoweringInput::IntentReconciliation {
                entity_id: EntityId::new(PartitionId(1), 41, 0),
                desired_aspect_fields: native_name_patch("advisory-preview-derived"),
            },
        },
    )
    .expect("preview-derived advisory mutation should normalize");

    let discovery =
        discover_effect_lifecycle_support(BasisFamily::PreviewDerived, EffectFamily::Mutation)
            .expect("fixture digest must fit");
    let advisory = match evaluate_effect_eligibility(normalized).expect("fixture digest must fit") {
        EffectEligibilityOutcome::Advisory(advisory) => advisory,
        other => panic!("expected advisory effect, got {other:?}"),
    };

    assert_eq!(discovery.posture(), EffectSupportPosture::Advisory);
    assert_eq!(discovery.cause(), EffectSupportCause::AdvisoryOnlyExecution);
    assert_eq!(
        advisory.advisory_cause(),
        EffectSupportCause::AdvisoryOnlyExecution
    );
    assert_eq!(
        advisory.normalized().basis_family(),
        BasisFamily::PreviewDerived
    );
    assert_eq!(advisory.normalized().family(), EffectFamily::Mutation);
    assert_eq!(advisory.counters().advisory_count(), 1);
    assert_eq!(advisory.decision_trace().cause(), "advisory_only_execution");
    assert_ne!(
        advisory.normalized().capability_digest(),
        advisory.decision_trace().trace_for_reporting()
    );
}
