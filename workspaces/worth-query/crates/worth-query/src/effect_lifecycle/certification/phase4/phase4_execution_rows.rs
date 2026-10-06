use worth_relational::facade::history::BranchId;

use crate::effect_lifecycle::{
    admit_effect_intent, bridge_observation_execution_record_subject_identity,
    bridge_observation_outcome_subject_identity, bridge_observation_receipt_subject_identity,
    bridge_observation_request_subject_identity, discover_effect_lifecycle_support, effect_batch,
    evaluate_effect_eligibility, normalize_raw_effect_intent, scope_admitted_effect_plan,
    BridgeExecutionOracle, EffectAuthoringBasis, EffectEligibilityOutcome,
    EffectExecutionAuthority, EffectFamily, RawEffectIntent, RelationalExecutionOracle,
};
use crate::workflow::{
    MergeLoweringInput, WorkflowAuthorityTargetFamily, WorkflowDeclarationFamily,
    WritebackLoweringInput,
};

use super::super::scenarios::{
    branch_mutation_basis, raw_mutation_effect_with_binding, runtime_workflow_binding,
    runtime_workflow_binding_for_branch, workflow_request,
};
use super::super::support::{
    branch_snapshot_identity, create_entity, relational_runtime_with_intent_strategy,
    test_bridge_with_writeback_authority,
};
use super::{
    EffectLifecyclePhase4CertificationRow, EffectLifecyclePhase4LaneKind,
    EffectLifecyclePhase4LaneOutcome,
};

pub(super) fn branch_mutation_execution_row() -> EffectLifecyclePhase4CertificationRow {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
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
        "phase4-branch-executed".to_string(),
    );
    let support = discover_effect_lifecycle_support(basis.family(), EffectFamily::Mutation);
    let normalized = normalize_raw_effect_intent(&basis, raw).expect("mutation should normalize");
    let admitted = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
        other => panic!("expected admitted mutation, got {other:?}"),
    };
    let executed = scope_admitted_effect_plan(admitted)
        .lower()
        .expect("mutation should lower")
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("mutation should execute");
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::BranchMutationExecution,
        EffectLifecyclePhase4LaneOutcome::Executed,
        basis.family(),
        EffectFamily::Mutation,
        executed.effect_execution_for_reporting().to_string(),
        format!(
            "support:{};plan:{}",
            support.discovery_for_reporting(),
            executed
                .lowered()
                .lowered_effect_execution_plan_for_reporting()
        ),
        executed.counters().clone(),
    )
}

pub(super) fn relational_merge_execution_row() -> EffectLifecyclePhase4CertificationRow {
    let mut runtime = relational_runtime_with_intent_strategy();
    create_entity(&mut runtime, "main", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("candidate".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("candidate branch should exist");
    create_entity(
        &mut runtime,
        "candidate-only",
        BranchId("candidate".to_string()),
    );
    let basis = EffectAuthoringBasis::from(branch_mutation_basis("main"));
    let raw = RawEffectIntent::Merge {
        binding: runtime_workflow_binding(),
        request: workflow_request(
            WorkflowDeclarationFamily::MergeLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMerge,
        ),
        input: MergeLoweringInput::reconcile_into_target(
            BranchId("main".to_string()),
            BranchId("candidate".to_string()),
        ),
    };
    let support = discover_effect_lifecycle_support(basis.family(), EffectFamily::Merge);
    let normalized = normalize_raw_effect_intent(&basis, raw).expect("merge should normalize");
    let admitted = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
        other => panic!("expected admitted merge, got {other:?}"),
    };
    let executed = scope_admitted_effect_plan(admitted)
        .lower()
        .expect("merge should lower")
        .execute_with(
            EffectExecutionAuthority::crate_relational_without_application_schema(&mut runtime),
        )
        .expect("merge should execute");
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::RelationalMergeExecution,
        EffectLifecyclePhase4LaneOutcome::Executed,
        basis.family(),
        EffectFamily::Merge,
        executed.effect_execution_for_reporting().to_string(),
        format!(
            "support:{};plan:{}",
            support.discovery_for_reporting(),
            executed
                .lowered()
                .lowered_effect_execution_plan_for_reporting()
        ),
        executed.counters().clone(),
    )
}

pub(super) fn bridge_writeback_execution_row() -> EffectLifecyclePhase4CertificationRow {
    let bridge = test_bridge_with_writeback_authority();
    let basis = EffectAuthoringBasis::from(branch_mutation_basis("branch-a"));
    let raw = RawEffectIntent::Writeback {
        binding: runtime_workflow_binding(),
        request: workflow_request(
            WorkflowDeclarationFamily::WritebackLoweringNarrow,
            WorkflowAuthorityTargetFamily::BridgeWriteback,
        ),
        input: WritebackLoweringInput::projected_state_diff(),
    };
    let support = discover_effect_lifecycle_support(basis.family(), EffectFamily::Writeback);
    let normalized = normalize_raw_effect_intent(&basis, raw).expect("writeback should normalize");
    let admitted = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
        other => panic!("expected admitted writeback, got {other:?}"),
    };
    let executed = scope_admitted_effect_plan(admitted)
        .lower()
        .expect("writeback should lower")
        .execute_with(EffectExecutionAuthority::bridge(&bridge))
        .expect("writeback should execute");
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::BridgeWritebackExecution,
        EffectLifecyclePhase4LaneOutcome::Executed,
        basis.family(),
        EffectFamily::Writeback,
        executed.effect_execution_for_reporting().to_string(),
        format!(
            "support:{};plan:{}",
            support.discovery_for_reporting(),
            executed
                .lowered()
                .lowered_effect_execution_plan_for_reporting()
        ),
        executed.counters().clone(),
    )
}

pub(super) fn batch_execution_row() -> EffectLifecyclePhase4CertificationRow {
    let mut runtime = relational_runtime_with_intent_strategy();
    let left = create_entity(&mut runtime, "left", BranchId("main".to_string()));
    let right = create_entity(&mut runtime, "right", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should exist");
    let basis = EffectAuthoringBasis::from(branch_mutation_basis("branch-a"));
    let binding = runtime_workflow_binding_for_branch(
        branch_snapshot_identity(&runtime, "branch-a"),
        "branch-a",
    );
    let support = discover_effect_lifecycle_support(basis.family(), EffectFamily::Mutation);
    let executed = effect_batch()
        .using_basis(basis.clone())
        .push(raw_mutation_effect_with_binding(
            binding.clone(),
            left,
            "left-phase4-batch".to_string(),
        ))
        .push(raw_mutation_effect_with_binding(
            binding,
            right,
            "right-phase4-batch".to_string(),
        ))
        .admit()
        .expect("batch should admit")
        .lower()
        .expect("batch should lower")
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("batch should execute");
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::BatchExecution,
        EffectLifecyclePhase4LaneOutcome::Executed,
        basis.family(),
        EffectFamily::Mutation,
        executed.batch_for_reporting().to_string(),
        format!(
            "support:{};width:{}",
            support.discovery_for_reporting(),
            executed.components().len()
        ),
        executed.counters().clone(),
    )
}

pub(super) fn relational_oracle_row() -> EffectLifecyclePhase4CertificationRow {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should exist");
    let basis = EffectAuthoringBasis::from(branch_mutation_basis("branch-a"));
    let executed = scope_admitted_effect_plan(
        match evaluate_effect_eligibility(
            normalize_raw_effect_intent(
                &basis,
                raw_mutation_effect_with_binding(
                    runtime_workflow_binding_for_branch(
                        branch_snapshot_identity(&runtime, "branch-a"),
                        "branch-a",
                    ),
                    entity_id,
                    "oracle-mutation".to_string(),
                ),
            )
            .expect("mutation should normalize"),
        ) {
            EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
            other => panic!("expected admitted mutation, got {other:?}"),
        },
    )
    .lower()
    .expect("mutation should lower")
    .execute_with(EffectExecutionAuthority::relational(&mut runtime))
    .expect("mutation should execute");
    let commit = executed
        .as_mutation()
        .expect("mutation artifact should exist");
    let oracle = RelationalExecutionOracle::new(
        "branch-a",
        commit.outcome().commit.commit_id.0,
        commit.outcome().commit.version_id.0,
        commit
            .outcome()
            .commit
            .parents
            .iter()
            .map(|id| id.0)
            .collect(),
    );
    let verification = executed
        .verify_against_relational_oracle(&oracle)
        .expect("oracle verification should succeed");
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::RelationalOracle,
        EffectLifecyclePhase4LaneOutcome::Verified,
        basis.family(),
        EffectFamily::Mutation,
        verification.verification_for_reporting().to_string(),
        "Mutation".to_string(),
        executed.counters().clone(),
    )
}

pub(super) fn bridge_oracle_row() -> EffectLifecyclePhase4CertificationRow {
    let bridge = test_bridge_with_writeback_authority();
    let basis = EffectAuthoringBasis::from(branch_mutation_basis("branch-a"));
    let executed = scope_admitted_effect_plan(
        match evaluate_effect_eligibility(
            normalize_raw_effect_intent(
                &basis,
                RawEffectIntent::Writeback {
                    binding: runtime_workflow_binding(),
                    request: workflow_request(
                        WorkflowDeclarationFamily::WritebackLoweringNarrow,
                        WorkflowAuthorityTargetFamily::BridgeWriteback,
                    ),
                    input: WritebackLoweringInput::projected_state_diff(),
                },
            )
            .expect("writeback should normalize"),
        ) {
            EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
            other => panic!("expected admitted writeback, got {other:?}"),
        },
    )
    .lower()
    .expect("writeback should lower")
    .execute_with(EffectExecutionAuthority::bridge(&bridge))
    .expect("writeback should execute");
    let (outcome, receipt) = executed
        .as_writeback()
        .expect("writeback artifact should exist");
    let oracle = BridgeExecutionOracle::new(
        bridge_observation_execution_record_subject_identity("bridge-record:phase4"),
        bridge_observation_outcome_subject_identity(outcome.digest()),
        outcome.outcome_class(),
        bridge_observation_request_subject_identity(receipt.request_digest()),
        bridge_observation_receipt_subject_identity(receipt.digest()),
    );
    let verification = executed
        .verify_against_bridge_oracle(&oracle)
        .expect("bridge oracle verification should succeed");
    EffectLifecyclePhase4CertificationRow::new(
        EffectLifecyclePhase4LaneKind::BridgeOracle,
        EffectLifecyclePhase4LaneOutcome::Verified,
        basis.family(),
        EffectFamily::Writeback,
        verification.verification_for_reporting().to_string(),
        "Writeback".to_string(),
        executed.counters().clone(),
    )
}
