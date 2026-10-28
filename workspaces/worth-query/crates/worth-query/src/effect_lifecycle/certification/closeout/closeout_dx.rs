use crate::basis_lifecycle::BasisFamily;
use crate::effect_lifecycle::{
    admit_effect_intent, discover_effect_lifecycle_support, effect_lifecycle_support_matrix,
    evaluate_effect_eligibility, normalize_raw_effect_intent, scope_admitted_effect_plan,
    EffectAuthoringBasis, EffectEligibilityOutcome, EffectExecutionCertificationLane,
    EffectExecutionCertificationRow, EffectFamily, EffectLifecycleCounters,
    EffectLifecyclePublicSurfaceInventory, EffectPublicSurfaceKind, RawEffectIntent,
};
use crate::identity::hash_parts;
use crate::workflow::{
    MutationLoweringInput, WorkflowAuthorityTargetFamily, WorkflowDeclarationFamily,
};
use worth_relational::facade::history::BranchId;

use super::closeout_receipts::{
    batch_receipt_surface, mutation_receipt_surface, writeback_receipt_surface,
};
use super::scenarios::{
    branch_mutation_basis, preview_closeout_basis, preview_workflow_binding,
    runtime_workflow_binding_for_branch, workflow_request,
};
use super::support::{
    branch_snapshot_identity, create_entity, fork_seed_branch_from_main,
    relational_runtime_with_intent_strategy,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DxStoryKind {
    CommonPathIntentAuthoring,
    WritebackCommonPath,
    InspectableLoweredPlan,
    DenialOrRebind,
    SupportDiscovery,
    BatchExecution,
    ReceiptFirstDiagnostics,
}

impl DxStoryKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::CommonPathIntentAuthoring => "common_path_intent_authoring",
            Self::WritebackCommonPath => "writeback_common_path",
            Self::InspectableLoweredPlan => "inspectable_lowered_plan",
            Self::DenialOrRebind => "denial_or_rebind",
            Self::SupportDiscovery => "support_discovery",
            Self::BatchExecution => "batch_execution",
            Self::ReceiptFirstDiagnostics => "receipt_first_diagnostics",
        }
    }
}

#[derive(Clone, Debug)]
struct DxStoryEvidence {
    kind: DxStoryKind,
    transcript: String,
    digest: String,
    counters: EffectLifecycleCounters,
}

impl DxStoryEvidence {
    fn new(
        kind: DxStoryKind,
        transcript: String,
        parts: Vec<String>,
        counters: EffectLifecycleCounters,
    ) -> Self {
        let digest = hash_parts(
            &std::iter::once(format!("story:{}", kind.as_str()))
                .chain(std::iter::once(format!("transcript:{transcript}")))
                .chain(std::iter::once(format!(
                    "counters:{}",
                    counters.counter_for_reporting()
                )))
                .chain(parts)
                .collect::<Vec<_>>(),
        );
        Self {
            kind,
            transcript,
            digest,
            counters,
        }
    }
}

pub(super) struct CloseoutDxEvidence {
    pub(super) row: EffectExecutionCertificationRow,
    pub(super) support_lookup_counters: EffectLifecycleCounters,
    pub(super) target_dx_digest: String,
    pub(super) golden_transcript_digest: String,
}

pub(super) fn build_closeout_dx(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    public_surface: &EffectLifecyclePublicSurfaceInventory,
) -> CloseoutDxEvidence {
    let mutation = mutation_receipt_surface(execution);
    let writeback = writeback_receipt_surface(execution);
    let batch = batch_receipt_surface(execution);
    let common_path = common_path_story(&mutation);
    let writeback_common_path = writeback_common_path_story(&writeback);
    let inspectable = inspectable_lowered_story(execution);
    let denial_or_rebind = denial_or_rebind_story();
    let support_discovery = support_discovery_story();
    let support_lookup_counters = support_discovery.counters.clone();
    let batch_execution = batch_execution_story(&batch);
    let receipt_first = receipt_first_story(&mutation);
    let stories = [
        common_path,
        writeback_common_path,
        inspectable,
        denial_or_rebind,
        support_discovery,
        batch_execution,
        receipt_first,
    ];

    let target_dx_digest = target_dx_digest(public_surface);
    let golden_transcript_digest = hash_parts(
        &stories
            .iter()
            .map(|story| format!("{}:{}", story.kind.as_str(), story.digest))
            .collect::<Vec<_>>(),
    );
    let row_evidence_digest = hash_parts(&[
        target_dx_digest.clone(),
        golden_transcript_digest.clone(),
        effect_lifecycle_support_matrix()
            .matrix_for_reporting()
            .to_string(),
    ]);
    let detail = stories
        .iter()
        .map(|story| format!("{}={}", story.kind.as_str(), story.transcript))
        .collect::<Vec<_>>()
        .join("|");
    let counters = stories
        .iter()
        .fold(EffectLifecycleCounters::default(), |aggregate, story| {
            aggregate.combine(&story.counters)
        });

    CloseoutDxEvidence {
        row: EffectExecutionCertificationRow::new(
            EffectExecutionCertificationLane::SupportAndDxSurface,
            row_evidence_digest,
            detail,
            &counters,
            None,
        ),
        support_lookup_counters,
        target_dx_digest,
        golden_transcript_digest,
    }
}

fn common_path_story(
    mutation: &super::closeout_receipts::ReceiptSurfaceEvidence,
) -> DxStoryEvidence {
    let transcript = format!(
        "basis()->effect(mutation)->using_basis(branch_head)->admit({})->lower({})->execute({})->effect_envelope({})",
        mutation.eligibility_digest,
        mutation.plan_digest,
        mutation.receipt_digest,
        mutation.envelope_digest
    );
    DxStoryEvidence::new(
        DxStoryKind::CommonPathIntentAuthoring,
        transcript,
        vec![
            mutation.raw_digest.clone(),
            mutation.normalized_digest.clone(),
            mutation.eligibility_digest.clone(),
            mutation.plan_digest.clone(),
            mutation.lowered_digest.clone(),
            mutation.receipt_digest.clone(),
            mutation.envelope_digest.clone(),
        ],
        mutation.counters.clone(),
    )
}

fn inspectable_lowered_story(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> DxStoryEvidence {
    let branch = "dx-lowered";
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity = create_entity(
        execution,
        &mut runtime,
        "before",
        BranchId("main".to_string()),
    );
    fork_seed_branch_from_main(execution, &mut runtime, branch);
    let basis = EffectAuthoringBasis::from(branch_mutation_basis(branch));
    let binding =
        runtime_workflow_binding_for_branch(branch_snapshot_identity(&runtime, branch), branch);
    let raw = RawEffectIntent::Mutation {
        binding,
        request: workflow_request(
            WorkflowDeclarationFamily::MutationLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMutation,
        ),
        input: MutationLoweringInput::IntentReconciliation {
            entity_id: entity,
            desired_aspect_fields:
                crate::aspect_field_authoring::single_native_string_aspect_field_patch(
                    "name",
                    "name",
                    "lowered-dx",
                )
                .expect("name patch should be native"),
        },
    };
    let normalized = normalize_raw_effect_intent(&basis, raw).expect("dx lowered story normalizes");
    let admitted = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
        other => panic!("expected admitted inspectable story, got {other:?}"),
    };
    let lowered = scope_admitted_effect_plan(admitted)
        .lower()
        .expect("dx lowered story lowers");
    let transcript = format!(
        "family:{} authority_lane:{} basis_lane:{} strategy:{} scope:{} artifact:{}",
        lowered.family().as_str(),
        lowered.authority_lane().as_str(),
        lowered.basis_lane().as_str(),
        lowered.strategy_identity_target().as_str(),
        lowered.invariant_scope().as_str(),
        lowered.artifact_policy().as_str(),
    );
    DxStoryEvidence::new(
        DxStoryKind::InspectableLoweredPlan,
        transcript,
        vec![
            lowered
                .lowered_effect_execution_plan_for_reporting()
                .to_string(),
            lowered.authority_owner().as_str().to_string(),
            lowered.preview_posture().as_str().to_string(),
            lowered.policy_posture().as_str().to_string(),
            lowered.conflict_footprint().as_str().to_string(),
        ],
        lowered.counters().clone(),
    )
}

fn writeback_common_path_story(
    writeback: &super::closeout_receipts::ReceiptSurfaceEvidence,
) -> DxStoryEvidence {
    let transcript = format!(
        "basis()->effect(writeback)->using_basis(tenant_scoped)->admit({})->lower({})->execute({})->effect_envelope({})",
        writeback.eligibility_digest,
        writeback.plan_digest,
        writeback.receipt_digest,
        writeback.envelope_digest
    );
    DxStoryEvidence::new(
        DxStoryKind::WritebackCommonPath,
        transcript,
        vec![
            writeback.raw_digest.clone(),
            writeback.normalized_digest.clone(),
            writeback.eligibility_digest.clone(),
            writeback.plan_digest.clone(),
            writeback.lowered_digest.clone(),
            writeback.receipt_digest.clone(),
            writeback.envelope_digest.clone(),
            writeback.authority_artifact_digest.clone(),
        ],
        writeback.counters.clone(),
    )
}

fn denial_or_rebind_story() -> DxStoryEvidence {
    let basis = EffectAuthoringBasis::from(preview_closeout_basis("dx-preview"));
    let raw = RawEffectIntent::Mutation {
        binding: preview_workflow_binding("dx-preview"),
        request: workflow_request(
            WorkflowDeclarationFamily::MutationLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMutation,
        ),
        input: MutationLoweringInput::IntentReconciliation {
            entity_id: worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId(1),
                55,
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
    let normalized = normalize_raw_effect_intent(&basis, raw).expect("dx rebind story normalizes");
    let rebind = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::RebindRequired(rebind) => rebind,
        other => panic!("expected rebind story, got {other:?}"),
    };
    DxStoryEvidence::new(
        DxStoryKind::DenialOrRebind,
        format!(
            "kind:{:?} cause:{} message:{}",
            rebind.denial_kind(),
            rebind.decision_trace().cause(),
            rebind.decision_trace().message()
        ),
        vec![
            rebind.normalized().normalized_digest().to_string(),
            rebind.decision_trace().trace_for_reporting().to_string(),
        ],
        rebind.counters().clone(),
    )
}

fn support_discovery_story() -> DxStoryEvidence {
    let admitted =
        discover_effect_lifecycle_support(BasisFamily::BranchHead, EffectFamily::Mutation);
    let rebind = discover_effect_lifecycle_support(BasisFamily::Preview, EffectFamily::Mutation);
    let deferred =
        discover_effect_lifecycle_support(BasisFamily::StoreBacked, EffectFamily::Writeback);
    DxStoryEvidence::new(
        DxStoryKind::SupportDiscovery,
        format!(
            "admitted:{} rebind:{} deferred:{}",
            admitted.posture().as_str(),
            rebind.posture().as_str(),
            deferred.posture().as_str()
        ),
        vec![
            admitted.discovery_for_reporting().to_string(),
            rebind.discovery_for_reporting().to_string(),
            deferred.discovery_for_reporting().to_string(),
        ],
        admitted
            .counters()
            .combine(rebind.counters())
            .combine(deferred.counters()),
    )
}

fn batch_execution_story(
    batch: &super::closeout_receipts::ReceiptSurfaceEvidence,
) -> DxStoryEvidence {
    DxStoryEvidence::new(
        DxStoryKind::BatchExecution,
        "effect_batch().using_basis(...).admit().lower().execute_receipt_with(...)".to_string(),
        vec![
            batch.eligibility_digest.clone(),
            batch.plan_digest.clone(),
            batch.lowered_digest.clone(),
            batch.receipt_digest.clone(),
        ],
        batch.counters.clone(),
    )
}

fn receipt_first_story(
    mutation: &super::closeout_receipts::ReceiptSurfaceEvidence,
) -> DxStoryEvidence {
    DxStoryEvidence::new(
        DxStoryKind::ReceiptFirstDiagnostics,
        "receipt.transition_rules()->receipt.effect_envelope()->receipt.materialize_diagnostics(...)"
            .to_string(),
        vec![
            mutation.receipt_digest.clone(),
            mutation.transition_digest.clone(),
            mutation.envelope_digest.clone(),
            mutation.diagnostics_digest.clone(),
        ],
        mutation.counters.clone(),
    )
}

#[path = "closeout_dx/target_surface_digest.rs"]
mod target_surface_digest;
use target_surface_digest::target_dx_digest;
