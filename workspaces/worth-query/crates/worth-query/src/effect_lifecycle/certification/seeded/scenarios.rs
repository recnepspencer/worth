#[path = "scenarios_executed.rs"]
mod scenarios_executed;
#[path = "scenarios_non_admitted.rs"]
mod scenarios_non_admitted;

use crate::basis_lifecycle::{
    admit_basis_capability, evaluate_basis_inspection_advisory_eligibility,
    evaluate_basis_mutation_preparation_eligibility, evaluate_basis_preview_closeout_eligibility,
    normalize_raw_basis_intent, scope_basis_for_mutation_preparation,
    scope_basis_for_preview_closeout, AdvisoryBasisEligibility, BasisFamily, BasisOperationLane,
    InspectionLaneWitness, MutationPreparationLaneWitness, PreviewCloseoutLaneWitness,
    RawBasisIntent, ScopedMutationPreparationBasis, ScopedPreviewCloseoutBasis,
};
use crate::effect_lifecycle::EffectLifecycleCounters;
use crate::effect_lifecycle::{EffectFamily, RawEffectIntent};
use crate::workflow::{
    synthetic_preview_workflow_binding, synthetic_runtime_workflow_binding_for_snapshot_identity,
    synthetic_runtime_workflow_binding_scoped_for_branch_snapshot_identity, MutationLoweringInput,
    WorkflowAuthorityTargetFamily, WorkflowBudgetClass, WorkflowContextBinding, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowDeclarationRequest, WorkflowFreshnessPolicy,
    WorkflowPreviewEvaluationClass,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_runtime_bridge::facade::{
    BridgePreviewSessionIdentity, RelationalBridgeSnapshotIdentityParts,
};

use crate::memory_workspace::WorthQuerySnapshotIdentity;

use super::{
    EffectLifecycleSeededCertificationRow, EffectLifecycleSeededOutcomeClass,
    EffectLifecycleSeededRowParts,
};

const TEMPLATE_COUNT: usize = 9;

pub(super) const fn minimum_seeded_scenario_count() -> usize {
    TEMPLATE_COUNT
}

pub(super) fn seeded_rows(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    seed: u64,
    scenario_count: usize,
) -> Vec<EffectLifecycleSeededCertificationRow> {
    let mut stepper = SeedStepper::new(seed);
    (0..scenario_count)
        .map(|index| {
            let template = ScenarioTemplate::from_index(((seed as usize) + index) % TEMPLATE_COUNT);
            materialize_row(execution, template, index, &mut stepper)
        })
        .collect()
}

fn materialize_row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    template: ScenarioTemplate,
    index: usize,
    stepper: &mut SeedStepper,
) -> EffectLifecycleSeededCertificationRow {
    match template {
        ScenarioTemplate::MutationExecuted => {
            scenarios_executed::scalar_mutation_row(execution, index, stepper)
        }
        ScenarioTemplate::WritebackExecuted => {
            scenarios_executed::scalar_writeback_row(execution, index, stepper)
        }
        ScenarioTemplate::MergeLowered => scenarios_executed::merge_lowered_row(index, stepper),
        ScenarioTemplate::BatchMutationExecuted => {
            scenarios_executed::batch_mutation_row(execution, index, stepper)
        }
        ScenarioTemplate::PreviewMutationRebind => {
            scenarios_non_admitted::preview_mutation_rebind_row(index, stepper)
        }
        ScenarioTemplate::PreviewDerivedAdvisory => {
            scenarios_non_admitted::preview_derived_advisory_row(index, stepper)
        }
        ScenarioTemplate::StoreBackedDeferred => {
            scenarios_non_admitted::store_backed_deferred_row(index, stepper)
        }
        ScenarioTemplate::DurableReloadDeferred => {
            scenarios_non_admitted::durable_reload_deferred_row(index, stepper)
        }
        ScenarioTemplate::TenantMergeDenied => {
            scenarios_non_admitted::tenant_merge_denied_row(index, stepper)
        }
    }
}

pub(super) fn branch_mutation_basis(branch_identity: &str) -> ScopedMutationPreparationBasis {
    let normalized = normalize_raw_basis_intent(
        RawBasisIntent::BranchHead {
            branch_identity: branch_identity.to_string(),
            accessible: true,
        },
        <MutationPreparationLaneWitness as BasisOperationLane>::lane_name(),
    )
    .expect("branch basis should normalize");
    let eligibility = evaluate_basis_mutation_preparation_eligibility(normalized)
        .expect("branch basis should admit");
    scope_basis_for_mutation_preparation(admit_basis_capability(eligibility))
}

pub(super) fn tenant_mutation_basis(tenant_identity: &str) -> ScopedMutationPreparationBasis {
    let normalized = normalize_raw_basis_intent(
        RawBasisIntent::TenantScoped {
            tenant_identity: tenant_identity.to_string(),
            branch_identity: "branch-a".to_string(),
            schema_identity: "schema-a".to_string(),
            tenant_schema_matches: true,
        },
        <MutationPreparationLaneWitness as BasisOperationLane>::lane_name(),
    )
    .expect("tenant basis should normalize");
    let eligibility = evaluate_basis_mutation_preparation_eligibility(normalized)
        .expect("tenant basis should admit");
    scope_basis_for_mutation_preparation(admit_basis_capability(eligibility))
}

pub(super) fn preview_closeout_basis(preview_identity: &str) -> ScopedPreviewCloseoutBasis {
    let normalized = normalize_raw_basis_intent(
        RawBasisIntent::Preview {
            preview_identity: preview_identity.to_string(),
            stale: false,
        },
        <PreviewCloseoutLaneWitness as BasisOperationLane>::lane_name(),
    )
    .expect("preview basis should normalize");
    let eligibility = evaluate_basis_preview_closeout_eligibility(normalized)
        .expect("preview closeout basis should admit");
    scope_basis_for_preview_closeout(admit_basis_capability(eligibility))
}

pub(super) fn preview_derived_inspection_advisory(
    preview_identity: &str,
    source_basis_identity: &str,
) -> AdvisoryBasisEligibility<InspectionLaneWitness> {
    let normalized = normalize_raw_basis_intent(
        RawBasisIntent::PreviewDerived {
            preview_identity: preview_identity.to_string(),
            source_basis_identity: source_basis_identity.to_string(),
        },
        <InspectionLaneWitness as BasisOperationLane>::lane_name(),
    )
    .expect("preview-derived basis should normalize");
    evaluate_basis_inspection_advisory_eligibility(normalized)
        .expect("preview-derived inspection should be advisory")
}

pub(super) fn runtime_workflow_binding() -> WorkflowContextBinding {
    runtime_workflow_binding_with_snapshot(WorthQuerySnapshotIdentity::from_relational_snapshot(
        RelationalBridgeSnapshotIdentityParts::new(1, 1),
    ))
}

pub(super) fn runtime_workflow_binding_with_snapshot(
    snapshot_identity: WorthQuerySnapshotIdentity,
) -> WorkflowContextBinding {
    synthetic_runtime_workflow_binding_for_snapshot_identity(
        "effect-lifecycle-seeded-runtime",
        snapshot_identity,
    )
}

pub(super) fn runtime_workflow_binding_for_branch(
    snapshot_identity: WorthQuerySnapshotIdentity,
    branch: &str,
) -> WorkflowContextBinding {
    synthetic_runtime_workflow_binding_scoped_for_branch_snapshot_identity(
        "effect-lifecycle-seeded-runtime",
        "effect-lifecycle-seeded-branch",
        snapshot_identity,
        BranchId(branch.to_string()),
    )
}

pub(super) fn preview_workflow_binding(label: &str) -> WorkflowContextBinding {
    synthetic_preview_workflow_binding(
        "effect-lifecycle-seeded-preview",
        BridgePreviewSessionIdentity::from_stable_name(label),
        WorkflowPreviewEvaluationClass::PromotionEligible,
    )
}

pub(super) fn workflow_request(
    family: WorkflowDeclarationFamily,
    authority_target_family: WorkflowAuthorityTargetFamily,
) -> WorkflowDeclarationRequest {
    let cost_class = match authority_target_family {
        WorkflowAuthorityTargetFamily::QueryInspection => WorkflowCostClass::InspectionNarrow,
        WorkflowAuthorityTargetFamily::RelationalMutation => {
            WorkflowCostClass::MutationLoweringNarrow
        }
        WorkflowAuthorityTargetFamily::RelationalMerge => WorkflowCostClass::MergeLoweringNarrow,
        WorkflowAuthorityTargetFamily::BridgeWriteback => {
            WorkflowCostClass::WritebackLoweringNarrow
        }
    };
    WorkflowDeclarationRequest::new(
        family,
        authority_target_family,
        cost_class,
        WorkflowBudgetClass::AuthorityTargetBounded,
        WorkflowFreshnessPolicy::ExactBasis,
    )
}

pub(super) fn raw_mutation_effect_with_binding(
    binding: WorkflowContextBinding,
    entity_id: EntityId,
    desired_name: String,
) -> RawEffectIntent {
    RawEffectIntent::Mutation {
        binding,
        request: workflow_request(
            WorkflowDeclarationFamily::MutationLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMutation,
        ),
        input: MutationLoweringInput::IntentReconciliation {
            entity_id,
            desired_aspect_fields:
                crate::aspect_field_authoring::single_native_string_aspect_field_patch(
                    "name",
                    "name",
                    desired_name,
                )
                .expect("name patch should be native"),
        },
    }
}

pub(super) fn seeded_label(prefix: &str, stepper: &mut SeedStepper, index: usize) -> String {
    format!("{prefix}-{index}-{}", stepper.next_u64())
}

pub(super) fn scalar_or_terminal_row(
    scenario_name: String,
    outcome_class: EffectLifecycleSeededOutcomeClass,
    basis_family: BasisFamily,
    effect_family: EffectFamily,
    batch_width: usize,
    support_discovery_digest: String,
    normalized_effect_intent_digest: Option<String>,
    effect_eligibility_digest: String,
    authority_scoped_effect_plan_digest: Option<String>,
    lowered_effect_execution_plan_digest: Option<String>,
    effect_execution_receipt_digest: Option<String>,
    failure_digest: Option<String>,
    counters: EffectLifecycleCounters,
) -> EffectLifecycleSeededCertificationRow {
    EffectLifecycleSeededCertificationRow::new(EffectLifecycleSeededRowParts {
        scenario_name,
        outcome_class,
        basis_family,
        effect_family,
        batch_width,
        support_discovery_digest,
        normalized_effect_intent_digest,
        effect_eligibility_digest,
        authority_scoped_effect_plan_digest,
        lowered_effect_execution_plan_digest,
        effect_execution_receipt_digest,
        failure_digest,
        counters,
    })
}

#[derive(Clone, Copy)]
enum ScenarioTemplate {
    MutationExecuted,
    WritebackExecuted,
    MergeLowered,
    BatchMutationExecuted,
    PreviewMutationRebind,
    PreviewDerivedAdvisory,
    StoreBackedDeferred,
    DurableReloadDeferred,
    TenantMergeDenied,
}

impl ScenarioTemplate {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::MutationExecuted,
            1 => Self::WritebackExecuted,
            2 => Self::MergeLowered,
            3 => Self::BatchMutationExecuted,
            4 => Self::PreviewMutationRebind,
            5 => Self::PreviewDerivedAdvisory,
            6 => Self::StoreBackedDeferred,
            7 => Self::DurableReloadDeferred,
            _ => Self::TenantMergeDenied,
        }
    }
}

pub(super) struct SeedStepper {
    state: u64,
}

impl SeedStepper {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    pub(super) fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    pub(super) fn next_index(&mut self, width: usize) -> usize {
        (self.next_u64() % width as u64) as usize
    }
}
