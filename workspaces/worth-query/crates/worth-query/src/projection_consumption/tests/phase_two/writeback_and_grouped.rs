use super::super::super::source::{
    ProjectionSourceCapabilityProfile, ProjectionWriteReceiptCapabilities,
};
use super::super::super::{
    declare_projection_consumption, discover_projection_consumption_support,
    evaluate_projection_consumption_eligibility, ProjectMaterializedFacts,
    ProjectionConsumptionBindingContext, ProjectionConsumptionEligibility,
    ProjectionConsumptionSource, ProjectionConsumptionSupportPosture, ProjectionSourceFamily,
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

#[test]
fn write_receipt_admits_target_source_reference_and_effect_aftermath_when_evidence_exists() {
    let source = source_with_profile(
        ProjectionSourceFamily::QueryWriteReceipt,
        ProjectionSourceCapabilityProfile::QueryWriteReceipt {
            capabilities: ProjectionWriteReceiptCapabilities::test_only(true, true, true, true),
        },
    );
    let report = discover_projection_consumption_support(&source).expect("fixture digest must fit");

    for fact_kind in [
        super::super::super::ProjectionFactKind::TargetIdentity,
        super::super::super::ProjectionFactKind::SourceReference,
        super::super::super::ProjectionFactKind::EffectContinuity,
        super::super::super::ProjectionFactKind::RelationEndpoint,
    ] {
        let row = report
            .rows()
            .iter()
            .find(|row| row.fact_kind() == fact_kind)
            .expect("support row should exist");
        assert_eq!(
            row.posture(),
            &ProjectionConsumptionSupportPosture::Admitted
        );
    }

    let declaration = declare_projection_consumption(
        source,
        test_binding(&[]),
        ProjectMaterializedFacts::declare()
            .target_identity()
            .source_references()
            .effect_continuity_facts()
            .relation_endpoints(),
    )
    .expect("write receipt declaration should be valid");

    assert!(matches!(
        evaluate_projection_consumption_eligibility(&declaration),
        ProjectionConsumptionEligibility::Admitted(_)
    ));
}

#[test]
fn write_receipt_effect_aftermath_stays_deferred_without_carried_evidence() {
    let source = source_with_profile(
        ProjectionSourceFamily::QueryWriteReceipt,
        ProjectionSourceCapabilityProfile::QueryWriteReceipt {
            capabilities: ProjectionWriteReceiptCapabilities::default(),
        },
    );
    let declaration = declare_projection_consumption(
        source,
        test_binding(&[]),
        ProjectMaterializedFacts::declare().effect_continuity_facts(),
    )
    .expect("write receipt declaration should be valid");

    assert!(matches!(
        evaluate_projection_consumption_eligibility(&declaration),
        ProjectionConsumptionEligibility::Deferred(_)
    ));
}

#[test]
fn write_receipt_source_reference_stays_deferred_without_provenance_or_symbolic_evidence() {
    let source = source_with_profile(
        ProjectionSourceFamily::QueryWriteReceipt,
        ProjectionSourceCapabilityProfile::QueryWriteReceipt {
            capabilities: ProjectionWriteReceiptCapabilities::test_only(false, false, false, false),
        },
    );
    let declaration = declare_projection_consumption(
        source,
        test_binding(&[]),
        ProjectMaterializedFacts::declare().source_references(),
    )
    .expect("write receipt declaration should be valid");

    assert!(matches!(
        evaluate_projection_consumption_eligibility(&declaration),
        ProjectionConsumptionEligibility::Deferred(_)
    ));
}

#[test]
fn relational_row_set_admits_identity_and_field_backed_fact_families() {
    let declaration = declare_projection_consumption(
        ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::RelationalRowSet,
            None,
            Some("snapshot:test"),
            None,
            None,
            "relational-row-set:test",
        ),
        test_binding(&["identity.id", "profile.display_name"]),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .view_local_identities()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            )
            .derived_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
    )
    .expect("relational row set declaration should be valid");

    assert!(matches!(
        evaluate_projection_consumption_eligibility(&declaration),
        ProjectionConsumptionEligibility::Admitted(_)
    ));
}

#[test]
fn grouped_sources_admit_membership_and_view_local_identity_but_not_entity_identity() {
    for family in [
        ProjectionSourceFamily::RelationalGroupedProjection,
        ProjectionSourceFamily::BridgeGroupedTruthView,
    ] {
        let grouped_declaration = declare_projection_consumption(
            ProjectionConsumptionSource::test_only(
                family,
                None,
                Some("snapshot:test"),
                None,
                None,
                "grouped:test",
            ),
            test_binding(&["identity.id"]),
            ProjectMaterializedFacts::declare()
                .memberships()
                .view_local_identities()
                .relation_endpoints(),
        )
        .expect("grouped declaration should be valid");
        assert!(matches!(
            evaluate_projection_consumption_eligibility(&grouped_declaration),
            ProjectionConsumptionEligibility::Admitted(_)
        ));

        let identity_declaration = declare_projection_consumption(
            ProjectionConsumptionSource::test_only(
                family,
                None,
                Some("snapshot:test"),
                None,
                None,
                "grouped:test",
            ),
            test_binding(&["identity.id"]),
            ProjectMaterializedFacts::declare().entity_identities(),
        )
        .expect("identity declaration should be valid");
        match evaluate_projection_consumption_eligibility(&identity_declaration)
            .expect("fixture digest must fit")
        {
            ProjectionConsumptionEligibility::SourceMismatch(mismatch) => {
                assert_eq!(mismatch.source_family(), family);
                assert_eq!(
                    mismatch.requested_fact_kind(),
                    super::super::super::ProjectionFactKind::EntityIdentity
                );
            }
            other => panic!("expected entity-identity mismatch for {family:?}, got {other:?}"),
        }
    }
}

#[test]
fn bridge_truth_view_row_set_matches_relational_row_set_admission_surface() {
    let declaration = declare_projection_consumption(
        ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::BridgeTruthViewRowSet,
            None,
            Some("snapshot:test"),
            None,
            None,
            "bridge-row-set:test",
        ),
        test_binding(&["identity.id", "profile.display_name"]),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .view_local_identities()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            )
            .derived_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
    )
    .expect("bridge row set declaration should be valid");

    assert!(matches!(
        evaluate_projection_consumption_eligibility(&declaration),
        ProjectionConsumptionEligibility::Admitted(_)
    ));
}
