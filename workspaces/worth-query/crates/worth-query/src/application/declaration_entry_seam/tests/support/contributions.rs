use crate::application::{
    WorthQueryCanonicalDeclarationArtifact, WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
};
use crate::domain_capabilities::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution, WorthQueryAdmissionContributionAuthoring,
    WorthQueryAdmittedAdmissionContribution, WorthQueryAdmittedAftermathContribution,
    WorthQueryAdmittedContinuityContribution, WorthQueryAdmittedExplanationContribution,
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryAdmittedSupportContribution,
    WorthQueryAdmittedWorkflowContribution, WorthQueryAftermathContributionAuthoring,
    WorthQueryContinuityContributionAuthoring, WorthQueryDeclarationBoundContributionTarget,
    WorthQueryExplanationContributionAuthoring, WorthQuerySupportContributionAuthoring,
    WorthQueryWorkflowContributionAuthoring,
};
use worth_proof::TransitionOutcome;

pub fn admitted_declaration_support(
    declaration_label: &str,
    semantic_code: &str,
    detail: &str,
) -> WorthQueryAdmittedSupportContribution<WorthQueryDeclarationBoundContributionTarget> {
    admitted(
        WorthQuerySupportContributionAuthoring::declaration_support(semantic_code, detail)
            .expect("fixture digest must fit")
            .bind_to_declaration_target(declaration_target(declaration_label))
            .expect("fixture digest must fit"),
    )
}

pub fn admitted_declaration_explanation<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    declaration: &WorthQueryCanonicalDeclarationArtifact<D, I>,
    semantic_code: &str,
    detail: &str,
) -> WorthQueryAdmittedExplanationContribution<WorthQueryDeclarationBoundContributionTarget> {
    admitted(
        WorthQueryExplanationContributionAuthoring::requires_context(semantic_code, detail)
            .expect("fixture digest must fit")
            .bind_to_declaration_target(canonical_declaration_target(declaration))
            .expect("fixture digest must fit"),
    )
}

pub fn admitted_declaration_advisory<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    declaration: &WorthQueryCanonicalDeclarationArtifact<D, I>,
    semantic_code: &str,
    detail: &str,
) -> WorthQueryAdmittedAdmissionContribution<WorthQueryDeclarationBoundContributionTarget> {
    admitted(
        WorthQueryAdmissionContributionAuthoring::advisory(semantic_code, detail)
            .expect("fixture digest must fit")
            .bind_to_declaration_target(canonical_declaration_target(declaration))
            .expect("fixture digest must fit"),
    )
}

pub fn admitted_declaration_workflow(
    declaration_label: &str,
    semantic_code: &str,
    detail: &str,
) -> WorthQueryAdmittedWorkflowContribution<WorthQueryDeclarationBoundContributionTarget> {
    admitted_generic(
        WorthQueryWorkflowContributionAuthoring::preview_only(semantic_code, detail)
            .expect("fixture digest must fit")
            .bind_to_declaration_target(declaration_target(declaration_label))
            .expect("fixture digest must fit"),
    )
}

pub fn admitted_plan_support(
    plan: &crate::runtime::WorthQueryAdmittedIntentPlan,
    semantic_code: &str,
    detail: &str,
) -> WorthQueryAdmittedSupportContribution<WorthQueryAdmittedPlanBoundContributionTarget> {
    admitted_generic(
        WorthQuerySupportContributionAuthoring::narrowed_support(semantic_code, detail)
            .expect("fixture digest must fit")
            .for_admitted_intent_plan(plan)
            .expect("fixture digest must fit"),
    )
}

pub fn admitted_plan_workflow(
    plan: &crate::runtime::WorthQueryAdmittedIntentPlan,
    semantic_code: &str,
    detail: &str,
) -> WorthQueryAdmittedWorkflowContribution<WorthQueryAdmittedPlanBoundContributionTarget> {
    admitted_generic(
        WorthQueryWorkflowContributionAuthoring::preview_only(semantic_code, detail)
            .expect("fixture digest must fit")
            .for_admitted_intent_plan(plan)
            .expect("fixture digest must fit"),
    )
}

pub fn admitted_plan_continuity(
    plan: &crate::runtime::WorthQueryAdmittedIntentPlan,
    semantic_code: &str,
    detail: &str,
) -> WorthQueryAdmittedContinuityContribution<WorthQueryAdmittedPlanBoundContributionTarget> {
    admitted_generic(
        WorthQueryContinuityContributionAuthoring::preserved(semantic_code, detail)
            .expect("fixture digest must fit")
            .for_admitted_intent_plan(plan)
            .expect("fixture digest must fit"),
    )
}

pub fn admitted_plan_aftermath(
    plan: &crate::runtime::WorthQueryAdmittedIntentPlan,
    semantic_code: &str,
    detail: &str,
) -> WorthQueryAdmittedAftermathContribution<WorthQueryAdmittedPlanBoundContributionTarget> {
    admitted_generic(
        WorthQueryAftermathContributionAuthoring::declares_residue(semantic_code, detail)
            .expect("fixture digest must fit")
            .for_admitted_intent_plan(plan)
            .expect("fixture digest must fit"),
    )
}

pub fn admitted_lower_runtime_explanation(
    envelope: &crate::runtime::WorthQueryLowerRuntimeBoundaryEnvelope,
    semantic_code: &str,
    detail: &str,
) -> WorthQueryAdmittedExplanationContribution<
    crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
> {
    admitted_generic(
        WorthQueryExplanationContributionAuthoring::requires_context(semantic_code, detail)
            .expect("fixture digest must fit")
            .for_lower_runtime_boundary_envelope(envelope)
            .expect("fixture digest must fit"),
    )
}

pub fn admitted_lower_runtime_aftermath(
    envelope: &crate::runtime::WorthQueryLowerRuntimeBoundaryEnvelope,
    semantic_code: &str,
    detail: &str,
) -> WorthQueryAdmittedAftermathContribution<
    crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
> {
    admitted_generic(
        WorthQueryAftermathContributionAuthoring::declares_residue(semantic_code, detail)
            .expect("fixture digest must fit")
            .for_lower_runtime_boundary_envelope(envelope)
            .expect("fixture digest must fit"),
    )
}

fn admitted<P>(
    requested: crate::domain_capabilities::WorthQueryRequestedDomainCapabilityContribution<
        P,
        WorthQueryDeclarationBoundContributionTarget,
    >,
) -> crate::domain_capabilities::WorthQueryAdmittedDomainCapabilityContribution<
    P,
    WorthQueryDeclarationBoundContributionTarget,
>
where
    P: crate::domain_capabilities::WorthQueryDomainCapabilityPayload,
{
    admitted_generic(requested)
}

fn admitted_generic<P, T>(
    requested: crate::domain_capabilities::WorthQueryRequestedDomainCapabilityContribution<P, T>,
) -> crate::domain_capabilities::WorthQueryAdmittedDomainCapabilityContribution<P, T>
where
    P: crate::domain_capabilities::WorthQueryDomainCapabilityPayload,
    T: crate::domain_capabilities::WorthQueryDomainCapabilityTargetBinding,
{
    let eligible = success(evaluate_requested_domain_capability_contribution(requested));
    success(admit_eligible_domain_capability_contribution(eligible))
}

fn success<T>(
    outcome: crate::domain_capabilities::WorthQueryDomainCapabilityTransitionOutcome<T>,
) -> T {
    match outcome {
        TransitionOutcome::Success(value) => value,
        TransitionOutcome::Denied(denial) => {
            panic!("expected success, got denial {:?}", denial.kind())
        }
        TransitionOutcome::Stale(stale) => {
            panic!("expected success, got stale {}", stale.category())
        }
        TransitionOutcome::RebindRequired(rebind) => {
            panic!("expected success, got rebind {}", rebind.category())
        }
        TransitionOutcome::Failed(failure) => {
            panic!("expected success, got failure {}", failure.message())
        }
        TransitionOutcome::Deferred(never) => match never {},
    }
}

fn declaration_target(label: &str) -> WorthQueryDeclarationBoundContributionTarget {
    let declaration = crate::runtime::WorthQueryIntentDeclaration::strategy_commit(
        format!("declaration-entry-seam.{label}"),
        format!("worth.declaration_entry_seam.{label}"),
        "1",
        "worth.declaration-entry-seam.fixture",
        crate::runtime::WorthQueryIntentInput::object([(
            "fixture",
            crate::runtime::WorthQueryIntentInput::string(label),
        )]),
    );
    WorthQueryDeclarationBoundContributionTarget::for_intent_declaration(&declaration)
}

fn canonical_declaration_target<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    declaration: &WorthQueryCanonicalDeclarationArtifact<D, I>,
) -> WorthQueryDeclarationBoundContributionTarget {
    WorthQueryDeclarationBoundContributionTarget::for_canonical_declaration(declaration)
}
