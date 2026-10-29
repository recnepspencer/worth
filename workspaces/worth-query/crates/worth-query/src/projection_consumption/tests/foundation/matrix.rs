use super::super::super::{
    declare_projection_consumption, discover_projection_consumption_support,
    evaluate_projection_consumption_eligibility, ProjectionConsumptionEligibility,
    ProjectionConsumptionSupportPosture, ProjectionFactKind,
};
use super::support::{
    all_source_families, assert_warning_matches_posture, request_for_kind, test_binding,
    test_source, visible_fields_for_kind,
};

#[test]
fn support_discovery_and_eligibility_stay_in_sync_for_all_lanes() {
    for family in all_source_families() {
        let source = test_source(family);
        let report = discover_projection_consumption_support(&source);
        for fact_kind in ProjectionFactKind::all().iter().copied() {
            let row = report
                .rows()
                .iter()
                .find(|row| row.fact_kind() == fact_kind)
                .expect("support row should exist for every fact kind");
            let declaration = declare_projection_consumption(
                source.clone(),
                test_binding(&visible_fields_for_kind(fact_kind)),
                request_for_kind(fact_kind),
            )
            .expect("matrix declaration should be structurally valid");
            let eligibility = evaluate_projection_consumption_eligibility(&declaration);
            match (row.posture(), eligibility) {
                (
                    ProjectionConsumptionSupportPosture::Admitted,
                    ProjectionConsumptionEligibility::Admitted(_),
                ) => {}
                (
                    ProjectionConsumptionSupportPosture::AdmittedWithWarnings(expected),
                    ProjectionConsumptionEligibility::AdmittedWithWarnings(_, warnings),
                ) => assert_warning_matches_posture(&warnings, expected.clone()),
                (
                    ProjectionConsumptionSupportPosture::Deferred(expected_reason),
                    ProjectionConsumptionEligibility::Deferred(deferred),
                ) => assert_eq!(deferred.reason(), expected_reason),
                (
                    ProjectionConsumptionSupportPosture::SourceMismatch,
                    ProjectionConsumptionEligibility::SourceMismatch(mismatch),
                ) => {
                    assert_eq!(mismatch.source_family(), family);
                    assert_eq!(mismatch.requested_fact_kind(), fact_kind);
                }
                (posture, other) => {
                    panic!(
                        "support posture and eligibility diverged for family {family:?} fact {fact_kind:?}: posture {posture:?}, eligibility {other:?}"
                    );
                }
            }
        }
    }
}
