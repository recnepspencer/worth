use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use super::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDomainCapabilityTargetBinding,
};
use super::test_support::{admitted_plan_target, ready, success};
use super::{
    materialize_intent_admission_support_traceability_row, materialize_runtime_admission_decision,
    materialize_runtime_admission_support_traceability_report,
    materialize_runtime_admission_support_traceability_row,
    materialize_runtime_continuity_evidence, WorthQueryAdmissionContributionAuthoring,
    WorthQueryContinuityContributionAuthoring, WorthQuerySupportContributionAuthoring,
};
use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceTag;
use crate::runtime::WorthQueryMutationEvidenceDigest;
use crate::target_binding::WorthQueryBindingTargetWitness;

#[test]
fn admission_runtime_materializer_builds_query_decisions() -> Result<(), IdentityDerivationDenial> {
    let advisory = success(materialize_runtime_admission_decision(ready_admission(
        WorthQueryAdmissionContributionAuthoring::advisory_at_stage(
            "spatial_arbitration",
            "spatial.arbitration.requires_clarification",
            "multiple candidates remain admissible",
        )?,
    )?));
    let violation = success(materialize_runtime_admission_decision(ready_admission(
        WorthQueryAdmissionContributionAuthoring::violation_at_stage(
            "spatial_arbitration",
            "spatial.arbitration.invalid_target",
            "requested target violates spatial law",
        )?,
    )?));

    match advisory {
        crate::intent_admission::WorthQueryIntentAdmissionDecision::Advisory(advisory) => {
            assert_eq!(advisory.family().as_str(), "authoritative-user-intent");
            assert_eq!(
                advisory.entrypoint().as_str(),
                "WorthQueryRuntime::execute_intent"
            );
            assert_eq!(advisory.stage(), "spatial_arbitration");
        }
        other => panic!("expected advisory decision, got {other:?}"),
    }

    match violation {
        crate::intent_admission::WorthQueryIntentAdmissionDecision::Violation(violation) => {
            assert_eq!(violation.family().as_str(), "authoritative-user-intent");
            assert_eq!(
                violation.entrypoint().as_str(),
                "WorthQueryRuntime::execute_intent"
            );
            assert_eq!(violation.stage(), "spatial_arbitration");
        }
        other => panic!("expected violation decision, got {other:?}"),
    }

    Ok(())
}

#[test]
fn admission_support_only_runtime_materializer_builds_support_traceability(
) -> Result<(), IdentityDerivationDenial> {
    let row = success(materialize_runtime_admission_support_traceability_row(
        ready_admission(WorthQueryAdmissionContributionAuthoring::support_only(
            "spatial.arbitration.support_only",
            "declaration remains support-scoped only",
        )?)?,
    ));
    let report = success(materialize_runtime_admission_support_traceability_report(
        ready_admission(WorthQueryAdmissionContributionAuthoring::support_only(
            "spatial.arbitration.support_only",
            "declaration remains support-scoped only",
        )?)?,
    ));

    assert_eq!(row.family(), "authoritative-user-intent");
    assert_eq!(row.entrypoint(), "WorthQueryRuntime::execute_intent");
    assert_eq!(row.lane(), "admission_local_support");
    assert_eq!(
        row.support_detail(),
        "spatial.arbitration.support_only:declaration remains support-scoped only"
    );
    assert_eq!(report.rows().len(), 1);
    assert_eq!(report.rows()[0], row);

    Ok(())
}

#[test]
fn admission_support_traceability_materializer_denies_non_support_postures(
) -> Result<(), IdentityDerivationDenial> {
    let advisory = materialize_runtime_admission_support_traceability_row(ready_admission(
        WorthQueryAdmissionContributionAuthoring::advisory(
            "spatial.arbitration.requires_clarification",
            "multiple candidates remain admissible",
        )?,
    )?);
    let violation = materialize_runtime_admission_support_traceability_report(ready_admission(
        WorthQueryAdmissionContributionAuthoring::violation(
            "spatial.arbitration.invalid_target",
            "requested target violates spatial law",
        )?,
    )?);

    assert!(matches!(
        advisory,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture
    ));
    assert!(matches!(
        violation,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture
    ));

    Ok(())
}

#[test]
fn admission_local_support_and_support_traceability_rows_stay_distinct(
) -> Result<(), IdentityDerivationDenial> {
    let admission_row = success(materialize_runtime_admission_support_traceability_row(
        ready_admission(WorthQueryAdmissionContributionAuthoring::support_only(
            "spatial.shared.support",
            "same support detail through shared runtime family",
        )?)?,
    ));
    let support_row = success(materialize_intent_admission_support_traceability_row(
        ready_support(WorthQuerySupportContributionAuthoring::declaration_support(
            "spatial.shared.support",
            "same support detail through shared runtime family",
        ))?,
    ));

    assert_eq!(admission_row.support_detail(), support_row.support_detail());
    assert_eq!(admission_row.family(), support_row.family());
    assert_eq!(admission_row.entrypoint(), support_row.entrypoint());
    assert_ne!(admission_row.lane(), support_row.lane());
    assert_ne!(admission_row.row_digest(), support_row.row_digest());

    Ok(())
}

#[test]
fn continuity_runtime_materializer_builds_continuity_evidence(
) -> Result<(), IdentityDerivationDenial> {
    let preserved = success(materialize_runtime_continuity_evidence(ready_continuity(
        WorthQueryContinuityContributionAuthoring::preserved_rebind(
            "edge:12",
            "edge:14",
            "continuity.identity.preserved",
            "edge retarget keeps a single successor identity",
        )?,
    )?)?);
    let split = success(materialize_runtime_continuity_evidence(ready_continuity(
        WorthQueryContinuityContributionAuthoring::split_successors(
            "edge:12",
            ["edge:14", "edge:15"],
            "continuity.identity.split",
            "edge split produces two descendant identities",
        )?,
    )?)?);

    assert_eq!(
        preserved.outcome_class(),
        crate::runtime::WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor
    );
    assert_eq!(preserved.prior_authoritative_identity().as_str(), "edge:12");
    let expected_binding_target = admitted_plan_target("plan-continuity")?;
    let expected_binding_identity =
        domain_capability_scope_encoder("domain_capability_continuity_binding_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("target_kind"),
                WorthQueryDomainCapabilityTargetBinding::kind(&expected_binding_target).as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("binding"),
                &WorthQueryBindingTargetWitness::binding_identity(&expected_binding_target),
            )
            .seal()?;
    let expected_basis_binding_digest = WorthQueryMutationEvidenceDigest::source_identity(
        "continuity-basis-binding",
        &expected_binding_identity,
    )?;
    assert_eq!(
        preserved
            .basis_binding_digest()
            .map(|digest| digest.as_str()),
        Some(expected_basis_binding_digest.as_str())
    );
    assert_eq!(
        preserved
            .successor_authoritative_identity()
            .map(|identity| identity.as_str()),
        Some("edge:14")
    );

    assert_eq!(
        split.outcome_class(),
        crate::runtime::WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors
    );
    assert_eq!(
        split
            .successor_authoritative_identities()
            .iter()
            .map(|identity| identity.as_str())
            .collect::<Vec<_>>(),
        vec!["edge:14", "edge:15"]
    );

    Ok(())
}

#[test]
fn continuity_runtime_materializer_denies_correspondence_only(
) -> Result<(), IdentityDerivationDenial> {
    let outcome = materialize_runtime_continuity_evidence(ready_continuity(
        WorthQueryContinuityContributionAuthoring::correspondence_only(
            "continuity.correspondence_only",
            "semantic correspondence exists without authoritative continuity",
        )?,
    )?)?;

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics
    ));

    Ok(())
}

#[test]
fn continuity_runtime_materializer_denies_inconsistent_runtime_semantics(
) -> Result<(), IdentityDerivationDenial> {
    let outcome = materialize_runtime_continuity_evidence(ready_continuity_payload(
        super::WorthQueryContinuityContributionPayload::with_runtime_semantics(
            super::WorthQueryContinuityContributionPosture::Preserved,
            "continuity.identity.preserved",
            "posture claims preserved while runtime semantics claim split",
            Some(super::WorthQueryContinuityRuntimeSemantics::new(
                crate::runtime::WorthQueryContinuityMutationFamily::SplitExistingTarget,
                crate::runtime::WorthQueryContinuityMutationOutcomeClass::ContinuesAsSplitSuccessors,
                "edge:12",
                ["edge:14", "edge:15"],
            )?),
        )?,
    )?)?;

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::WorthQueryDomainCapabilityProgressionDenialKind::InconsistentCanonicalMaterializationSemantics
    ));

    Ok(())
}

fn ready_admission(
    authoring: WorthQueryAdmissionContributionAuthoring,
) -> Result<
    super::WorthQueryMaterializationReadyAdmissionContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready(authoring.bind_to_admitted_plan_target(
        admitted_plan_target("plan-admission")?,
    ))?)
}

fn ready_support(
    authoring: WorthQuerySupportContributionAuthoring,
) -> Result<
    super::WorthQueryMaterializationReadySupportContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready(authoring.bind_to_admitted_plan_target(
        admitted_plan_target("plan-admission")?,
    ))?)
}

fn ready_continuity(
    authoring: WorthQueryContinuityContributionAuthoring,
) -> Result<
    super::WorthQueryMaterializationReadyContinuityContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready_continuity_payload(
        authoring
            .bind_to_admitted_plan_target(admitted_plan_target("plan-continuity")?)
            .payload()
            .payload()
            .clone(),
    )?)
}

fn ready_continuity_payload(
    payload: super::WorthQueryContinuityContributionPayload,
) -> Result<
    super::WorthQueryMaterializationReadyContinuityContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready(
        super::proof_integration::create_requested_domain_capability_contribution(
            admitted_plan_target("plan-continuity")?,
            payload,
        )?,
    )?)
}
