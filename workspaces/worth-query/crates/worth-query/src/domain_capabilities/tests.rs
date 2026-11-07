use worth_proof::TransitionOutcome;

use super::authoring::{
    WorthQueryAdmissionContributionAuthoring, WorthQueryAftermathContributionAuthoring,
    WorthQueryContinuityContributionAuthoring, WorthQueryExplanationContributionAuthoring,
    WorthQueryInvariantCapabilityContributionAuthoring, WorthQuerySupportContributionAuthoring,
    WorthQueryWorkflowContributionAuthoring,
};
use super::eligibility::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    prepare_admitted_domain_capability_contribution_for_materialization,
};
use super::payloads::{WorthQuerySupportContributionPayload, WorthQuerySupportContributionPosture};
use super::proof_integration::create_requested_domain_capability_contribution;
use super::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
    WorthQueryDomainCapabilityTargetKind, WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
};
use super::test_support::{
    admitted_plan, admitted_plan_target, declaration_target, lower_runtime_envelope,
    lower_runtime_target, success,
};
use crate::runtime::WorthQueryIntentDeclaration;
use crate::target_binding::{
    WorthQueryAdmittedIntentPlanBindingTarget, WorthQueryBindingTargetWitness,
    WorthQueryIntentDeclarationBindingTarget, WorthQueryLowerRuntimeBoundaryEnvelopeBindingTarget,
};

#[test]
fn requested_contribution_denies_empty_semantic_code() {
    let declaration = sample_declaration("rotate");
    let requested = WorthQueryAdmissionContributionAuthoring::advisory("", "needs clarification")
        .expect("fixture digest must fit")
        .for_intent_declaration(&declaration);

    match evaluate_requested_domain_capability_contribution(requested) {
        TransitionOutcome::Denied(denial) => {
            assert_eq!(
                denial.kind(),
                super::WorthQueryDomainCapabilityProgressionDenialKind::EmptySemanticCode
            );
            assert_eq!(denial.category(), "admission");
            assert_eq!(
                denial.target_kind(),
                WorthQueryDomainCapabilityTargetKind::IntentDeclaration
            );
        }
        _ => panic!("expected empty semantic code denial"),
    }
}

#[test]
fn request_digest_survives_successful_progression() {
    let declaration = sample_declaration("move");
    let requested = WorthQueryAdmissionContributionAuthoring::advisory(
        "arbitration.requires_clarification",
        "multiple candidates remain admissible",
    )
    .expect("fixture digest must fit")
    .for_intent_declaration(&declaration);
    let request_digest = requested.payload().request_digest().to_string();
    let eligible = success(evaluate_requested_domain_capability_contribution(requested));
    let admitted = success(admit_eligible_domain_capability_contribution(eligible));
    let ready = success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            admitted,
            WorthQueryDeclarationBoundContributionTarget::for_intent_declaration(&declaration),
        )
        .expect("fixture digest must fit"),
    );

    assert_eq!(ready.payload().request_digest(), request_digest);
}

#[test]
fn declaration_drift_requires_rebind() {
    let declaration = sample_declaration("offset");
    let requested = WorthQueryAdmissionContributionAuthoring::violation(
        "spatial.target_binding_changed",
        "the original intent declaration no longer matches the contribution binding",
    )
    .expect("fixture digest must fit")
    .for_intent_declaration(&declaration);
    let eligible = success(evaluate_requested_domain_capability_contribution(requested));
    let admitted = success(admit_eligible_domain_capability_contribution(eligible));
    let rebound_target = declaration_target("different-intent-digest");
    let rebound_target_for_reporting = rebound_target
        .target_identity()
        .expect("fixture digest must fit")
        .as_str()
        .to_string();

    match prepare_admitted_domain_capability_contribution_for_materialization(
        admitted,
        rebound_target,
    ) {
        TransitionOutcome::RebindRequired(rebind) => {
            assert_eq!(rebind.category(), "admission");
            assert_eq!(
                rebind.current_target_for_reporting(),
                rebound_target_for_reporting
            );
        }
        _ => panic!("expected rebind-required outcome"),
    }
}

#[test]
fn lower_runtime_boundary_drift_is_stale_not_rebind() {
    let requested = create_requested_domain_capability_contribution(
        lower_runtime_target("envelope-a"),
        WorthQuerySupportContributionPayload::new(
            WorthQuerySupportContributionPosture::DeclarationSupport,
            "runtime.boundary.support",
            "support posture comes from a lower-runtime seam",
        ),
    )
    .expect("fixture digest must fit");
    let eligible = success(evaluate_requested_domain_capability_contribution(requested));
    let admitted = success(admit_eligible_domain_capability_contribution(eligible));

    match prepare_admitted_domain_capability_contribution_for_materialization(
        admitted,
        lower_runtime_target("envelope-b"),
    ) {
        TransitionOutcome::Stale(stale) => {
            assert_eq!(stale.category(), "support-traceability");
            assert_ne!(
                stale.bound_target_for_reporting(),
                stale.current_target_for_reporting()
            );
        }
        _ => panic!("expected stale outcome"),
    }
}

#[test]
fn every_category_reaches_admitted_form_through_the_same_lifecycle() {
    let declaration_target = declaration_target("intent-declaration-a");
    let admitted_plan_target = admitted_plan_target("admitted-plan-a");
    let envelope_target = lower_runtime_target("boundary-envelope-a");

    assert_admitted(
        WorthQueryAdmissionContributionAuthoring::advisory(
            "admission.advisory",
            "admission detail",
        )
        .expect("fixture digest must fit")
        .bind_to_declaration_target(declaration_target.clone()),
    );
    assert_admitted(
        WorthQuerySupportContributionAuthoring::declaration_support(
            "support.declaration",
            "support detail",
        )
        .expect("fixture digest must fit")
        .bind_to_declaration_target(declaration_target.clone()),
    );
    assert_admitted(
        WorthQueryInvariantCapabilityContributionAuthoring::capability_gap(
            "invariant.capability_gap",
            "invariant detail",
        )
        .expect("fixture digest must fit")
        .bind_to_lower_runtime_boundary_target(envelope_target.clone()),
    );
    assert_admitted(
        WorthQueryWorkflowContributionAuthoring::promotion_eligible(
            "workflow.promotion_eligible",
            "workflow detail",
        )
        .expect("fixture digest must fit")
        .bind_to_admitted_plan_target(admitted_plan_target.clone()),
    );
    assert_admitted(
        WorthQueryContinuityContributionAuthoring::preserved(
            "continuity.preserved",
            "continuity detail",
        )
        .expect("fixture digest must fit")
        .bind_to_admitted_plan_target(admitted_plan_target.clone()),
    );
    assert_admitted(
        WorthQueryAftermathContributionAuthoring::declares_residue(
            "aftermath.residue",
            "aftermath detail",
        )
        .expect("fixture digest must fit")
        .bind_to_lower_runtime_boundary_target(envelope_target),
    );
    assert_admitted(
        WorthQueryExplanationContributionAuthoring::requires_context(
            "explanation.context",
            "explanation detail",
        )
        .expect("fixture digest must fit")
        .bind_to_declaration_target(declaration_target),
    );
}

#[test]
fn compatibility_targets_share_digests_with_the_shared_binding_core() {
    let declaration = sample_declaration("bind");
    let legacy_declaration =
        WorthQueryDeclarationBoundContributionTarget::for_intent_declaration(&declaration);
    let shared_declaration =
        WorthQueryIntentDeclarationBindingTarget::for_intent_declaration(&declaration);

    assert_eq!(
        WorthQueryBindingTargetWitness::binding_digest(&legacy_declaration),
        shared_declaration.binding_digest()
    );
    assert_eq!(
        WorthQueryBindingTargetWitness::target_digest(&legacy_declaration),
        shared_declaration.target_digest()
    );

    let plan = admitted_plan("admitted-plan-a.request-a.eligibility-a.decision-a")
        .expect("fixture digest must fit");
    let plan_target =
        WorthQueryAdmittedPlanBoundContributionTarget::for_admitted_intent_plan(&plan);
    let shared_plan = WorthQueryAdmittedIntentPlanBindingTarget::for_admitted_intent_plan(&plan);
    assert_eq!(
        WorthQueryBindingTargetWitness::binding_digest(&plan_target),
        shared_plan.binding_digest()
    );

    let envelope = lower_runtime_envelope("boundary-envelope-a").expect("fixture digest must fit");
    let envelope_target =
        WorthQueryLowerRuntimeBoundaryBoundContributionTarget::for_lower_runtime_boundary_envelope(
            &envelope,
        );
    let shared_envelope =
        WorthQueryLowerRuntimeBoundaryEnvelopeBindingTarget::for_lower_runtime_boundary_envelope(
            &envelope,
        );
    assert_eq!(
        WorthQueryBindingTargetWitness::binding_digest(&envelope_target),
        shared_envelope.binding_digest()
    );
}

fn sample_declaration(name: &str) -> WorthQueryIntentDeclaration {
    WorthQueryIntentDeclaration::strategy_commit(
        name,
        format!("worth.spatial.{name}"),
        "1",
        "worth.spatial.intent",
        crate::runtime::WorthQueryIntentInput::object([(
            "entity",
            crate::runtime::WorthQueryIntentInput::string(format!("edge:{name}")),
        )]),
    )
}

fn assert_admitted<P, T>(requested: super::WorthQueryRequestedDomainCapabilityContribution<P, T>)
where
    P: super::WorthQueryDomainCapabilityPayload,
    T: super::WorthQueryDomainCapabilityTargetBinding,
{
    let eligible = success(evaluate_requested_domain_capability_contribution(requested));
    let _admitted = success(admit_eligible_domain_capability_contribution(eligible));
}
