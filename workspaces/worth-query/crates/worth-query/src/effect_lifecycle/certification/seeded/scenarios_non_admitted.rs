use crate::effect_lifecycle::{
    discover_effect_lifecycle_support, evaluate_effect_eligibility, normalize_raw_effect_intent,
    EffectAuthoringBasis, EffectEligibilityOutcome, EffectFamily, RawEffectIntent,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::workflow::{
    MergeLoweringInput, MutationLoweringInput, WorkflowAuthorityTargetFamily,
    WorkflowDeclarationFamily, WritebackLoweringInput,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};

use super::{
    preview_closeout_basis, preview_derived_inspection_advisory, preview_workflow_binding,
    raw_mutation_effect_with_binding, runtime_workflow_binding, scalar_or_terminal_row,
    seeded_label, tenant_mutation_basis, workflow_request, EffectLifecycleSeededCertificationRow,
    EffectLifecycleSeededOutcomeClass, SeedStepper,
};

pub(super) fn preview_mutation_rebind_row(
    index: usize,
    stepper: &mut SeedStepper,
) -> Result<EffectLifecycleSeededCertificationRow, IdentityDerivationDenial> {
    let preview = seeded_label("preview", stepper, index);
    let basis = EffectAuthoringBasis::from(preview_closeout_basis(&preview));
    let support = discover_effect_lifecycle_support(basis.family(), EffectFamily::Mutation)?;
    let normalized = normalize_raw_effect_intent(
        &basis,
        RawEffectIntent::Mutation {
            binding: preview_workflow_binding(&preview)?,
            request: workflow_request(
                WorkflowDeclarationFamily::MutationLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMutation,
            ),
            input: MutationLoweringInput::IntentReconciliation {
                entity_id: EntityId::new(PartitionId(1), 100 + index as u64, 0),
                desired_aspect_fields:
                    crate::aspect_field_authoring::single_native_string_aspect_field_patch(
                        "name",
                        "name",
                        seeded_label("preview-mutation", stepper, index),
                    )
                    .expect("name patch should be native"),
            },
        },
    )
    .expect("preview mutation should normalize");
    let rebind = match evaluate_effect_eligibility(normalized.clone())? {
        EffectEligibilityOutcome::RebindRequired(rebind) => rebind,
        other => panic!("expected preview rebind scenario, got {other:?}"),
    };
    Ok(scalar_or_terminal_row(
        format!("seeded-preview-rebind-{index}"),
        EffectLifecycleSeededOutcomeClass::RebindRequired,
        basis.family(),
        EffectFamily::Mutation,
        1,
        support.discovery_for_reporting().to_string(),
        Some(normalized.normalized_digest().to_string()),
        rebind.decision_trace().trace_for_reporting().to_string(),
        None,
        None,
        None,
        Some(rebind.decision_trace().trace_for_reporting().to_string()),
        rebind.counters().clone(),
    ))
}

pub(super) fn preview_derived_advisory_row(
    index: usize,
    stepper: &mut SeedStepper,
) -> Result<EffectLifecycleSeededCertificationRow, IdentityDerivationDenial> {
    let preview = seeded_label("preview", stepper, index);
    let branch = seeded_label("branch", stepper, index);
    let basis = EffectAuthoringBasis::from(preview_derived_inspection_advisory(&preview, &branch));
    let support = discover_effect_lifecycle_support(basis.family(), EffectFamily::Mutation)?;
    let normalized = normalize_raw_effect_intent(
        &basis,
        raw_mutation_effect_with_binding(
            runtime_workflow_binding()?,
            EntityId::new(PartitionId(1), 200 + index as u64, 0),
            seeded_label("advisory", stepper, index),
        ),
    )
    .expect("preview-derived mutation should normalize");
    let advisory = match evaluate_effect_eligibility(normalized.clone())? {
        EffectEligibilityOutcome::Advisory(advisory) => advisory,
        other => panic!("expected advisory scenario, got {other:?}"),
    };
    Ok(scalar_or_terminal_row(
        format!("seeded-preview-derived-advisory-{index}"),
        EffectLifecycleSeededOutcomeClass::Advisory,
        basis.family(),
        EffectFamily::Mutation,
        1,
        support.discovery_for_reporting().to_string(),
        Some(normalized.normalized_digest().to_string()),
        advisory.decision_trace().trace_for_reporting().to_string(),
        None,
        None,
        None,
        Some(advisory.decision_trace().trace_for_reporting().to_string()),
        advisory.counters().clone(),
    ))
}

pub(super) fn store_backed_deferred_row(
    index: usize,
    _stepper: &mut SeedStepper,
) -> Result<EffectLifecycleSeededCertificationRow, IdentityDerivationDenial> {
    deferred_row(
        index,
        EffectAuthoringBasis::store_backed(format!("store-basis-{index}")),
        "seeded-store-backed-deferred",
    )
}

pub(super) fn durable_reload_deferred_row(
    index: usize,
    _stepper: &mut SeedStepper,
) -> Result<EffectLifecycleSeededCertificationRow, IdentityDerivationDenial> {
    deferred_row(
        index,
        EffectAuthoringBasis::durable_reload(format!("durable-reload-{index}")),
        "seeded-durable-reload-deferred",
    )
}

pub(super) fn tenant_merge_denied_row(
    index: usize,
    stepper: &mut SeedStepper,
) -> Result<EffectLifecycleSeededCertificationRow, IdentityDerivationDenial> {
    let basis = EffectAuthoringBasis::from(tenant_mutation_basis(&seeded_label(
        "tenant", stepper, index,
    )));
    let support = discover_effect_lifecycle_support(basis.family(), EffectFamily::Merge)?;
    let normalized = normalize_raw_effect_intent(
        &basis,
        RawEffectIntent::Merge {
            binding: runtime_workflow_binding()?,
            request: workflow_request(
                WorkflowDeclarationFamily::MergeLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMerge,
            ),
            input: MergeLoweringInput::reconcile_into_target(
                BranchId("main".to_string()),
                BranchId(seeded_label("candidate", stepper, index)),
            ),
        },
    )
    .expect("tenant merge should normalize");
    let denied = match evaluate_effect_eligibility(normalized.clone())? {
        EffectEligibilityOutcome::Denied(denied) => denied,
        other => panic!("expected denied tenant merge scenario, got {other:?}"),
    };
    Ok(scalar_or_terminal_row(
        format!("seeded-tenant-merge-denied-{index}"),
        EffectLifecycleSeededOutcomeClass::Denied,
        basis.family(),
        EffectFamily::Merge,
        1,
        support.discovery_for_reporting().to_string(),
        Some(normalized.normalized_digest().to_string()),
        denied.decision_trace().trace_for_reporting().to_string(),
        None,
        None,
        None,
        Some(denied.decision_trace().trace_for_reporting().to_string()),
        denied.counters().clone(),
    ))
}

pub(super) fn deferred_row(
    index: usize,
    basis: EffectAuthoringBasis,
    prefix: &str,
) -> Result<EffectLifecycleSeededCertificationRow, IdentityDerivationDenial> {
    let support = discover_effect_lifecycle_support(basis.family(), EffectFamily::Writeback)?;
    let normalized = normalize_raw_effect_intent(
        &basis,
        RawEffectIntent::Writeback {
            binding: runtime_workflow_binding()?,
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        },
    )
    .expect("deferred writeback should normalize");
    let deferred = match evaluate_effect_eligibility(normalized.clone())? {
        EffectEligibilityOutcome::Deferred(deferred) => deferred,
        other => panic!("expected deferred scenario, got {other:?}"),
    };
    Ok(scalar_or_terminal_row(
        format!("{prefix}-{index}"),
        EffectLifecycleSeededOutcomeClass::Deferred,
        basis.family(),
        EffectFamily::Writeback,
        1,
        support.discovery_for_reporting().to_string(),
        Some(normalized.normalized_digest().to_string()),
        deferred.decision_trace().trace_for_reporting().to_string(),
        None,
        None,
        None,
        Some(deferred.decision_trace().trace_for_reporting().to_string()),
        deferred.counters().clone(),
    ))
}
