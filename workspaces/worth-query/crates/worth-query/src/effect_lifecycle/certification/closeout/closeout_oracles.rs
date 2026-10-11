use crate::effect_lifecycle::{
    admit_effect_intent, effect_batch, evaluate_effect_eligibility, normalize_raw_effect_intent,
    scope_admitted_effect_plan, EffectAuthoringBasis, EffectEligibilityOutcome,
    EffectExecutionAuthority, RawEffectIntent,
};
use crate::workflow::{
    WorkflowAuthorityTargetFamily, WorkflowDeclarationFamily, WritebackLoweringInput,
};
use worth_relational::facade::history::BranchId;

use super::scenarios::{
    branch_mutation_basis, raw_mutation_effect_with_binding, runtime_workflow_binding,
    runtime_workflow_binding_for_branch, tenant_mutation_basis, workflow_request,
};
use super::support::{
    branch_snapshot_identity, create_entity, relational_runtime_with_intent_strategy,
    test_bridge_with_writeback_authority,
};

pub(super) struct CloseoutOracleEvidence {
    relational_oracle_digest: String,
    bridge_oracle_digest: String,
}

impl CloseoutOracleEvidence {
    pub(super) fn new(relational_oracle_digest: String, bridge_oracle_digest: String) -> Self {
        Self {
            relational_oracle_digest,
            bridge_oracle_digest,
        }
    }

    pub(super) fn relational_oracle_digest(&self) -> &str {
        &self.relational_oracle_digest
    }

    pub(super) fn bridge_oracle_digest(&self) -> &str {
        &self.bridge_oracle_digest
    }
}

pub(super) fn build_closeout_oracles(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> CloseoutOracleEvidence {
    CloseoutOracleEvidence::new(
        relational_oracle_digest(execution).to_string(),
        bridge_oracle_digest(execution).to_string(),
    )
}

fn relational_oracle_digest(execution: &worth_execution::ExecutionRequest<'_, '_>) -> String {
    let mutation_digest = scalar_mutation_oracle_digest(execution);
    let batch_digest = batch_mutation_oracle_digest(execution);
    crate::identity::hash_parts(&[
        "effect_closeout_relational_oracles_v1".to_string(),
        format!("scalar:{mutation_digest}"),
        format!("batch:{batch_digest}"),
    ])
}

fn scalar_mutation_oracle_digest(execution: &worth_execution::ExecutionRequest<'_, '_>) -> String {
    let branch = "closeout-oracle-branch";
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(
        execution,
        &mut runtime,
        "before",
        BranchId("main".to_string()),
    );
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId(branch.to_string()),
        &BranchId("main".to_string()),
    )
    .expect("closeout oracle branch should be created");
    let raw = raw_mutation_effect_with_binding(
        runtime_workflow_binding_for_branch(branch_snapshot_identity(&runtime, branch), branch),
        entity_id,
        "closeout-oracle".to_string(),
    );
    let normalized = normalize_raw_effect_intent(
        &EffectAuthoringBasis::from(branch_mutation_basis(branch)),
        raw,
    )
    .expect("closeout oracle mutation should normalize");
    let admitted = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
        other => panic!("expected admitted closeout oracle mutation, got {other:?}"),
    };
    let lowered = scope_admitted_effect_plan(admitted)
        .lower()
        .expect("closeout oracle mutation should lower");

    lowered
        .execute_with(
            execution,
            EffectExecutionAuthority::relational(&mut runtime),
        )
        .expect("closeout oracle mutation should execute")
        .verify_against_relational_runtime(&runtime)
        .expect("closeout oracle mutation should verify")
        .relational_oracle_for_reporting()
        .expect("relational oracle digest should exist")
        .to_string()
}

fn batch_mutation_oracle_digest(execution: &worth_execution::ExecutionRequest<'_, '_>) -> String {
    let branch = "closeout-oracle-batch";
    let mut runtime = relational_runtime_with_intent_strategy();
    let left = create_entity(
        execution,
        &mut runtime,
        "left",
        BranchId("main".to_string()),
    );
    let right = create_entity(
        execution,
        &mut runtime,
        "right",
        BranchId("main".to_string()),
    );
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId(branch.to_string()),
        &BranchId("main".to_string()),
    )
    .expect("closeout oracle batch branch should be created");
    let binding =
        runtime_workflow_binding_for_branch(branch_snapshot_identity(&runtime, branch), branch);
    let executed = effect_batch()
        .using_basis(EffectAuthoringBasis::from(branch_mutation_basis(branch)))
        .push(raw_mutation_effect_with_binding(
            binding.clone(),
            left,
            "left-closeout-oracle".to_string(),
        ))
        .push(raw_mutation_effect_with_binding(
            binding,
            right,
            "right-closeout-oracle".to_string(),
        ))
        .admit()
        .expect("closeout oracle batch should admit")
        .lower()
        .expect("closeout oracle batch should lower")
        .execute_with(
            execution,
            EffectExecutionAuthority::relational(&mut runtime),
        )
        .expect("closeout oracle batch should execute");

    executed
        .verify_against_relational_runtime(&runtime)
        .expect("closeout oracle batch should verify")
        .relational_oracle_for_reporting()
        .expect("relational batch oracle digest should exist")
        .to_string()
}

fn bridge_oracle_digest(execution: &worth_execution::ExecutionRequest<'_, '_>) -> String {
    let bridge = test_bridge_with_writeback_authority();
    let basis = EffectAuthoringBasis::from(tenant_mutation_basis("tenant-a"));
    let raw = RawEffectIntent::Writeback {
        binding: runtime_workflow_binding(),
        request: workflow_request(
            WorkflowDeclarationFamily::WritebackLoweringNarrow,
            WorkflowAuthorityTargetFamily::BridgeWriteback,
        ),
        input: WritebackLoweringInput::projected_state_diff(),
    };
    let normalized = normalize_raw_effect_intent(&basis, raw)
        .expect("closeout bridge oracle writeback normalizes");
    let admitted = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
        other => panic!("expected admitted closeout bridge writeback, got {other:?}"),
    };
    let executed = scope_admitted_effect_plan(admitted)
        .lower()
        .expect("closeout bridge oracle writeback should lower")
        .execute_with(execution, EffectExecutionAuthority::bridge(&bridge))
        .expect("closeout bridge oracle writeback should execute");

    executed
        .verify_against_bridge_runtime(&bridge)
        .expect("closeout bridge oracle should verify")
        .bridge_oracle_for_reporting()
        .expect("bridge oracle digest should exist")
        .to_string()
}
