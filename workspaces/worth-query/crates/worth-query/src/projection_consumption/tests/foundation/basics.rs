use super::super::super::{
    declare_projection_consumption, discover_projection_consumption_support,
    evaluate_projection_consumption_eligibility, ProjectMaterializedFacts,
    ProjectionConsumptionBindingContext, ProjectionConsumptionDeclarationError,
    ProjectionConsumptionDenialReason, ProjectionConsumptionEligibility,
    ProjectionConsumptionSupportPosture, ProjectionConsumptionWarningKind, ProjectionFactKind,
    ProjectionSourceFamily,
};
use super::support::{test_binding, test_binding_with_projection_metadata, test_source};

#[test]
fn equivalent_declarations_share_digest() {
    let requested = ProjectMaterializedFacts::declare()
        .entity_identities()
        .display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
            ]),
        );

    let left = declare_projection_consumption(
        test_source(ProjectionSourceFamily::QueryReadReceipt),
        test_binding(&["identity.id", "profile.display_name"]),
        requested.clone(),
    )
    .expect("equivalent declaration should be valid");
    let right = declare_projection_consumption(
        test_source(ProjectionSourceFamily::QueryReadReceipt),
        test_binding(&["identity.id", "profile.display_name"]),
        requested,
    )
    .expect("equivalent declaration should be valid");

    assert_eq!(left.declaration_digest(), right.declaration_digest());
}

#[test]
fn read_receipt_admits_visible_identity_and_display_requests() {
    let declaration = declare_projection_consumption(
        test_source(ProjectionSourceFamily::QueryReadReceipt),
        test_binding(&["identity.id", "profile.display_name"]),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
    )
    .expect("read-backed declaration should be valid");

    let eligibility = evaluate_projection_consumption_eligibility(&declaration);
    match eligibility {
        ProjectionConsumptionEligibility::Admitted(admitted) => {
            assert!(!admitted.eligibility_digest().is_empty());
        }
        other => panic!("unexpected eligibility posture: {other:?}"),
    }
}

#[test]
fn masked_display_field_denies_before_admission() {
    let declaration = declare_projection_consumption(
        test_source(ProjectionSourceFamily::QueryReadReceipt),
        test_binding(&["identity.id"]),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
    )
    .expect("masked field declaration should still be structurally valid");

    let eligibility = evaluate_projection_consumption_eligibility(&declaration);
    match eligibility {
        ProjectionConsumptionEligibility::Denied(denied) => {
            assert_eq!(
                denied.reason(),
                &ProjectionConsumptionDenialReason::FactFamilyNotVisible {
                    field_key: "profile.display_name".to_string(),
                }
            );
            assert_eq!(denied.counters().denied_count(), 1);
        }
        other => panic!("unexpected eligibility posture: {other:?}"),
    }
}

#[test]
fn read_receipt_membership_request_returns_source_mismatch() {
    let declaration = declare_projection_consumption(
        test_source(ProjectionSourceFamily::QueryReadReceipt),
        test_binding(&["identity.id"]),
        ProjectMaterializedFacts::declare().memberships(),
    )
    .expect("membership declaration should be structurally valid");

    let eligibility = evaluate_projection_consumption_eligibility(&declaration);
    match eligibility {
        ProjectionConsumptionEligibility::SourceMismatch(mismatch) => {
            assert_eq!(
                mismatch.requested_fact_kind(),
                ProjectionFactKind::Membership
            );
            assert_eq!(
                mismatch.source_family(),
                ProjectionSourceFamily::QueryReadReceipt
            );
        }
        other => panic!("unexpected eligibility posture: {other:?}"),
    }
}

#[test]
fn write_receipt_target_identity_is_deferred_for_now() {
    let declaration = declare_projection_consumption(
        test_source(ProjectionSourceFamily::QueryWriteReceipt),
        test_binding(&[]),
        ProjectMaterializedFacts::declare().target_identity(),
    )
    .expect("write receipt declaration should be structurally valid");

    let eligibility = evaluate_projection_consumption_eligibility(&declaration);
    assert!(matches!(
        eligibility,
        ProjectionConsumptionEligibility::Deferred(_)
    ));
}

#[test]
fn query_context_support_and_admission_share_warning_posture() {
    let source = test_source(ProjectionSourceFamily::QueryContextExecution);
    let report = discover_projection_consumption_support(&source).expect("fixture digest must fit");
    let row = report
        .rows()
        .iter()
        .find(|row| row.fact_kind() == ProjectionFactKind::DerivedField)
        .expect("derived scalar support row should exist");
    assert!(matches!(
        row.posture(),
        ProjectionConsumptionSupportPosture::AdmittedWithWarnings(
            ProjectionConsumptionWarningKind::QueryContextRowBound
        )
    ));

    let declaration = declare_projection_consumption(
        source,
        test_binding(&["profile.display_name"]),
        ProjectMaterializedFacts::declare().derived_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    )
    .expect("query-context declaration should be valid");
    let eligibility = evaluate_projection_consumption_eligibility(&declaration);
    match eligibility {
        ProjectionConsumptionEligibility::AdmittedWithWarnings(_, warnings) => {
            assert_eq!(
                warnings.warning_kinds(),
                [ProjectionConsumptionWarningKind::QueryContextRowBound]
            );
        }
        other => panic!("unexpected eligibility posture: {other:?}"),
    }
}

#[test]
fn empty_projection_fact_request_is_rejected_before_declaration_exists() {
    let declaration = declare_projection_consumption(
        test_source(ProjectionSourceFamily::QueryReadReceipt),
        test_binding(&["identity.id"]),
        ProjectMaterializedFacts::declare(),
    );

    assert_eq!(
        declaration,
        Err(ProjectionConsumptionDeclarationError::NoRequestedFacts)
    );
}

#[test]
fn source_and_binding_result_shapes_must_match() {
    let declaration = declare_projection_consumption(
        test_source(ProjectionSourceFamily::QueryReadReceipt),
        ProjectionConsumptionBindingContext::test_only(
            "result-shape:other",
            "authorized-projection:test",
            crate::projection_consumption::test_authorized_field_paths(&["identity.id"]),
        ),
        ProjectMaterializedFacts::declare().entity_identities(),
    );

    assert_eq!(
        declaration,
        Err(
            ProjectionConsumptionDeclarationError::BindingAuthorizedProjectionResultShapeMismatch {
                binding_result_shape_digest: "result-shape:other".to_string(),
                authorized_projection_result_shape_digest: "result-shape:test".to_string(),
            }
        )
    );
}

#[test]
fn source_and_authorized_projection_queries_must_match() {
    let declaration = declare_projection_consumption(
        test_source(ProjectionSourceFamily::QueryReadReceipt),
        test_binding_with_projection_metadata("result-shape:test", "query:other", &["identity.id"]),
        ProjectMaterializedFacts::declare().entity_identities(),
    );

    assert_eq!(
        declaration,
        Err(
            ProjectionConsumptionDeclarationError::SourceAuthorizedProjectionQueryMismatch {
                source_query_digest: "query:test".to_string(),
                authorized_projection_query_digest: "query:other".to_string(),
            }
        )
    );
}

#[test]
fn binding_result_shape_must_match_authorized_projection_result_shape() {
    let declaration = declare_projection_consumption(
        test_source(ProjectionSourceFamily::RelationalRowSet),
        ProjectionConsumptionBindingContext::test_only_with_projection_metadata(
            "result-shape:other",
            "query:test",
            "result-shape:test",
            "authorized-projection:test",
            "narrowed-result-shape:test",
            "policy:test",
            "tenant-schema:test",
            crate::projection_consumption::test_authorized_field_paths(&["identity.id"]),
        ),
        ProjectMaterializedFacts::declare().entity_identities(),
    );

    assert_eq!(
        declaration,
        Err(
            ProjectionConsumptionDeclarationError::BindingAuthorizedProjectionResultShapeMismatch {
                binding_result_shape_digest: "result-shape:other".to_string(),
                authorized_projection_result_shape_digest: "result-shape:test".to_string(),
            }
        )
    );
}
