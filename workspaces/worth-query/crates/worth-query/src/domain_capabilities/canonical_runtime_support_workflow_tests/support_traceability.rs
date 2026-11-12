use super::super::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDomainCapabilityTargetBinding,
};
use super::super::test_support::{
    admitted_plan_target, admitted_plan_target_parts, ready, success,
};
use super::super::{
    materialize_intent_admission_support_traceability_report,
    WorthQuerySupportContributionAuthoring, WorthQuerySupportContributionPayload,
    WorthQuerySupportContributionPosture,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[test]
fn support_traceability_materializer_builds_domain_scoped_report(
) -> Result<(), IdentityDerivationDenial> {
    let plan_target = admitted_plan_target("plan-support")?;
    let report = success(materialize_intent_admission_support_traceability_report(
        ready_support(WorthQuerySupportContributionAuthoring::declaration_support(
            "spatial.arbitration.support",
            "multiple candidates remain admissible",
        )?)?,
    )?);

    assert_eq!(report.rows().len(), 1);
    let row = &report.rows()[0];
    assert_eq!(row.lane(), "domain_support");
    assert_eq!(row.family(), "authoritative-user-intent");
    assert_eq!(row.entrypoint(), "WorthQueryRuntime::execute_intent");
    assert_eq!(
        row.support_detail(),
        "spatial.arbitration.support:multiple candidates remain admissible"
    );
    assert!(row.target_binding_digest().is_some());
    let (_, _, request_digest, eligibility_digest, decision_digest) = plan_target
        .semantics()
        .admitted_intent_plan()
        .expect("plan semantics");
    assert_eq!(row.request_digest(), Some(request_digest));
    assert_eq!(row.eligibility_digest(), Some(eligibility_digest));
    assert_eq!(row.decision_digest(), Some(decision_digest));

    Ok(())
}

#[test]
fn equivalent_support_meaning_materializes_same_traceability_digest(
) -> Result<(), IdentityDerivationDenial> {
    let authored = success(materialize_intent_admission_support_traceability_report(
        ready_support(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "spatial.traceability",
                "query admission support is declaration-scoped",
            )?,
        )?,
    )?);
    let direct = success(materialize_intent_admission_support_traceability_report(
        ready(
            super::super::proof_integration::create_requested_domain_capability_contribution(
                admitted_plan_target("plan-support")?,
                WorthQuerySupportContributionPayload::new(
                    WorthQuerySupportContributionPosture::DeclarationTraceability,
                    "spatial.traceability",
                    "query admission support is declaration-scoped",
                )
                .expect("positive fixture identity fits canonical digest bounds"),
            )?,
        )?,
    )?);

    assert_eq!(
        authored.decision_support_traceability_digest(),
        direct.decision_support_traceability_digest()
    );

    Ok(())
}

#[test]
fn support_traceability_digest_changes_when_admitted_plan_scope_changes(
) -> Result<(), IdentityDerivationDenial> {
    let left = success(materialize_intent_admission_support_traceability_report(
        ready(
            super::super::proof_integration::create_requested_domain_capability_contribution(
                admitted_plan_target_parts(
                    "plan-support-left",
                    "request-left",
                    "eligibility-left",
                    "decision-left",
                )?,
                WorthQuerySupportContributionPayload::new(
                    WorthQuerySupportContributionPosture::DeclarationSupport,
                    "spatial.traceability",
                    "support stays attached to the admitted plan",
                )
                .expect("positive fixture identity fits canonical digest bounds"),
            )?,
        )?,
    )?);
    let right = success(materialize_intent_admission_support_traceability_report(
        ready(
            super::super::proof_integration::create_requested_domain_capability_contribution(
                admitted_plan_target_parts(
                    "plan-support-right",
                    "request-right",
                    "eligibility-right",
                    "decision-right",
                )?,
                WorthQuerySupportContributionPayload::new(
                    WorthQuerySupportContributionPosture::DeclarationSupport,
                    "spatial.traceability",
                    "support stays attached to the admitted plan",
                )
                .expect("positive fixture identity fits canonical digest bounds"),
            )?,
        )?,
    )?);

    assert_ne!(
        left.decision_support_traceability_digest(),
        right.decision_support_traceability_digest()
    );

    Ok(())
}

fn ready_support(
    authoring: WorthQuerySupportContributionAuthoring,
) -> Result<
    super::super::WorthQueryMaterializationReadySupportContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    ready(authoring.bind_to_admitted_plan_target(admitted_plan_target("plan-support")?)?)
}
