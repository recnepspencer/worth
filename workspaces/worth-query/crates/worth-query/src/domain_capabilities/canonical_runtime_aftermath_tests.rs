use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use super::targets::WorthQueryAdmittedPlanBoundContributionTarget;
use super::test_support::{admitted_plan_target, ready, success};
use super::{
    materialize_admitted_projection_consumption, materialize_projection_consumption_contract,
    materialize_projection_consumption_eligibility, materialize_projection_consumption_review,
    materialize_projection_consumption_support_report, WorthQueryAftermathContributionAuthoring,
};
use crate::projection_consumption::{
    ProjectionConsumptionBindingContext, ProjectionConsumptionEligibility,
    ProjectionConsumptionSource, ProjectionConsumptionSupportPosture,
    ProjectionConsumptionWarningKind, ProjectionSourceFamily,
};

#[test]
fn aftermath_runtime_materializer_builds_projection_consumption(
) -> Result<(), IdentityDerivationDenial> {
    let admitted = success(materialize_admitted_projection_consumption(
        ready_aftermath(
            WorthQueryAftermathContributionAuthoring::establishes_projection_contract(
                "aftermath.projection.contract",
                "projection contract should bind to an admitted plan",
                admitted_projection_source(),
                admitted_projection_binding(),
                crate::projection_consumption::ProjectMaterializedFacts::declare()
                    .display_field_path(
                        crate::projection_consumption::projection_fact_field_path_from_segments([
                            worth_foundational::facade::FieldKey::new("field")
                                .expect("projection fact field segment should admit"),
                            worth_foundational::facade::FieldKey::new("visible")
                                .expect("projection fact field segment should admit"),
                        ]),
                    ),
            )?,
        )?,
    )?);

    let contract = success(materialize_projection_consumption_contract(
        ready_aftermath(
            WorthQueryAftermathContributionAuthoring::consumes_projection_contract(
                "aftermath.projection.contract",
                "projection contract should materialize a stable consequence contract",
                admitted_projection_source(),
                admitted_projection_binding(),
                crate::projection_consumption::ProjectMaterializedFacts::declare()
                    .display_field_path(
                        crate::projection_consumption::projection_fact_field_path_from_segments([
                            worth_foundational::facade::FieldKey::new("field")
                                .expect("projection fact field segment should admit"),
                            worth_foundational::facade::FieldKey::new("visible")
                                .expect("projection fact field segment should admit"),
                        ]),
                    ),
            )?,
        )?,
    )?);

    assert!(!admitted.eligibility_digest().is_empty());
    assert_eq!(
        contract.source_family(),
        ProjectionSourceFamily::QueryReadReceipt
    );
    assert_eq!(contract.source_identity(), "query-read:domain-capability");
    assert!(!contract.contract_digest().is_empty());

    Ok(())
}

#[test]
fn aftermath_runtime_materializer_denies_missing_semantics() -> Result<(), IdentityDerivationDenial>
{
    let outcome = materialize_admitted_projection_consumption(ready_aftermath(
        WorthQueryAftermathContributionAuthoring::establishes_fact(
            "aftermath.support.only",
            "support-only aftermath cannot mint projection consumption",
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
fn aftermath_runtime_materializer_denies_invalid_projection_declaration(
) -> Result<(), IdentityDerivationDenial> {
    let outcome = materialize_projection_consumption_eligibility(ready_aftermath(
        WorthQueryAftermathContributionAuthoring::establishes_projection_contract(
            "aftermath.projection.invalid_declaration",
            "invalid declaration inputs should deny before eligibility materialization",
            admitted_projection_source(),
            invalid_projection_binding(),
            crate::projection_consumption::ProjectMaterializedFacts::declare().display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("field")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("visible")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
        )?,
    )?)?;

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture
    ));

    Ok(())
}

#[test]
fn aftermath_runtime_materializer_preserves_warning_bearing_eligibility(
) -> Result<(), IdentityDerivationDenial> {
    let eligibility = success(materialize_projection_consumption_eligibility(
        ready_aftermath(
            WorthQueryAftermathContributionAuthoring::establishes_projection_contract(
                "aftermath.projection.warning",
                "query-context display-field aftermath should preserve warnings",
                warning_projection_source(),
                warning_projection_binding(),
                crate::projection_consumption::ProjectMaterializedFacts::declare()
                    .display_field_path(
                        crate::projection_consumption::projection_fact_field_path_from_segments([
                            worth_foundational::facade::FieldKey::new("field")
                                .expect("projection fact field segment should admit"),
                            worth_foundational::facade::FieldKey::new("visible")
                                .expect("projection fact field segment should admit"),
                        ]),
                    ),
            )?,
        )?,
    )?);

    match eligibility {
        ProjectionConsumptionEligibility::AdmittedWithWarnings(_, warnings) => {
            assert_eq!(
                warnings.warning_kinds(),
                [ProjectionConsumptionWarningKind::QueryContextRowBound]
            );
        }
        other => panic!("expected warning-bearing eligibility, got {other:?}"),
    }

    Ok(())
}

#[test]
fn aftermath_runtime_materializer_preserves_deferred_eligibility(
) -> Result<(), IdentityDerivationDenial> {
    let eligibility = success(materialize_projection_consumption_eligibility(
        ready_aftermath(
            WorthQueryAftermathContributionAuthoring::establishes_projection_contract(
                "aftermath.projection.deferred",
                "write-receipt target identity should remain deferred in the canonical adapter",
                deferred_projection_source(),
                deferred_projection_binding(),
                crate::projection_consumption::ProjectMaterializedFacts::declare()
                    .target_identity(),
            )?,
        )?,
    )?);

    assert!(matches!(
        eligibility,
        ProjectionConsumptionEligibility::Deferred(_)
    ));

    Ok(())
}

#[test]
fn aftermath_runtime_materializer_preserves_source_mismatch_eligibility(
) -> Result<(), IdentityDerivationDenial> {
    let eligibility = success(materialize_projection_consumption_eligibility(
        ready_aftermath(
            WorthQueryAftermathContributionAuthoring::establishes_projection_contract(
                "aftermath.projection.source_mismatch",
                "membership requests should remain source mismatches for read receipts",
                admitted_projection_source(),
                admitted_projection_binding(),
                crate::projection_consumption::ProjectMaterializedFacts::declare().memberships(),
            )?,
        )?,
    )?);

    match eligibility {
        ProjectionConsumptionEligibility::SourceMismatch(mismatch) => {
            assert_eq!(
                mismatch.source_family(),
                ProjectionSourceFamily::QueryReadReceipt
            );
            assert_eq!(
                mismatch.requested_fact_kind(),
                crate::projection_consumption::ProjectionFactKind::Membership
            );
        }
        other => panic!("expected source mismatch eligibility, got {other:?}"),
    }

    Ok(())
}

#[test]
fn aftermath_runtime_review_and_support_hook_preserve_deferred_truth(
) -> Result<(), IdentityDerivationDenial> {
    let review_contribution = ready_aftermath(
        WorthQueryAftermathContributionAuthoring::establishes_projection_contract(
            "aftermath.projection.deferred",
            "write-receipt target identity should remain deferred in the checked lane",
            deferred_projection_source(),
            deferred_projection_binding(),
            crate::projection_consumption::ProjectMaterializedFacts::declare().target_identity(),
        )?,
    )?;
    let support_contribution = ready_aftermath(
        WorthQueryAftermathContributionAuthoring::establishes_projection_contract(
            "aftermath.projection.deferred",
            "write-receipt target identity should remain deferred in the checked lane",
            deferred_projection_source(),
            deferred_projection_binding(),
            crate::projection_consumption::ProjectMaterializedFacts::declare().target_identity(),
        )?,
    )?;

    let review = success(materialize_projection_consumption_review(
        review_contribution,
    )?);
    let support = success(materialize_projection_consumption_support_report(
        support_contribution,
    )?);

    assert_eq!(review.semantic_code(), "aftermath.projection.deferred");
    assert_eq!(
        review.declaration().source().family(),
        ProjectionSourceFamily::QueryWriteReceipt
    );
    assert!(matches!(
        review.eligibility(),
        ProjectionConsumptionEligibility::Deferred(_)
    ));
    assert_eq!(
        support.source_family(),
        ProjectionSourceFamily::QueryWriteReceipt
    );
    assert!(support.rows().iter().any(|row| {
        row.fact_kind() == crate::projection_consumption::ProjectionFactKind::TargetIdentity
            && matches!(
                row.posture(),
                ProjectionConsumptionSupportPosture::Deferred(_)
            )
    }));

    Ok(())
}

fn ready_aftermath(
    authoring: WorthQueryAftermathContributionAuthoring,
) -> Result<
    super::WorthQueryMaterializationReadyAftermathContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    ready(
        authoring
            .bind_to_admitted_plan_target(admitted_plan_target("plan-aftermath")?)
            .expect("fixture digest must fit"),
    )
}

fn admitted_projection_source() -> ProjectionConsumptionSource {
    ProjectionConsumptionSource::test_only(
        ProjectionSourceFamily::QueryReadReceipt,
        Some("query-digest:domain-capability"),
        Some("basis-digest:domain-capability"),
        Some("result-digest:domain-capability"),
        Some("shape-digest:domain-capability"),
        "query-read:domain-capability",
    )
}

fn warning_projection_source() -> ProjectionConsumptionSource {
    ProjectionConsumptionSource::test_only(
        ProjectionSourceFamily::QueryContextExecution,
        Some("query-digest:domain-capability"),
        Some("basis-digest:domain-capability"),
        Some("result-digest:domain-capability"),
        Some("shape-digest:domain-capability"),
        "query-context:domain-capability",
    )
}

fn deferred_projection_source() -> ProjectionConsumptionSource {
    ProjectionConsumptionSource::test_only(
        ProjectionSourceFamily::QueryWriteReceipt,
        None,
        Some("basis-digest:domain-capability"),
        None,
        None,
        "query-write:domain-capability",
    )
}

fn admitted_projection_binding() -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::intent_admission_certification_binding(
        "shape-digest:domain-capability",
        "query-digest:domain-capability",
        "shape-digest:domain-capability",
        "authorized-projection:domain-capability",
        "narrowed-shape-digest:domain-capability",
        "policy-digest:domain-capability",
        "tenant-schema-digest:domain-capability",
        crate::projection_consumption::test_authorized_field_paths(&["field.visible"]),
    )
}

fn warning_projection_binding() -> ProjectionConsumptionBindingContext {
    admitted_projection_binding()
}

fn deferred_projection_binding() -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::test_only(
        "result-shape:test",
        "authorized-projection:domain-capability",
        crate::projection_consumption::test_authorized_field_paths(&["identity.id"]),
    )
}

fn invalid_projection_binding() -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::test_only(
        "shape-drift:domain-capability",
        "authorized-projection:domain-capability",
        crate::projection_consumption::test_authorized_field_paths(&["field.visible"]),
    )
}
