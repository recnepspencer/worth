use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use super::eligibility::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    prepare_admitted_domain_capability_contribution_for_materialization,
};
use super::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
    WorthQueryDomainCapabilityTargetBinding, WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
};
use super::{
    WorthQueryDomainCapabilityPayload, WorthQueryDomainCapabilityTransitionOutcome,
    WorthQueryMaterializationReadyDomainCapabilityContribution,
    WorthQueryRequestedDomainCapabilityContribution,
};
use crate::domain_capabilities::identity::{domain_capability_scope_encoder, seal};
use crate::evidence_identity::WorthQueryEvidenceTag;
use crate::intent_admission::admit_runtime_intent_request;
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan, WorthQueryLowerRuntimeSeamKey,
};
use crate::runtime::{
    WorthQueryAdmittedIntentPlan, WorthQueryIntentAdmissionDecision, WorthQueryIntentDeclaration,
};

pub(super) fn ready<P, T>(
    requested: WorthQueryRequestedDomainCapabilityContribution<P, T>,
) -> Result<
    WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
    IdentityDerivationDenial,
>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    let eligible = success(evaluate_requested_domain_capability_contribution(
        requested,
    )?);
    let admitted = success(admit_eligible_domain_capability_contribution(eligible));
    let target = admitted.payload().target().clone();
    Ok(success(
        prepare_admitted_domain_capability_contribution_for_materialization(admitted, target),
    ))
}

pub(super) fn ready_payload<P, T>(
    target: T,
    payload: P,
) -> Result<
    WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
    IdentityDerivationDenial,
>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
    (P, T): super::proof_integration::AllowedContributionBinding<P, T>,
{
    Ok(ready(
        super::proof_integration::create_requested_domain_capability_contribution(target, payload)?,
    )?)
}

pub(super) fn success<T>(outcome: WorthQueryDomainCapabilityTransitionOutcome<T>) -> T {
    match outcome {
        TransitionOutcome::Success(value) => value,
        TransitionOutcome::Denied(denial) => {
            panic!("expected success, got denial {:?}", denial.kind())
        }
        TransitionOutcome::Stale(stale) => {
            panic!("expected success, got stale {}", stale.category())
        }
        TransitionOutcome::RebindRequired(rebind) => {
            panic!(
                "expected success, got rebind-required {}",
                rebind.category()
            )
        }
        TransitionOutcome::Failed(failure) => {
            panic!("expected success, got failure {}", failure.message())
        }
        TransitionOutcome::Deferred(never) => match never {},
    }
}

pub(super) fn intent_declaration(label: &str) -> WorthQueryIntentDeclaration {
    WorthQueryIntentDeclaration::strategy_commit(
        format!("domain-capability.{label}"),
        format!("worth.domain_capability.{label}"),
        "1",
        "worth.domain-capability.fixture",
        crate::runtime::WorthQueryIntentInput::object([(
            "fixture",
            crate::runtime::WorthQueryIntentInput::string(label),
        )]),
    )
}

pub(super) fn admitted_plan(
    label: &str,
) -> Result<WorthQueryAdmittedIntentPlan, IdentityDerivationDenial> {
    let request = crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(
        intent_declaration(label),
    )
    .expect("domain-capability fixture intent request should build");
    Ok(match admit_runtime_intent_request(request)? {
        WorthQueryIntentAdmissionDecision::Admitted(plan) => plan,
        other => panic!("expected admitted domain-capability fixture plan, got {other:?}"),
    })
}

pub(super) fn admitted_plan_target_parts(
    plan_label: &str,
    request_digest: &str,
    eligibility_digest: &str,
    decision_digest: &str,
) -> Result<WorthQueryAdmittedPlanBoundContributionTarget, IdentityDerivationDenial> {
    let fixture_label = seal(
        domain_capability_scope_encoder("worth_query_domain_capability_admitted_plan_fixture_v1")
            .field_shape(WorthQueryEvidenceTag::new("plan_label"), plan_label)
            .field_shape(WorthQueryEvidenceTag::new("request"), request_digest)
            .field_shape(
                WorthQueryEvidenceTag::new("eligibility"),
                eligibility_digest,
            )
            .field_shape(WorthQueryEvidenceTag::new("decision"), decision_digest),
    )?;
    Ok(
        WorthQueryAdmittedPlanBoundContributionTarget::for_admitted_intent_plan(&admitted_plan(
            &fixture_label,
        )?),
    )
}

pub(super) fn declaration_target(label: &str) -> WorthQueryDeclarationBoundContributionTarget {
    WorthQueryDeclarationBoundContributionTarget::for_intent_declaration(&intent_declaration(label))
}

pub(super) fn admitted_plan_target(
    label: &str,
) -> Result<WorthQueryAdmittedPlanBoundContributionTarget, IdentityDerivationDenial> {
    Ok(
        WorthQueryAdmittedPlanBoundContributionTarget::for_admitted_intent_plan(&admitted_plan(
            label,
        )?),
    )
}

pub(super) fn lower_runtime_envelope(
    label: &str,
) -> Result<WorthQueryLowerRuntimeBoundaryEnvelope, IdentityDerivationDenial> {
    let subject_identity =
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "domain-capability-fixture-subject",
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("fixture"),
            label,
        )
        .seal()?;
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        "signal-invalidation-routing",
        subject_identity,
    )?;
    let detail_identity = crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("fixture_detail"),
        label,
    )
    .seal()?;
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request,
        &detail_identity,
    )?;
    let route = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "domain-capability-fixture-route",
            &detail_identity,
        )?,
    )?;
    let retained_evidence =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "domain-capability-fixture",
            &crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_value(
                crate::evidence_identity::WorthQueryEvidenceTag::new("fixture_retained"),
                label,
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

pub(super) fn lower_runtime_target(
    label: &str,
) -> Result<WorthQueryLowerRuntimeBoundaryBoundContributionTarget, IdentityDerivationDenial> {
    Ok(
        WorthQueryLowerRuntimeBoundaryBoundContributionTarget::for_lower_runtime_boundary_envelope(
            &lower_runtime_envelope(label)?,
        ),
    )
}
