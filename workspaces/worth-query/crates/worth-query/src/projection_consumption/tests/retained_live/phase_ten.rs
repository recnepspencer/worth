use crate::projection_consumption::{
    evaluate_projection_consumption_eligibility, ProjectMaterializedFacts,
    ProjectionConsumptionEligibility, ProjectionConsumptionSupportPosture,
    ProjectionContractSourcePosture, ProjectionFactKind, ProjectionSourceFamily,
};

use super::support::{
    authorized_projection, live_binding, retained_binding, shared_test_result_shape,
    test_result_shape_artifact, test_result_shape_canonical_digest,
};

#[test]
fn retained_binding_support_is_first_class_and_family_specific() {
    let binding = retained_binding();

    let support = binding
        .discover_projection_fact_consumption_support()
        .expect("fixture digest must fit");

    assert_eq!(
        support.source_family(),
        ProjectionSourceFamily::RetainedDerivedArtifactBinding
    );
    assert_eq!(support.rows().len(), 9);
    assert!(support.rows().iter().all(|row| {
        row.source_family() == ProjectionSourceFamily::RetainedDerivedArtifactBinding
    }));
    assert!(matches!(
        support
            .rows()
            .iter()
            .find(|row| row.fact_kind() == ProjectionFactKind::ViewLocalIdentity)
            .expect("view local identity row should exist")
            .posture(),
        ProjectionConsumptionSupportPosture::Admitted
    ));
    assert!(matches!(
        support
            .rows()
            .iter()
            .find(|row| row.fact_kind() == ProjectionFactKind::EntityIdentity)
            .expect("entity identity row should exist")
            .posture(),
        ProjectionConsumptionSupportPosture::SourceMismatch
    ));
}

#[test]
fn retained_binding_declaration_preserves_binding_identity_and_target_refs() {
    let binding = retained_binding();
    let declaration = binding
        .declare_projection_fact_consumption(
            &test_result_shape_artifact("result-shape:test"),
            &authorized_projection(
                "query:test",
                &test_result_shape_canonical_digest("result-shape:test"),
                &["profile.display_name"],
            ),
            ProjectMaterializedFacts::declare().display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
        )
        .expect("retained binding declaration should succeed");

    assert_eq!(
        declaration.source().family(),
        ProjectionSourceFamily::RetainedDerivedArtifactBinding
    );
    assert_eq!(declaration.source().query_digest(), None);
    assert_eq!(
        declaration.source().basis_digest(),
        Some(binding.snapshot_identity().evidence_identity().as_str())
    );
    assert_eq!(
        declaration.source().source_identity(),
        binding.binding_for_reporting()
    );
    assert_eq!(
        declaration
            .source()
            .source_reference_identities()
            .iter()
            .map(|identity| (identity.label(), identity.identity()))
            .collect::<Vec<_>>(),
        vec![
            ("retained_target_view", "derived.first"),
            ("retained_target_view", "derived.second"),
        ]
    );
}

#[test]
fn live_binding_eligibility_admits_honest_fact_families() {
    let binding = live_binding();
    let result_shape = shared_test_result_shape();
    let declaration = binding
        .declare_projection_fact_consumption(
            &result_shape.identity,
            &authorized_projection(
                "query:test",
                &result_shape.digest,
                &["profile.display_name"],
            ),
            ProjectMaterializedFacts::declare()
                .entity_identities()
                .display_field_path(
                    crate::projection_consumption::projection_fact_field_path_from_segments([
                        worth_foundational::facade::FieldKey::new("profile")
                            .expect("projection fact field segment should admit"),
                        worth_foundational::facade::FieldKey::new("display_name")
                            .expect("projection fact field segment should admit"),
                    ]),
                )
                .source_references(),
        )
        .expect("live binding declaration should succeed");

    match evaluate_projection_consumption_eligibility(&declaration)
        .expect("fixture digest must fit")
    {
        ProjectionConsumptionEligibility::Admitted(admitted) => {
            assert_eq!(
                admitted.declaration().source().family(),
                ProjectionSourceFamily::LiveArtifactBinding
            );
        }
        other => panic!("expected admitted eligibility, got {other:?}"),
    }
}

#[test]
fn live_binding_support_reports_live_family_not_legacy_escape_hatch() {
    let binding = live_binding();

    let support = binding
        .discover_projection_fact_consumption_support()
        .expect("fixture digest must fit");

    assert_eq!(
        support.source_family(),
        ProjectionSourceFamily::LiveArtifactBinding
    );
    assert!(matches!(
        support
            .rows()
            .iter()
            .find(|row| row.fact_kind() == ProjectionFactKind::EntityIdentity)
            .expect("entity identity row should exist")
            .posture(),
        ProjectionConsumptionSupportPosture::Admitted
    ));
    assert!(
        support
            .rows()
            .iter()
            .all(|row| !row.support_digest().is_empty()),
        "support digests should stay machine-addressable"
    );
}

#[test]
fn retained_and_live_contract_postures_are_named_as_distinct_authority_kinds() {
    assert_eq!(
        ProjectionContractSourcePosture::RetainedArtifactBindingSource.as_str(),
        "retained_artifact_binding_source"
    );
    assert_eq!(
        ProjectionContractSourcePosture::LiveArtifactBindingSource.as_str(),
        "live_artifact_binding_source"
    );
}
