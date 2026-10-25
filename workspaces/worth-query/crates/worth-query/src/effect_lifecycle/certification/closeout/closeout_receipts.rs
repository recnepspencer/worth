use crate::effect_lifecycle::{
    admit_effect_intent, effect_batch, evaluate_effect_eligibility, normalize_raw_effect_intent,
    scope_admitted_effect_plan, EffectAuthoringBasis, EffectDiagnosticsRequest,
    EffectEligibilityOutcome, EffectExecutionAuthority, EffectLifecycleCounters, RawEffectIntent,
};
use crate::identity::hash_parts;
use crate::workflow::{
    WorkflowAuthorityTargetFamily, WorkflowDeclarationFamily, WritebackLoweringInput,
};
use worth_relational::facade::history::BranchId;

use super::closeout_artifacts::{
    EffectExecutionCertificationLane, EffectExecutionCertificationRow,
};
use super::scenarios;
use super::support::{
    branch_snapshot_identity, create_entity, fork_seed_branch_from_main,
    relational_runtime_with_intent_strategy, test_bridge_with_writeback_authority,
};

#[derive(Clone)]
pub(super) struct ReceiptSurfaceEvidence {
    pub(super) raw_digest: String,
    pub(super) query_digest: String,
    pub(super) family_digest: String,
    pub(super) authority_digest: String,
    pub(super) basis_digest: String,
    pub(super) scope_digest: String,
    pub(super) policy_digest: String,
    pub(super) strategy_digest: String,
    pub(super) normalized_digest: String,
    pub(super) eligibility_digest: String,
    pub(super) plan_digest: String,
    pub(super) lowered_digest: String,
    pub(super) receipt_digest: String,
    pub(super) envelope_digest: String,
    pub(super) diagnostics_digest: String,
    pub(super) transition_digest: String,
    pub(super) authority_artifact_digest: String,
    pub(super) decision_trace_digest: String,
    pub(super) structural_delta_digest: String,
    pub(super) integrity_digest: String,
    pub(super) normalization_counter_digest: String,
    pub(super) eligibility_counter_digest: String,
    pub(super) lowering_counter_digest: String,
    pub(super) execution_counter_digest: String,
    pub(super) envelope_counter_digest: String,
    pub(super) counters: EffectLifecycleCounters,
}

impl ReceiptSurfaceEvidence {
    pub(super) fn row_evidence_digest(&self) -> String {
        hash_parts(&[
            self.receipt_digest.clone(),
            self.envelope_digest.clone(),
            self.diagnostics_digest.clone(),
            self.transition_digest.clone(),
        ])
    }

    pub(super) fn as_row(
        &self,
        lane: EffectExecutionCertificationLane,
    ) -> EffectExecutionCertificationRow {
        EffectExecutionCertificationRow::new(
            lane,
            self.row_evidence_digest(),
            format!(
                "receipt:{}|envelope:{}|diagnostics:{}|transitions:{}",
                self.receipt_digest,
                self.envelope_digest,
                self.diagnostics_digest,
                self.transition_digest
            ),
            &self.counters,
            None,
        )
    }
}

pub(super) fn mutation_receipt_surface(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> ReceiptSurfaceEvidence {
    let branch = "cert-phase6-branch";
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(
        execution,
        &mut runtime,
        "before",
        BranchId("main".to_string()),
    );
    fork_seed_branch_from_main(execution, &mut runtime, branch);
    let basis = EffectAuthoringBasis::from(scenarios::branch_mutation_basis(branch));
    let raw = scenarios::raw_mutation_effect_with_binding(
        scenarios::runtime_workflow_binding_for_branch(
            branch_snapshot_identity(&runtime, branch),
            branch,
        ),
        entity_id,
        "after".to_string(),
    );
    scalar_surface_evidence(
        execution,
        &basis,
        raw,
        EffectExecutionAuthority::relational(&mut runtime),
    )
}

pub(super) fn writeback_receipt_surface(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> ReceiptSurfaceEvidence {
    let basis = EffectAuthoringBasis::from(scenarios::tenant_mutation_basis("tenant-a"));
    let raw = RawEffectIntent::Writeback {
        binding: scenarios::runtime_workflow_binding(),
        request: scenarios::workflow_request(
            WorkflowDeclarationFamily::WritebackLoweringNarrow,
            WorkflowAuthorityTargetFamily::BridgeWriteback,
        ),
        input: WritebackLoweringInput::projected_state_diff(),
    };
    let bridge = test_bridge_with_writeback_authority();
    scalar_surface_evidence(
        execution,
        &basis,
        raw,
        EffectExecutionAuthority::bridge(&bridge),
    )
}

pub(super) fn batch_receipt_surface(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> ReceiptSurfaceEvidence {
    let branch = "cert-phase6-batch";
    let mut runtime = relational_runtime_with_intent_strategy();
    let main = BranchId("main".to_string());
    let left = create_entity(execution, &mut runtime, "left", main.clone());
    let right = create_entity(execution, &mut runtime, "right", main);
    fork_seed_branch_from_main(execution, &mut runtime, branch);
    let basis = EffectAuthoringBasis::from(scenarios::branch_mutation_basis(branch));
    let binding = scenarios::runtime_workflow_binding_for_branch(
        branch_snapshot_identity(&runtime, branch),
        branch,
    );
    let receipt = effect_batch()
        .using_basis(basis)
        .push(scenarios::raw_mutation_effect_with_binding(
            binding.clone(),
            left,
            "left-after".to_string(),
        ))
        .push(scenarios::raw_mutation_effect_with_binding(
            binding,
            right,
            "right-after".to_string(),
        ))
        .admit()
        .expect("closeout batch should admit")
        .lower()
        .expect("closeout batch should lower")
        .execute_receipt_with(
            execution,
            EffectExecutionAuthority::relational(&mut runtime),
        )
        .expect("closeout batch should execute");
    let envelope = receipt.effect_envelope();
    let diagnostics = receipt.materialize_diagnostics(EffectDiagnosticsRequest::forensic());
    ReceiptSurfaceEvidence {
        raw_digest: hash_parts(&["batch-raw".to_string()]),
        query_digest: hash_parts(&["batch-query".to_string()]),
        family_digest: hash_parts(&["family:mutation".to_string()]),
        authority_digest: hash_parts(&["authority:worth-relational".to_string()]),
        basis_digest: hash_parts(&[format!("basis:{branch}")]),
        scope_digest: hash_parts(&["scope:batch_mutation".to_string()]),
        policy_digest: hash_parts(&["policy:receipt_first".to_string()]),
        strategy_digest: hash_parts(&["strategy:native_strategy_commit_request".to_string()]),
        normalized_digest: hash_parts(&["batch-normalized".to_string()]),
        eligibility_digest: receipt
            .decision_trace()
            .admitted_or_batch_for_reporting()
            .to_string(),
        plan_digest: receipt
            .decision_trace()
            .admitted_or_batch_for_reporting()
            .to_string(),
        lowered_digest: receipt.lowered_for_reporting().to_string(),
        receipt_digest: receipt.receipt_for_reporting().to_string(),
        envelope_digest: envelope.envelope_for_reporting().to_string(),
        diagnostics_digest: diagnostics.diagnostics_for_reporting().to_string(),
        transition_digest: receipt.transition_rules().rules_for_reporting().to_string(),
        authority_artifact_digest: receipt
            .integrity_markers()
            .authority_artifact_for_reporting()
            .to_string(),
        decision_trace_digest: receipt
            .decision_trace()
            .decision_trace_for_reporting()
            .to_string(),
        structural_delta_digest: hash_parts(
            &envelope
                .structural_delta_identities()
                .iter()
                .map(|delta| delta.as_str().to_string())
                .collect::<Vec<_>>(),
        ),
        integrity_digest: receipt
            .integrity_markers()
            .integrity_for_reporting()
            .to_string(),
        normalization_counter_digest: hash_parts(&["batch-normalization-counters".to_string()]),
        eligibility_counter_digest: hash_parts(&["batch-eligibility-counters".to_string()]),
        lowering_counter_digest: receipt
            .delivery_counters()
            .counter_for_reporting()
            .to_string(),
        execution_counter_digest: receipt
            .delivery_counters()
            .counter_for_reporting()
            .to_string(),
        envelope_counter_digest: envelope
            .sources()
            .counter_snapshot_for_reporting()
            .to_string(),
        counters: receipt.delivery_counters().clone(),
    }
}

fn scalar_surface_evidence(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &EffectAuthoringBasis,
    raw: RawEffectIntent,
    authority: EffectExecutionAuthority<'_>,
) -> ReceiptSurfaceEvidence {
    let raw_digest = raw_effect_intent_digest(&raw);
    let normalized =
        normalize_raw_effect_intent(basis, raw).expect("closeout receipt surface should normalize");
    let normalization_counter_digest = normalized.counters().counter_for_reporting().to_string();
    let eligibility = match evaluate_effect_eligibility(normalized.clone()) {
        EffectEligibilityOutcome::Admitted(eligibility) => eligibility,
        other => panic!("expected admitted closeout surface, got {other:?}"),
    };
    let eligibility_digest = eligibility
        .decision_trace()
        .trace_for_reporting()
        .to_string();
    let eligibility_counter_digest = eligibility.counters().counter_for_reporting().to_string();
    let admitted = admit_effect_intent(eligibility);
    let scoped_plan = scope_admitted_effect_plan(admitted);
    let query_digest = normalized
        .workflow_binding()
        .query_for_reporting()
        .to_string();
    let family_digest = hash_parts(&[format!("family:{}", normalized.family().as_str())]);
    let authority_digest = hash_parts(&[
        format!("lane:{}", scoped_plan.authority_lane().as_str()),
        format!("owner:{}", scoped_plan.authority_owner().as_str()),
    ]);
    let basis_digest = hash_parts(&[
        format!("family:{}", normalized.basis_family().as_str()),
        format!("authority:{}", normalized.basis_authority().as_str()),
        format!("lifecycle:{}", normalized.basis_lifecycle().as_str()),
        format!("capability:{}", normalized.capability_digest()),
        format!("scoped:{}", normalized.scoped_basis_digest()),
    ]);
    let scope_digest = hash_parts(&[
        format!("invariant:{}", scoped_plan.invariant_scope().as_str()),
        format!("preview:{}", scoped_plan.preview_posture().as_str()),
        format!("footprint:{}", scoped_plan.conflict_footprint().as_str()),
    ]);
    let policy_digest = hash_parts(&[
        format!("policy:{}", scoped_plan.policy_posture().as_str()),
        format!("artifact:{}", scoped_plan.artifact_policy().as_str()),
        format!("freshness:{}", scoped_plan.freshness_policy().as_str()),
    ]);
    let strategy_digest = hash_parts(&[
        format!("target:{}", scoped_plan.strategy_identity_target().as_str()),
        format!(
            "lowering:{}",
            scoped_plan.permitted_lowering_family().as_str()
        ),
    ]);
    let plan_digest = scoped_plan.plan_for_reporting().to_string();
    let lowered = scoped_plan
        .lower()
        .expect("closeout receipt surface should lower");
    let lowering_counter_digest = lowered.counters().counter_for_reporting().to_string();
    let lowered_digest = lowered
        .lowered_effect_execution_plan_for_reporting()
        .to_string();
    let receipt = lowered
        .execute_receipt_with(execution, authority)
        .expect("closeout receipt surface should execute");
    let envelope = receipt.effect_envelope();
    let diagnostics = receipt.materialize_diagnostics(EffectDiagnosticsRequest::forensic());
    ReceiptSurfaceEvidence {
        raw_digest,
        query_digest,
        family_digest,
        authority_digest,
        basis_digest,
        scope_digest,
        policy_digest,
        strategy_digest,
        normalized_digest: normalized.normalized_digest().to_string(),
        eligibility_digest,
        plan_digest,
        lowered_digest,
        receipt_digest: receipt.receipt_for_reporting().to_string(),
        envelope_digest: envelope.envelope_for_reporting().to_string(),
        diagnostics_digest: diagnostics.diagnostics_for_reporting().to_string(),
        transition_digest: receipt.transition_rules().rules_for_reporting().to_string(),
        authority_artifact_digest: receipt
            .integrity_markers()
            .authority_artifact_for_reporting()
            .to_string(),
        decision_trace_digest: receipt
            .decision_trace()
            .decision_trace_for_reporting()
            .to_string(),
        structural_delta_digest: hash_parts(
            &envelope
                .structural_delta_identities()
                .iter()
                .map(|delta| delta.as_str().to_string())
                .collect::<Vec<_>>(),
        ),
        integrity_digest: receipt
            .integrity_markers()
            .integrity_for_reporting()
            .to_string(),
        normalization_counter_digest,
        eligibility_counter_digest,
        lowering_counter_digest,
        execution_counter_digest: receipt
            .delivery_counters()
            .counter_for_reporting()
            .to_string(),
        envelope_counter_digest: envelope
            .sources()
            .counter_snapshot_for_reporting()
            .to_string(),
        counters: receipt.delivery_counters().clone(),
    }
}

fn raw_effect_intent_digest(raw: &RawEffectIntent) -> String {
    match raw {
        RawEffectIntent::Mutation {
            binding,
            request,
            input,
        } => hash_parts(&[
            "raw_effect_intent_v1".to_string(),
            "family:mutation".to_string(),
            format!("binding:{}", binding.binding_digest()),
            format!("declaration:{}", request.declaration_family().as_str()),
            format!("target:{}", request.authority_target_family().as_str()),
            format!("input_family:{}", input.family().as_str()),
            format!("input:{input:?}"),
        ]),
        RawEffectIntent::Merge {
            binding,
            request,
            input,
        } => hash_parts(&[
            "raw_effect_intent_v1".to_string(),
            "family:merge".to_string(),
            format!("binding:{}", binding.binding_digest()),
            format!("declaration:{}", request.declaration_family().as_str()),
            format!("target:{}", request.authority_target_family().as_str()),
            format!("intent:{}", input.intent().as_str()),
            format!("target_branch:{:?}", input.target_branch()),
            format!("source_branch:{:?}", input.source_branch()),
        ]),
        RawEffectIntent::Writeback {
            binding,
            request,
            input,
        } => hash_parts(&[
            "raw_effect_intent_v1".to_string(),
            "family:writeback".to_string(),
            format!("binding:{}", binding.binding_digest()),
            format!("declaration:{}", request.declaration_family().as_str()),
            format!("target:{}", request.authority_target_family().as_str()),
            format!("input_family:{}", input.family().as_str()),
        ]),
    }
}
