mod phase_progression;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
pub(super) use phase_progression::{admitted_ready, success};
mod replay_gap;
pub(super) use replay_gap::{store_backed_replay_gap_parts, store_backed_replay_gap_request};

use worth_foundational::facade::{AspectKey, FieldKey};
use worth_proof::TransitionOutcome;
use worth_relational::facade::runtime::InvariantCatalog;
use worth_runtime_bridge::facade::BridgePreviewSessionIdentity;

use crate::domain_capabilities::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    prepare_admitted_domain_capability_contribution_for_materialization,
    WorthQueryInstalledAdmittedPlanContributionTarget,
    WorthQueryInstalledDeclarationContributionTarget,
    WorthQueryInstalledLowerRuntimeContributionTarget, WorthQueryLowerRuntimeExplanationRequest,
    WorthQueryProjectionContractRequest, WorthQueryRequestedAdmissionContribution,
    WorthQueryRequestedAftermathContribution, WorthQueryRequestedContinuityContribution,
    WorthQueryRequestedDomainCapabilityContribution, WorthQueryRequestedExplanationContribution,
    WorthQueryRequestedInvariantCapabilityContribution, WorthQueryRequestedSupportContribution,
    WorthQueryRequestedWorkflowContribution, WorthQuerySupportContributionAuthoring,
};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan, WorthQueryLowerRuntimeSeamKey,
};
use crate::projection_consumption::{
    ProjectMaterializedFacts, ProjectionConsumptionBindingContext, ProjectionConsumptionSource,
};
use crate::runtime::{
    anchor_causal_observation, causal_inspection_target, resolve_causal_evidence_references,
    CausalEvidenceFamily, CausalEvidenceReferenceResolution, CausalEvidenceReferenceSet,
    CausalInspectionMaterializationPolicy, CausalInspectionReason, CausalInspectionRedactionPolicy,
    CausalInspectionTarget, QueryObservationReceipt, WorthQueryAdmittedIntentPlan,
    WorthQueryGraphCompositionCapabilityClass, WorthQueryIntentDeclaration, WorthQueryIntentInput,
};

pub(super) fn intent_declaration(name: &str) -> WorthQueryIntentDeclaration {
    WorthQueryIntentDeclaration::strategy_commit(
        name,
        "spatial.commit",
        "1",
        "geometry.patch",
        WorthQueryIntentInput::object([("edge", WorthQueryIntentInput::string("e-1"))]),
    )
}

pub(super) fn admitted_basis_observation_plan(
) -> Result<WorthQueryAdmittedIntentPlan, IdentityDerivationDenial> {
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::basis_observation_lane(
            crate::basis_lifecycle::RawBasisIntent::CurrentHead,
        )
        .expect("basis-observation request should build");

    Ok(
        match crate::intent_admission::admit_runtime_intent_request(request)? {
            crate::intent_admission::WorthQueryIntentAdmissionDecision::Admitted(plan) => plan,
            other => panic!("expected admitted basis-observation plan, got {other:?}"),
        },
    )
}

pub(super) fn admitted_projection_consumption_plan(
) -> Result<WorthQueryAdmittedIntentPlan, IdentityDerivationDenial> {
    let declaration = crate::projection_consumption::declare_projection_consumption(
        admitted_projection_source()?,
        admitted_projection_binding(),
        ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("field")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("visible")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    )
    .expect("projection declaration should build");
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::projection_consumption(
            declaration,
        )
        .expect("projection request should build");

    Ok(
        match crate::intent_admission::admit_runtime_intent_request(request)? {
            crate::intent_admission::WorthQueryIntentAdmissionDecision::Admitted(plan) => plan,
            other => panic!("expected admitted projection-consumption plan, got {other:?}"),
        },
    )
}

pub(super) fn projection_contract_request(
) -> Result<WorthQueryProjectionContractRequest, IdentityDerivationDenial> {
    Ok(WorthQueryProjectionContractRequest::new(
        admitted_projection_source()?,
        admitted_projection_binding(),
        ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("field")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("visible")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    ))
}

pub(super) fn projection_contract_parts() -> Result<
    (
        ProjectionConsumptionSource,
        ProjectionConsumptionBindingContext,
        ProjectMaterializedFacts,
    ),
    IdentityDerivationDenial,
> {
    Ok(projection_contract_request()?.into_parts())
}

pub(super) fn lower_runtime_envelope(
    target_digest: &str,
) -> Result<WorthQueryLowerRuntimeBoundaryEnvelope, IdentityDerivationDenial> {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        "signal-invalidation-routing",
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "certification-report-target",
        )
        .field_value(WorthQueryEvidenceTag::new("test_target"), target_digest)
        .seal()?,
    )?;
    let detail_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(WorthQueryEvidenceTag::new("test_detail"), "detail")
            .seal()?;
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request,
        &detail_identity,
    )?;
    let route = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "certification-report-route",
            &detail_identity,
        )?,
    )?;
    let retained_evidence =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "domain-capability-certification-report",
            &WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_value(
                WorthQueryEvidenceTag::new("certification_report_target"),
                target_digest,
            )
            .seal()?,
        )?;
    let boundary = WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(
        &route,
        &retained_evidence,
    )?;

    Ok(WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
        WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        &route,
        &boundary,
        &retained_evidence,
    )?)
}

fn admitted_projection_source() -> Result<ProjectionConsumptionSource, IdentityDerivationDenial> {
    Ok(
        crate::projection_consumption::intent_admission_admitted_projection_declaration()?
            .source()
            .clone(),
    )
}

fn admitted_projection_binding() -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::intent_admission_certification_binding(
        "shape-digest",
        "query-digest",
        "shape-digest",
        "query-read:certification-admitted",
        "narrowed-shape-digest",
        "policy-digest",
        "tenant-schema-digest",
        vec![field_path("field.visible")],
    )
}

fn field_path(path: &str) -> crate::authorized_projection::AuthorizedProjectionFieldPath {
    let Some((aspect, field)) = path.split_once('.') else {
        panic!("domain capability fixture field path should include an aspect and field");
    };
    crate::authorized_projection::AuthorizedProjectionFieldPath::from_native_keys(
        AspectKey::new(aspect.to_string()).expect("fixture aspect key"),
        FieldKey::new(field.to_string()).expect("fixture field key"),
    )
}

pub(super) fn support_traceability_requested(
    target: WorthQueryInstalledDeclarationContributionTarget,
) -> Result<
    WorthQueryRequestedSupportContribution<WorthQueryInstalledDeclarationContributionTarget>,
    IdentityDerivationDenial,
> {
    Ok(
        WorthQuerySupportContributionAuthoring::declaration_traceability(
            "worth.spatial.traceability.edge_split",
            "declaration-scoped support remains declaration scoped",
        )
        .bind_to_installed_target(target)?,
    )
}

pub(super) fn plain_support_requested(
    target: WorthQueryInstalledDeclarationContributionTarget,
) -> Result<
    WorthQueryRequestedSupportContribution<WorthQueryInstalledDeclarationContributionTarget>,
    IdentityDerivationDenial,
> {
    Ok(WorthQuerySupportContributionAuthoring::declaration_support(
        "worth.spatial.traceability.edge_split",
        "declaration-scoped support remains declaration scoped",
    )
    .bind_to_installed_target(target)?)
}

pub(super) fn plan_support_requested(
    target: WorthQueryInstalledAdmittedPlanContributionTarget,
) -> Result<
    WorthQueryRequestedSupportContribution<WorthQueryInstalledAdmittedPlanContributionTarget>,
    IdentityDerivationDenial,
> {
    Ok(WorthQuerySupportContributionAuthoring::declaration_support(
        "worth.spatial.support.runtime_floor",
        "runtime floor remains explicitly supported",
    )
    .bind_to_installed_target(target)?)
}

pub(super) fn admission_requested(
    target: WorthQueryInstalledAdmittedPlanContributionTarget,
) -> Result<
    WorthQueryRequestedAdmissionContribution<WorthQueryInstalledAdmittedPlanContributionTarget>,
    IdentityDerivationDenial,
> {
    Ok(
        crate::domain_capabilities::WorthQueryAdmissionContributionAuthoring::advisory(
            "worth.spatial.admission.routing_gap",
            "runtime routing still needs clarification",
        )?
        .bind_to_installed_target(target)?,
    )
}

pub(super) fn workflow_requested(
    target: WorthQueryInstalledDeclarationContributionTarget,
) -> Result<
    WorthQueryRequestedWorkflowContribution<WorthQueryInstalledDeclarationContributionTarget>,
    IdentityDerivationDenial,
> {
    Ok(crate::domain_capabilities::WorthQueryWorkflowContributionAuthoring::promotion_eligible_mutation_lowering(
        "worth.spatial.workflow.preview_mutation",
        "preview mutation planning should preserve canonical workflow semantics",
        BridgePreviewSessionIdentity::from_stable_name("preview-session:certification"),
    )?
    .bind_to_installed_target(target)?)
}

pub(super) fn continuity_requested(
    target: WorthQueryInstalledAdmittedPlanContributionTarget,
) -> Result<
    WorthQueryRequestedContinuityContribution<WorthQueryInstalledAdmittedPlanContributionTarget>,
    IdentityDerivationDenial,
> {
    Ok(
        crate::domain_capabilities::WorthQueryContinuityContributionAuthoring::preserved_rebind(
            "edge:before",
            "edge:after",
            "worth.spatial.continuity.edge_split",
            "edge split preserves one authoritative successor",
        )?
        .bind_to_installed_target(target)?,
    )
}

pub(super) fn aftermath_requested(
    target: WorthQueryInstalledAdmittedPlanContributionTarget,
) -> Result<
    WorthQueryRequestedAftermathContribution<WorthQueryInstalledAdmittedPlanContributionTarget>,
    IdentityDerivationDenial,
> {
    let (source, binding, facts) = projection_contract_parts()?;
    Ok(crate::domain_capabilities::WorthQueryAftermathContributionAuthoring::consumes_projection_contract(
        "worth.spatial.aftermath.projection_contract",
        "projection aftermath should bind a stable contract",
        source,
        binding,
        facts,
    )?
    .bind_to_installed_target(target)?)
}

pub(super) fn explanation_requested(
    installed_target: WorthQueryInstalledLowerRuntimeContributionTarget,
) -> Result<
    WorthQueryRequestedExplanationContribution<WorthQueryInstalledLowerRuntimeContributionTarget>,
    IdentityDerivationDenial,
> {
    let (reference_set, inspection_target) = store_backed_replay_gap_parts()?;
    Ok(crate::domain_capabilities::WorthQueryExplanationContributionAuthoring::store_backed_replay_gap_explanation(
        "worth.spatial.explanation.store_backed_replay",
        "store-backed replay should preserve denied explanation identity",
        reference_set,
        inspection_target,
        vec![CausalEvidenceFamily::QueryInspection],
        crate::runtime::CausalInspectionRedactionPolicy::PreserveDetail,
        crate::runtime::CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )?
    .bind_to_installed_target(installed_target)?)
}

pub(super) fn invariant_requested(
    target: WorthQueryInstalledDeclarationContributionTarget,
) -> Result<
    WorthQueryRequestedInvariantCapabilityContribution<
        WorthQueryInstalledDeclarationContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(crate::domain_capabilities::WorthQueryInvariantCapabilityContributionAuthoring::invariant_registration(
        InvariantCatalog::default(),
        "worth.spatial.invariant.edge_split",
        "geometry kernel must reject invalid edge splits",
    )?
    .bind_to_installed_target(target)?)
}

pub(super) fn capability_requested(
    target: WorthQueryInstalledLowerRuntimeContributionTarget,
) -> Result<
    WorthQueryRequestedInvariantCapabilityContribution<
        WorthQueryInstalledLowerRuntimeContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(crate::domain_capabilities::WorthQueryInvariantCapabilityContributionAuthoring::graph_capability_gap(
        "face-split-target-combination",
        WorthQueryGraphCompositionCapabilityClass::TargetCombination,
        "worth.spatial.capability.face_split",
        "face split still depends on graph-composition capability support",
    )?
    .bind_to_installed_target(target)?)
}

pub(super) fn invariant_denial_requested(
    target: WorthQueryInstalledLowerRuntimeContributionTarget,
) -> Result<
    WorthQueryRequestedInvariantCapabilityContribution<
        WorthQueryInstalledLowerRuntimeContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(crate::domain_capabilities::WorthQueryInvariantCapabilityContributionAuthoring::graph_invariant_denial(
        "non_manifold_edge_split",
        vec!["edges".to_string()],
        vec!["edge_symbol".to_string()],
        vec!["face-edge".to_string()],
        vec!["split-existing-target".to_string()],
        "program-digest:domain-capability",
        "breadth-digest:domain-capability",
        "counter-snapshot:domain-capability",
        "worth.spatial.invariant.non_manifold",
        "non-manifold edge split must deny graph composition",
    )?
    .bind_to_installed_target(target)?)
}
