use crate::projection_consumption::{
    evaluate_projection_consumption_eligibility, ProjectionConsumptionEligibility,
    ProjectionConsumptionSupportPosture, ProjectionFactKind, ProjectionSourceFamily,
};

use super::super::support::{
    authorized_projection, live_binding, request_for_kind, retained_binding,
    shared_test_result_shape, test_result_shape_artifact, test_result_shape_canonical_digest,
    visible_fields_for_kind,
};

fn assert_support_and_eligibility_sync_for_retained_binding() {
    let support = retained_binding().discover_projection_fact_consumption_support();

    for fact_kind in ProjectionFactKind::all().iter().copied() {
        let support_row = support
            .rows()
            .iter()
            .find(|row| row.fact_kind() == fact_kind)
            .expect("support row should exist");
        let declaration = retained_binding()
            .declare_projection_fact_consumption(
                &test_result_shape_artifact("result-shape:test"),
                &authorized_projection(
                    "query:test",
                    &test_result_shape_canonical_digest("result-shape:test"),
                    &visible_fields_for_kind(fact_kind),
                ),
                request_for_kind(fact_kind),
            )
            .expect("retained declaration should remain structurally valid");
        let eligibility = evaluate_projection_consumption_eligibility(&declaration);

        match (support_row.posture(), eligibility) {
            (
                ProjectionConsumptionSupportPosture::Admitted,
                ProjectionConsumptionEligibility::Admitted(_),
            ) => {}
            (
                ProjectionConsumptionSupportPosture::SourceMismatch,
                ProjectionConsumptionEligibility::SourceMismatch(mismatch),
            ) => {
                assert_eq!(
                    mismatch.source_family(),
                    ProjectionSourceFamily::RetainedDerivedArtifactBinding
                );
                assert_eq!(mismatch.requested_fact_kind(), fact_kind);
            }
            (posture, other) => {
                panic!(
                    "retained support posture and eligibility diverged for fact {fact_kind:?}: posture {posture:?}, eligibility {other:?}"
                );
            }
        }
    }
}

fn assert_support_and_eligibility_sync_for_live_binding() {
    let support = live_binding().discover_projection_fact_consumption_support();

    for fact_kind in ProjectionFactKind::all().iter().copied() {
        let support_row = support
            .rows()
            .iter()
            .find(|row| row.fact_kind() == fact_kind)
            .expect("support row should exist");
        let declaration = live_binding()
            .declare_projection_fact_consumption(
                &shared_test_result_shape().identity,
                &authorized_projection(
                    "query:test",
                    &shared_test_result_shape().digest,
                    &visible_fields_for_kind(fact_kind),
                ),
                request_for_kind(fact_kind),
            )
            .expect("live declaration should remain structurally valid");
        let eligibility = evaluate_projection_consumption_eligibility(&declaration);

        match (support_row.posture(), eligibility) {
            (
                ProjectionConsumptionSupportPosture::Admitted,
                ProjectionConsumptionEligibility::Admitted(_),
            ) => {}
            (
                ProjectionConsumptionSupportPosture::SourceMismatch,
                ProjectionConsumptionEligibility::SourceMismatch(mismatch),
            ) => {
                assert_eq!(
                    mismatch.source_family(),
                    ProjectionSourceFamily::LiveArtifactBinding
                );
                assert_eq!(mismatch.requested_fact_kind(), fact_kind);
            }
            (posture, other) => {
                panic!(
                    "live support posture and eligibility diverged for fact {fact_kind:?}: posture {posture:?}, eligibility {other:?}"
                );
            }
        }
    }
}

#[test]
fn retained_and_live_support_reports_match_family_boundaries() {
    let retained = retained_binding().discover_projection_fact_consumption_support();
    let live = live_binding().discover_projection_fact_consumption_support();

    assert!(matches!(
        retained
            .rows()
            .iter()
            .find(|row| row.fact_kind() == ProjectionFactKind::SourceReference)
            .expect("retained source reference row should exist")
            .posture(),
        ProjectionConsumptionSupportPosture::Admitted
    ));
    assert!(matches!(
        retained
            .rows()
            .iter()
            .find(|row| row.fact_kind() == ProjectionFactKind::EntityIdentity)
            .expect("retained entity identity row should exist")
            .posture(),
        ProjectionConsumptionSupportPosture::SourceMismatch
    ));
    assert!(matches!(
        live.rows()
            .iter()
            .find(|row| row.fact_kind() == ProjectionFactKind::EntityIdentity)
            .expect("live entity identity row should exist")
            .posture(),
        ProjectionConsumptionSupportPosture::Admitted
    ));
}

#[test]
fn retained_and_live_support_and_eligibility_stay_in_sync_for_all_fact_kinds() {
    assert_support_and_eligibility_sync_for_retained_binding();
    assert_support_and_eligibility_sync_for_live_binding();
}
