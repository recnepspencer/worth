use super::super::super::source::{
    ProjectionSourceCapabilityProfile, ProjectionSourceExecutionPosture,
    ProjectionSourceReferenceIdentity, ProjectionWriteReceiptCapabilities,
};
use super::super::super::{
    declare_projection_consumption, discover_projection_consumption_support,
    evaluate_projection_consumption_eligibility, ProjectMaterializedFacts,
    ProjectionConsumptionBindingContext, ProjectionConsumptionEligibility,
    ProjectionConsumptionSource, ProjectionConsumptionSupportPosture,
    ProjectionConsumptionWarningKind, ProjectionFactKind, ProjectionSourceFamily,
};

fn test_binding(visible_fields: &[&str]) -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::test_only(
        "result-shape:test",
        "authorized-projection:test",
        crate::projection_consumption::test_authorized_field_paths(visible_fields),
    )
}

fn source_with_profile(
    family: ProjectionSourceFamily,
    capability_profile: ProjectionSourceCapabilityProfile,
) -> ProjectionConsumptionSource {
    ProjectionConsumptionSource::test_only_with_profile(
        family,
        capability_profile,
        Some("query:test"),
        Some("basis:test"),
        Some("result:test"),
        Some("result-shape:test"),
        "source:test",
    )
}

fn query_context_source_with_references(
    execution_posture: ProjectionSourceExecutionPosture,
) -> ProjectionConsumptionSource {
    ProjectionConsumptionSource::test_only_with_source_references(
        ProjectionSourceFamily::QueryContextExecution,
        ProjectionSourceCapabilityProfile::QueryContextExecution { execution_posture },
        Some("query:test"),
        Some("basis:test"),
        Some("result:test"),
        Some("result-shape:test"),
        "query-context:test",
        vec![ProjectionSourceReferenceIdentity::test_only(
            "query_context_materialization_path",
            "materialization-path:test",
        )],
    )
}

fn request_for_kind(kind: ProjectionFactKind) -> ProjectMaterializedFacts {
    match kind {
        ProjectionFactKind::EntityIdentity => {
            ProjectMaterializedFacts::declare().entity_identities()
        }
        ProjectionFactKind::ViewLocalIdentity => {
            ProjectMaterializedFacts::declare().view_local_identities()
        }
        ProjectionFactKind::TargetIdentity => ProjectMaterializedFacts::declare().target_identity(),
        ProjectionFactKind::SourceReference => {
            ProjectMaterializedFacts::declare().source_references()
        }
        ProjectionFactKind::EffectContinuity => {
            ProjectMaterializedFacts::declare().effect_continuity_facts()
        }
        ProjectionFactKind::Membership => ProjectMaterializedFacts::declare().memberships(),
        ProjectionFactKind::RelationEndpoint => {
            ProjectMaterializedFacts::declare().relation_endpoints()
        }
        ProjectionFactKind::DisplayField => ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
        ProjectionFactKind::DerivedField => ProjectMaterializedFacts::declare().derived_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    }
}

fn visible_fields_for_kind(kind: ProjectionFactKind) -> Vec<&'static str> {
    match kind {
        ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField => {
            vec!["profile.display_name"]
        }
        _ => vec!["identity.id"],
    }
}

#[test]
fn historical_query_context_identity_requests_fail_as_source_mismatch() {
    let declaration = declare_projection_consumption(
        source_with_profile(
            ProjectionSourceFamily::QueryContextExecution,
            ProjectionSourceCapabilityProfile::QueryContextExecution {
                execution_posture: ProjectionSourceExecutionPosture::Historical,
            },
        ),
        test_binding(&["identity.id"]),
        ProjectMaterializedFacts::declare().entity_identities(),
    )
    .expect("historical query-context declaration should be valid");

    match evaluate_projection_consumption_eligibility(&declaration)
        .expect("fixture digest must fit")
    {
        ProjectionConsumptionEligibility::SourceMismatch(mismatch) => {
            assert_eq!(
                mismatch.requested_fact_kind(),
                ProjectionFactKind::EntityIdentity
            );
            assert_eq!(
                mismatch.source_family(),
                ProjectionSourceFamily::QueryContextExecution
            );
        }
        other => panic!("expected historical source mismatch, got {other:?}"),
    }
}

#[test]
fn non_preview_query_context_display_fields_warn_as_row_bound_not_preview_derived() {
    let declaration = declare_projection_consumption(
        source_with_profile(
            ProjectionSourceFamily::QueryContextExecution,
            ProjectionSourceCapabilityProfile::QueryContextExecution {
                execution_posture: ProjectionSourceExecutionPosture::Historical,
            },
        ),
        test_binding(&["profile.display_name"]),
        ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    )
    .expect("historical query-context display declaration should be valid");

    match evaluate_projection_consumption_eligibility(&declaration)
        .expect("fixture digest must fit")
    {
        ProjectionConsumptionEligibility::AdmittedWithWarnings(_, warnings) => {
            assert_eq!(
                warnings.warning_kinds(),
                &[ProjectionConsumptionWarningKind::QueryContextRowBound]
            );
        }
        other => panic!("expected row-bound warning, got {other:?}"),
    }
}

#[test]
fn query_context_source_reference_requires_bound_source_reference_evidence() {
    let admitted_declaration = declare_projection_consumption(
        query_context_source_with_references(ProjectionSourceExecutionPosture::Current),
        test_binding(&["identity.id"]),
        ProjectMaterializedFacts::declare().source_references(),
    )
    .expect("query-context source-reference declaration should be valid");

    assert!(matches!(
        evaluate_projection_consumption_eligibility(&admitted_declaration)
            .expect("fixture digest must fit"),
        ProjectionConsumptionEligibility::AdmittedWithWarnings(_, _)
    ));

    let denied_declaration = declare_projection_consumption(
        source_with_profile(
            ProjectionSourceFamily::QueryContextExecution,
            ProjectionSourceCapabilityProfile::QueryContextExecution {
                execution_posture: ProjectionSourceExecutionPosture::Current,
            },
        ),
        test_binding(&["identity.id"]),
        ProjectMaterializedFacts::declare().source_references(),
    )
    .expect("query-context declaration should be valid");

    assert!(matches!(
        evaluate_projection_consumption_eligibility(&denied_declaration)
            .expect("fixture digest must fit"),
        ProjectionConsumptionEligibility::SourceMismatch(_)
    ));
}

#[test]
fn support_discovery_and_eligibility_stay_in_sync_for_first_slice_source_instances() {
    let sources = vec![
        ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::QueryReadReceipt,
            Some("query:test"),
            Some("basis:test"),
            Some("result:test"),
            Some("result-shape:test"),
            "read:test",
        ),
        source_with_profile(
            ProjectionSourceFamily::QueryWriteReceipt,
            ProjectionSourceCapabilityProfile::QueryWriteReceipt {
                capabilities: ProjectionWriteReceiptCapabilities::test_only(true, true, true, true),
            },
        ),
        ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::QueryContextExecution,
            Some("query:test"),
            Some("basis:test"),
            Some("result:test"),
            Some("result-shape:test"),
            "query-context:test",
        ),
        ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::RelationalRowSet,
            None,
            Some("snapshot:test"),
            None,
            None,
            "relational-row-set:test",
        ),
        ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::RelationalGroupedProjection,
            None,
            Some("snapshot:test"),
            None,
            None,
            "relational-grouped:test",
        ),
        ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::BridgeTruthViewRowSet,
            None,
            Some("snapshot:test"),
            None,
            None,
            "bridge-row-set:test",
        ),
        ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::BridgeGroupedTruthView,
            None,
            Some("snapshot:test"),
            None,
            None,
            "bridge-grouped:test",
        ),
    ];

    for source in sources {
        let report =
            discover_projection_consumption_support(&source).expect("fixture digest must fit");
        for fact_kind in ProjectionFactKind::all().iter().copied() {
            let row = report
                .rows()
                .iter()
                .find(|row| row.fact_kind() == fact_kind)
                .expect("support row should exist");
            let declaration = declare_projection_consumption(
                source.clone(),
                test_binding(&visible_fields_for_kind(fact_kind)),
                request_for_kind(fact_kind),
            )
            .expect("declaration should be valid");
            let eligibility = evaluate_projection_consumption_eligibility(&declaration)
                .expect("fixture digest must fit");

            match (row.posture(), eligibility) {
                (
                    ProjectionConsumptionSupportPosture::Admitted,
                    ProjectionConsumptionEligibility::Admitted(_),
                ) => {}
                (
                    ProjectionConsumptionSupportPosture::AdmittedWithWarnings(_),
                    ProjectionConsumptionEligibility::AdmittedWithWarnings(_, _),
                ) => {}
                (
                    ProjectionConsumptionSupportPosture::Deferred(_),
                    ProjectionConsumptionEligibility::Deferred(_),
                ) => {}
                (
                    ProjectionConsumptionSupportPosture::SourceMismatch,
                    ProjectionConsumptionEligibility::SourceMismatch(_),
                ) => {}
                (posture, other) => panic!(
                    "support posture and eligibility diverged for source {:?} fact {:?}: {:?} vs {:?}",
                    source.family(),
                    fact_kind,
                    posture,
                    other
                ),
            }
        }
    }
}
