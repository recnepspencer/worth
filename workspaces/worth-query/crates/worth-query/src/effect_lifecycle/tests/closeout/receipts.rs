use worth_relational::facade::history::BranchId;
use worth_runtime_bridge::facade::BridgeWritebackOutcomeClass;

use crate::effect_lifecycle::{
    scope_admitted_effect_plan, EffectDiagnosticsRequest, EffectEnvelopePrimaryResult,
    EffectExecutionAuthority, EffectFamily, EffectPublicSurfaceAvailability,
    EffectPublicSurfaceKind, EffectReceiptArtifactKind, EffectReceiptTargetEvidence,
    EffectReceiptTransitionKind, EffectReceiptTransitionPosture,
};

use super::execution_support::{
    branch_snapshot_identity, create_entity, relational_runtime_with_intent_strategy,
    test_bridge_with_writeback_authority,
};
use super::support::{
    admitted_mutation_effect_for_entity_with_binding, admitted_tenant_writeback_effect,
    branch_mutation_basis, native_name_patch, raw_mutation_effect_with_binding,
    runtime_workflow_binding_for_branch,
};

#[test]
fn mutation_execution_mints_receipt_first_envelope_and_diagnostics() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should be created");
    let receipt = scope_admitted_effect_plan(admitted_mutation_effect_for_entity_with_binding(
        runtime_workflow_binding_for_branch(
            branch_snapshot_identity(&runtime, "branch-a"),
            "branch-a",
        ),
        entity_id,
        native_name_patch("receipt-first"),
    ))
    .lower()
    .expect("mutation should lower")
    .execute_receipt_with(EffectExecutionAuthority::relational(&mut runtime))
    .expect("mutation should execute to receipt");

    assert_eq!(
        receipt.receipt_family(),
        EffectReceiptArtifactKind::WorthQueryIntentExecution
    );
    assert_eq!(receipt.declared_effect_family(), EffectFamily::Mutation);
    assert_eq!(receipt.write_count(), 1);
    assert!(matches!(
        receipt.target_evidence(),
        EffectReceiptTargetEvidence::MutationCommit { .. }
    ));

    let envelope = receipt.effect_envelope();
    assert_eq!(
        envelope.primary_result(),
        EffectEnvelopePrimaryResult::MutationCommitted
    );
    assert_eq!(envelope.receipt_family(), receipt.receipt_family());
    assert_eq!(envelope.authority_owner(), receipt.authority_owner());
    assert_eq!(envelope.basis_lane(), receipt.basis_lane());
    assert_eq!(
        envelope.trace_for_reporting(),
        receipt.decision_trace().decision_trace_for_reporting()
    );
    assert_eq!(
        envelope.sources().receipt_for_reporting(),
        receipt.receipt_for_reporting()
    );
    assert_eq!(
        envelope.sources().lowered_for_reporting(),
        receipt.lowered_for_reporting()
    );
    assert_eq!(
        envelope.sources().authority_artifact_for_reporting(),
        receipt
            .integrity_markers()
            .authority_artifact_for_reporting()
    );
    assert_eq!(
        envelope.sources().counter_snapshot_for_reporting(),
        receipt.integrity_markers().counter_snapshot_for_reporting()
    );

    let transitions = receipt.transition_rules();
    assert_eq!(transitions.receipt_family(), receipt.receipt_family());
    assert!(transitions.rules().iter().any(|rule| {
        rule.kind() == EffectReceiptTransitionKind::InspectReceipt
            && rule.posture() == EffectReceiptTransitionPosture::Implemented
    }));
    assert!(transitions.rules().iter().any(|rule| {
        rule.kind() == EffectReceiptTransitionKind::ProjectMaterializedFacts
            && rule.posture() == EffectReceiptTransitionPosture::Deferred
    }));

    let diagnostics = receipt.materialize_diagnostics(EffectDiagnosticsRequest::forensic());
    assert_eq!(
        diagnostics.receipt_for_reporting(),
        receipt.receipt_for_reporting()
    );
    assert_eq!(
        diagnostics.envelope_for_reporting(),
        envelope.envelope_for_reporting()
    );
    assert!(diagnostics
        .detail_sections()
        .iter()
        .any(|section| section.starts_with("lowered:")));
    assert!(diagnostics
        .detail_sections()
        .iter()
        .any(|section| section.starts_with("transitions:")));
    assert!(diagnostics
        .detail_sections()
        .iter()
        .any(|section| section.starts_with("sources:")));
}

#[test]
fn receipt_only_mutations_release_each_bounded_published_snapshot() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("receipt-capacity".to_string()),
        &BranchId("main".to_string()),
    )
    .unwrap();
    let baseline = runtime
        .storage_access()
        .storage_stats()
        .published_snapshot_handle_count;
    assert_eq!(baseline, 0);

    for ordinal in 0..70 {
        let receipt = scope_admitted_effect_plan(admitted_mutation_effect_for_entity_with_binding(
            runtime_workflow_binding_for_branch(
                branch_snapshot_identity(&runtime, "receipt-capacity"),
                "receipt-capacity",
            ),
            entity_id,
            native_name_patch(format!("receipt-{ordinal}")),
        ))
        .lower()
        .unwrap()
        .execute_receipt_with(EffectExecutionAuthority::relational(&mut runtime))
        .unwrap();
        assert!(matches!(
            receipt.target_evidence(),
            EffectReceiptTargetEvidence::MutationCommit { .. }
        ));
    }
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats()
            .published_snapshot_handle_count,
        baseline
    );
}

#[test]
fn writeback_execution_mints_write_receipt_family() {
    let bridge = test_bridge_with_writeback_authority();
    let receipt = scope_admitted_effect_plan(admitted_tenant_writeback_effect())
        .lower()
        .expect("writeback should lower")
        .execute_receipt_with(EffectExecutionAuthority::bridge(&bridge))
        .expect("writeback should execute to receipt");

    assert_eq!(
        receipt.receipt_family(),
        EffectReceiptArtifactKind::WorthQueryWriteReceipt
    );
    assert_eq!(receipt.declared_effect_family(), EffectFamily::Writeback);
    match receipt.target_evidence() {
        EffectReceiptTargetEvidence::Writeback {
            outcome_identity,
            authority_receipt_identity,
            execution_receipt_identity,
        } => {
            assert!(!outcome_identity.as_str().is_empty());
            assert!(!authority_receipt_identity.as_str().is_empty());
            assert!(!execution_receipt_identity.as_str().is_empty());
        }
        other => panic!("expected writeback target evidence, got {other:?}"),
    }
    assert_eq!(
        receipt.effect_envelope().primary_result(),
        EffectEnvelopePrimaryResult::WritebackCommitted
    );
    assert!(receipt.effect_envelope().deferred_neighbors().len() >= 2);
    assert_eq!(
        receipt.effect_envelope().transition_rules_for_reporting(),
        receipt.transition_rules().rules_for_reporting()
    );
    let executed = scope_admitted_effect_plan(admitted_tenant_writeback_effect())
        .lower()
        .expect("writeback should lower")
        .execute_with(EffectExecutionAuthority::bridge(&bridge))
        .expect("writeback should still execute");
    let (_, truth_receipt) = executed
        .as_writeback()
        .expect("writeback artifact should exist");
    assert_eq!(
        truth_receipt.outcome_class(),
        BridgeWritebackOutcomeClass::AuthoritativeCommit
    );
}

#[test]
fn batch_execution_mints_batch_write_receipt_family() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let left = create_entity(&mut runtime, "left", BranchId("main".to_string()));
    let right = create_entity(&mut runtime, "right", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should be created");

    let receipt = crate::effect_lifecycle::effect_batch()
        .using_basis(crate::effect_lifecycle::EffectAuthoringBasis::from(
            branch_mutation_basis(),
        ))
        .push(raw_mutation_effect_with_binding(
            runtime_workflow_binding_for_branch(
                branch_snapshot_identity(&runtime, "branch-a"),
                "branch-a",
            ),
            left,
            native_name_patch("left-batch-receipt"),
        ))
        .push(raw_mutation_effect_with_binding(
            runtime_workflow_binding_for_branch(
                branch_snapshot_identity(&runtime, "branch-a"),
                "branch-a",
            ),
            right,
            native_name_patch("right-batch-receipt"),
        ))
        .admit()
        .expect("batch should admit")
        .lower()
        .expect("batch should lower")
        .execute_receipt_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("batch should execute to receipt");

    assert_eq!(
        receipt.receipt_family(),
        EffectReceiptArtifactKind::WorthQueryBatchWriteReceipt
    );
    assert_eq!(receipt.write_count(), 2);
    match receipt.target_evidence() {
        EffectReceiptTargetEvidence::BatchMutation {
            component_count, ..
        } => {
            assert_eq!(component_count, 2);
        }
        other => panic!("expected batch target evidence, got {other:?}"),
    }
    assert_eq!(
        receipt.effect_envelope().primary_result(),
        EffectEnvelopePrimaryResult::BatchMutationCommitted
    );
    assert_ne!(
        receipt.decision_trace().admitted_or_batch_for_reporting(),
        receipt.decision_trace().lowered_for_reporting()
    );
}

#[test]
fn receipt_only_batches_release_each_bounded_published_snapshot() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let left = create_entity(&mut runtime, "left", BranchId("main".to_string()));
    let right = create_entity(&mut runtime, "right", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("batch-receipt-capacity".to_string()),
        &BranchId("main".to_string()),
    )
    .unwrap();
    let baseline = runtime
        .storage_access()
        .storage_stats()
        .published_snapshot_handle_count;
    assert_eq!(baseline, 0);

    for ordinal in 0..70 {
        let branch = "batch-receipt-capacity";
        let receipt = crate::effect_lifecycle::effect_batch()
            .using_basis(crate::effect_lifecycle::EffectAuthoringBasis::from(
                branch_mutation_basis(),
            ))
            .push(raw_mutation_effect_with_binding(
                runtime_workflow_binding_for_branch(
                    branch_snapshot_identity(&runtime, branch),
                    branch,
                ),
                left,
                native_name_patch(format!("left-{ordinal}")),
            ))
            .push(raw_mutation_effect_with_binding(
                runtime_workflow_binding_for_branch(
                    branch_snapshot_identity(&runtime, branch),
                    branch,
                ),
                right,
                native_name_patch(format!("right-{ordinal}")),
            ))
            .admit()
            .unwrap()
            .lower()
            .unwrap()
            .execute_receipt_with(EffectExecutionAuthority::relational(&mut runtime))
            .unwrap();
        assert!(matches!(
            receipt.target_evidence(),
            EffectReceiptTargetEvidence::BatchMutation {
                component_count: 2,
                ..
            }
        ));
    }
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats()
            .published_snapshot_handle_count,
        baseline
    );
}

#[test]
fn public_surface_inventory_now_marks_diagnostics_envelope_as_implemented() {
    let inventory = crate::effect_lifecycle::effect_lifecycle_public_surface_inventory();
    let diagnostics = inventory
        .rows()
        .iter()
        .find(|row| row.surface_kind() == EffectPublicSurfaceKind::DiagnosticsEnvelope)
        .expect("diagnostics row should exist");

    assert_eq!(
        diagnostics.availability(),
        EffectPublicSurfaceAvailability::Implemented
    );
    assert!(diagnostics
        .entrypoint()
        .expect("diagnostics row should advertise entrypoint")
        .contains("effect_envelope"));
}
