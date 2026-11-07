use crate::projection_consumption::certification::{
    projection_consumption_forbidden_fallback_audit, ProjectionConsumptionForbiddenFallbackSeam,
    ProjectionConsumptionOrdinaryPathSurface,
};
use crate::projection_consumption::{
    ProjectMaterializedFacts, ProjectionFactConsumptionAttempt, ProjectionSourceFamily,
};
use crate::runtime::{
    forbidden_fallback_seam_invocation_count, reset_forbidden_fallback_seam_invocations,
    WorthQueryForbiddenFallbackSeam,
};

use super::support::{
    authorized_projection, live_binding, retained_binding, shared_test_result_shape,
    test_result_shape_artifact, test_result_shape_canonical_digest,
};

#[test]
fn ordinary_projection_path_forbidden_fallback_audit_reports_exact_zero() {
    let audit = projection_consumption_forbidden_fallback_audit().expect("fixture digest must fit");

    assert_eq!(audit.total_occurrence_count(), 0);
    for surface in [ProjectionConsumptionOrdinaryPathSurface::CommonPathDx] {
        for seam in [
            ProjectionConsumptionForbiddenFallbackSeam::ConsumeScalarFields,
            ProjectionConsumptionForbiddenFallbackSeam::DecodeRowPair,
            ProjectionConsumptionForbiddenFallbackSeam::DecodeRowTriple,
            ProjectionConsumptionForbiddenFallbackSeam::VerifyScalarAlignment,
            ProjectionConsumptionForbiddenFallbackSeam::ReadLiveArtifactBundle,
            ProjectionConsumptionForbiddenFallbackSeam::BindLiveArtifact,
            ProjectionConsumptionForbiddenFallbackSeam::ReadLiveArtifactBinding,
        ] {
            let row = audit
                .rows()
                .iter()
                .find(|row| row.ordinary_surface() == surface && row.forbidden_seam() == seam)
                .expect("audit row should exist for every surface/seam pair");
            assert_eq!(row.occurrence_count(), 0);
        }
    }
}

#[test]
fn retained_and_live_ordinary_consumption_path_stays_receipt_first_and_zero_reopen() {
    let retained_binding = retained_binding();
    let live_binding = live_binding();
    reset_forbidden_fallback_seam_invocations();

    let retained = retained_binding
        .consume_projection_facts(
            &test_result_shape_artifact("result-shape:test"),
            &authorized_projection(
                "query:test",
                &test_result_shape_canonical_digest("result-shape:test"),
                &["profile.display_name"],
            ),
            ProjectMaterializedFacts::declare()
                .view_local_identities()
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
        .expect("retained ordinary projection path should succeed");
    let live = live_binding
        .consume_projection_facts(
            &shared_test_result_shape().identity,
            &authorized_projection(
                "query:test",
                &shared_test_result_shape().digest,
                &["profile.display_name"],
            ),
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
                .source_references(),
        )
        .expect("live ordinary projection path should succeed");

    for (expected_family, attempt) in [
        (
            ProjectionSourceFamily::RetainedDerivedArtifactBinding,
            retained,
        ),
        (ProjectionSourceFamily::LiveArtifactBinding, live),
    ] {
        match attempt {
            ProjectionFactConsumptionAttempt::Admitted(completed) => {
                assert_eq!(completed.source_family(), expected_family);
                assert_eq!(completed.authority_reopen_count(), 0);
                assert_eq!(
                    completed.receipt().source_family(),
                    completed.projection_consumption_envelope().source_family()
                );
                assert_eq!(
                    completed.receipt().receipt_digest(),
                    completed
                        .projection_consumption_envelope()
                        .sources()
                        .receipt_digest()
                );
            }
            other => panic!("expected admitted ordinary retained/live path, got {other:?}"),
        }
    }

    for seam in [
        WorthQueryForbiddenFallbackSeam::ConsumeScalarFields,
        WorthQueryForbiddenFallbackSeam::DecodeRowPair,
        WorthQueryForbiddenFallbackSeam::DecodeRowTriple,
        WorthQueryForbiddenFallbackSeam::VerifyScalarAlignment,
        WorthQueryForbiddenFallbackSeam::ReadLiveArtifactBundle,
        WorthQueryForbiddenFallbackSeam::BindLiveArtifact,
        WorthQueryForbiddenFallbackSeam::ReadLiveArtifactBinding,
    ] {
        assert_eq!(forbidden_fallback_seam_invocation_count(seam), 0);
    }
}
