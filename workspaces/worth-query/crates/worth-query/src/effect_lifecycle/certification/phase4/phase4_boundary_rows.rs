use worth_relational::facade::history::BranchId;

use crate::basis_lifecycle::BasisFamily;
use crate::effect_lifecycle::{
    admit_effect_intent, evaluate_effect_eligibility, normalize_raw_effect_intent,
    scope_admitted_effect_plan, EffectAuthoringBasis, EffectEligibilityOutcome,
    EffectExecutionAuthority, EffectFamily, RawEffectIntent,
};
use crate::workflow::{
    MutationLoweringInput, WorkflowAuthorityTargetFamily, WorkflowDeclarationFamily,
    WritebackLoweringInput,
};

use super::super::scenarios::{
    branch_mutation_basis, preview_closeout_basis, preview_workflow_binding,
    raw_mutation_effect_with_binding, runtime_workflow_binding,
    runtime_workflow_binding_for_branch, workflow_request,
};
use super::super::support::{
    branch_snapshot_identity, create_entity, relational_runtime_with_intent_strategy,
    test_bridge_with_writeback_authority,
};
use super::super::EffectLifecycleSeededCertificationBundle;
use super::{
    EffectLifecyclePhase4CertificationRow, EffectLifecyclePhase4LaneKind,
    EffectLifecyclePhase4LaneOutcome,
};

pub(super) fn batch_lane_denial_row() -> EffectLifecyclePhase4CertificationRow {
    let basis = EffectAuthoringBasis::from(branch_mutation_basis("branch-a"));
    let denial = crate::effect_lifecycle::effect_batch()
        .using_basis(basis)
        .push(RawEffectIntent::Mutation {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::MutationLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMutation,
            ),
            input: MutationLoweringInput::IntentReconciliation {
                entity_id: worth_relational::facade::identity::EntityId::new(
                    worth_relational::facade::identity::PartitionId(1),
                    99,
                    0,
                ),
                desired_aspect_fields:
                    crate::aspect_field_authoring::single_native_string_aspect_field_patch(
                        "name",
                        "name",
                        "mixed-authority",
                    )
                    .expect("name patch should be native"),
            },
        })
        .push(RawEffectIntent::Writeback {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        })
        .admit()
        .expect_err("mixed authority batch should deny");
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::BatchLaneDenial,
        EffectLifecyclePhase4LaneOutcome::Denied,
        BasisFamily::CurrentHead,
        EffectFamily::Mutation,
        format!("{:?}", denial.denial_kind()),
        denial.message().to_string(),
        denial.counters().clone(),
    )
}

pub(super) fn preview_rebind_row() -> EffectLifecyclePhase4CertificationRow {
    let basis = EffectAuthoringBasis::from(preview_closeout_basis("phase4-preview"));
    let raw = RawEffectIntent::Mutation {
        binding: preview_workflow_binding("phase4-preview"),
        request: workflow_request(
            WorkflowDeclarationFamily::MutationLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMutation,
        ),
        input: MutationLoweringInput::IntentReconciliation {
            entity_id: worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId(1),
                77,
                0,
            ),
            desired_aspect_fields:
                crate::aspect_field_authoring::single_native_string_aspect_field_patch(
                    "name",
                    "name",
                    "preview-rebind",
                )
                .expect("name patch should be native"),
        },
    };
    let normalized = normalize_raw_effect_intent(&basis, raw).expect("preview mutation normalizes");
    let rebind = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::RebindRequired(rebind) => rebind,
        other => panic!("expected preview rebind, got {other:?}"),
    };
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::PreviewRebind,
        EffectLifecyclePhase4LaneOutcome::RebindRequired,
        basis.family(),
        EffectFamily::Mutation,
        format!("{:?}", rebind.denial_kind()),
        rebind.decision_trace().message().to_string(),
        rebind.counters().clone(),
    )
}

pub(super) fn deferred_replay_row() -> EffectLifecyclePhase4CertificationRow {
    let basis = EffectAuthoringBasis::store_backed("phase4-store-basis");
    let raw = RawEffectIntent::Writeback {
        binding: runtime_workflow_binding(),
        request: workflow_request(
            WorkflowDeclarationFamily::WritebackLoweringNarrow,
            WorkflowAuthorityTargetFamily::BridgeWriteback,
        ),
        input: WritebackLoweringInput::projected_state_diff(),
    };
    let normalized =
        normalize_raw_effect_intent(&basis, raw).expect("deferred writeback normalizes");
    let deferred = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Deferred(deferred) => deferred,
        other => panic!("expected deferred writeback, got {other:?}"),
    };
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::DeferredReplay,
        EffectLifecyclePhase4LaneOutcome::Deferred,
        basis.family(),
        EffectFamily::Writeback,
        format!("{:?}", deferred.denial_kind()),
        deferred.decision_trace().message().to_string(),
        deferred.counters().clone(),
    )
}

pub(super) fn host_override_denial_row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> EffectLifecyclePhase4CertificationRow {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(
        execution,
        &mut runtime,
        "before",
        BranchId("main".to_string()),
    );
    let bridge = test_bridge_with_writeback_authority();
    let basis = EffectAuthoringBasis::from(branch_mutation_basis("branch-a"));
    let raw = raw_mutation_effect_with_binding(
        runtime_workflow_binding(),
        entity_id,
        "host-override".to_string(),
    );
    let normalized = normalize_raw_effect_intent(&basis, raw).expect("mutation should normalize");
    let admitted = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
        other => panic!("expected admitted mutation, got {other:?}"),
    };
    let denial = scope_admitted_effect_plan(admitted)
        .lower()
        .expect("mutation should lower")
        .execute_with(execution, EffectExecutionAuthority::bridge(&bridge))
        .expect_err("bridge host override should deny");
    let denial = denial.denial().expect("bridge override is a denial");
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::HostOverrideDenial,
        EffectLifecyclePhase4LaneOutcome::Denied,
        basis.family(),
        EffectFamily::Mutation,
        denial.denial_for_reporting().to_string(),
        denial.message().to_string(),
        denial.counters().clone(),
    )
}

pub(super) fn stale_after_admission_row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> EffectLifecyclePhase4CertificationRow {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(
        execution,
        &mut runtime,
        "before",
        BranchId("main".to_string()),
    );
    crate::runtime::fork_branch_from_exact_source(
        execution,
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should exist");
    let basis = EffectAuthoringBasis::from(branch_mutation_basis("branch-a"));
    let raw = raw_mutation_effect_with_binding(
        runtime_workflow_binding_for_branch(
            branch_snapshot_identity(&runtime, "branch-a"),
            "branch-a",
        ),
        entity_id,
        "stale-after-admission".to_string(),
    );
    let normalized = normalize_raw_effect_intent(&basis, raw).expect("mutation should normalize");
    let admitted = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
        other => panic!("expected admitted mutation, got {other:?}"),
    };
    create_entity(
        execution,
        &mut runtime,
        "intervening",
        BranchId("branch-a".to_string()),
    );
    let denial = scope_admitted_effect_plan(admitted)
        .lower()
        .expect("mutation should still lower after admission")
        .execute_with(
            execution,
            EffectExecutionAuthority::relational(&mut runtime),
        )
        .expect_err("changed truth after admission should deny at execution");
    let denial = denial.denial().expect("stale admission is a denial");
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::StaleAfterAdmission,
        EffectLifecyclePhase4LaneOutcome::Denied,
        basis.family(),
        EffectFamily::Mutation,
        denial.denial_for_reporting().to_string(),
        denial.message().to_string(),
        denial.counters().clone(),
    )
}

pub(super) fn stale_after_lowering_row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> EffectLifecyclePhase4CertificationRow {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(
        execution,
        &mut runtime,
        "before",
        BranchId("main".to_string()),
    );
    crate::runtime::fork_branch_from_exact_source(
        execution,
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should exist");
    let basis = EffectAuthoringBasis::from(branch_mutation_basis("branch-a"));
    let lowered = scope_admitted_effect_plan(
        match evaluate_effect_eligibility(
            normalize_raw_effect_intent(
                &basis,
                raw_mutation_effect_with_binding(
                    runtime_workflow_binding_for_branch(
                        branch_snapshot_identity(&runtime, "branch-a"),
                        "branch-a",
                    ),
                    entity_id,
                    "stale-after-lowering".to_string(),
                ),
            )
            .expect("mutation should normalize"),
        ) {
            EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
            other => panic!("expected admitted mutation, got {other:?}"),
        },
    )
    .lower()
    .expect("mutation should lower");
    create_entity(
        execution,
        &mut runtime,
        "intervening",
        BranchId("branch-a".to_string()),
    );
    let denial = lowered
        .execute_with(
            execution,
            EffectExecutionAuthority::relational(&mut runtime),
        )
        .expect_err("retained lowered artifact should stale-deny");
    let denial = denial.denial().expect("stale lowering is a denial");
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::StaleAfterLowering,
        EffectLifecyclePhase4LaneOutcome::Denied,
        basis.family(),
        EffectFamily::Mutation,
        denial.denial_for_reporting().to_string(),
        denial.message().to_string(),
        denial.counters().clone(),
    )
}

pub(super) fn seeded_replay_row(
    seeded: &EffectLifecycleSeededCertificationBundle,
) -> EffectLifecyclePhase4CertificationRow {
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::SeededReplay,
        EffectLifecyclePhase4LaneOutcome::Certified,
        BasisFamily::CurrentHead,
        EffectFamily::Mutation,
        seeded.seed_replay_digest().to_string(),
        seeded.certification_bundle_digest().to_string(),
        seeded.rows()[0].counters().clone(),
    )
}
