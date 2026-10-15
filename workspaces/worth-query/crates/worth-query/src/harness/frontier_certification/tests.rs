use super::{
    row_catalog::FRONTIER_REJECTION_ROW_SPECS, FrontierRouteClass,
    MilestoneFivePointThreeFrontierCertificationAdapter,
};

#[test]
fn frontier_certification_lanes_preserve_exact_identities_and_results() {
    use crate::harness::fixtures::execution_preflights::{
        ordered_collection_preflight, ordered_collection_without_traversal_preflight,
    };
    let matrix = MilestoneFivePointThreeFrontierCertificationAdapter::frontier_planning_and_serial_fallback_parity_test();
    for row in &matrix.rows {
        let expected_family = match row.row_name {
            "frontier-serial-control" => FrontierRouteClass::SerialControl,
            "serial-fallback-parity" => FrontierRouteClass::SerialFallback,
            "exact-basis-bundle-parity" => FrontierRouteClass::SerialFallbackBundle,
            other => panic!("unexpected canonical row {other}"),
        };
        let preflight = if expected_family == FrontierRouteClass::SerialControl {
            ordered_collection_without_traversal_preflight()
        } else {
            ordered_collection_preflight()
        };
        let baseline =
            crate::execution::execute_preflight_bundle(&preflight).expect("baseline execution");
        for lane in [&row.control_lane, &row.hostile_lane, &row.parity_lane] {
            assert_eq!(lane.route_class(), expected_family);
            assert_eq!(
                lane.parity_bundle.query_digest(),
                preflight.plan().query().validated_query_digest()
            );
            assert_eq!(
                lane.parity_bundle.plan_digest(),
                preflight.plan().query().plan_digest()
            );
            assert_eq!(
                lane.parity_bundle.basis_digest(),
                preflight.basis().proof().digest().as_str()
            );
            assert_eq!(
                lane.parity_bundle.result_digest(),
                baseline.report().result_digest()
            );
            let counters = lane.counter_snapshot();
            assert_eq!(
                counters.reported_execution_records_examined_count(),
                baseline.counters().execution_records_examined_count()
            );
            assert_eq!(
                lane.parity_bundle
                    .reported_execution_records_examined_count(),
                baseline.counters().execution_records_examined_count()
            );
            let members = if expected_family == FrontierRouteClass::SerialFallbackBundle {
                2
            } else {
                1
            };
            assert_eq!(counters.planned_packet_merge_boundary_count(), members);
            // Fixtures declare two projections and one ordering surface (3).
            // Bounded materialization adds a traversal surface and one manager edge (5).
            // The bundle contains two bounded members (10), irrespective of executor work.
            let expected_breadth = match expected_family {
                FrontierRouteClass::SerialControl => 3,
                FrontierRouteClass::SerialFallback => 5,
                FrontierRouteClass::SerialFallbackBundle => 10,
            };
            assert_eq!(counters.planned_frontier_breadth(), expected_breadth);
            let selected_breadth = if expected_family == FrontierRouteClass::SerialControl {
                3
            } else {
                5
            };
            assert_eq!(
                lane.parity_bundle.predicted_breadth().value(),
                selected_breadth
            );
            assert_eq!(
                counters.bundle_serial_route_count(),
                if members == 2 { 2 } else { 0 }
            );
            assert_eq!(
                counters.planned_serial_fallback_route_count(),
                if expected_family == FrontierRouteClass::SerialControl {
                    0
                } else {
                    members
                }
            );
            assert_eq!(
                counters.selected_route_nonbudget_drift_posture_count(),
                usize::from(expected_family == FrontierRouteClass::SerialFallback)
            );
        }
    }
}

#[test]
fn frontier_certification_rejections_are_typed() {
    let matrix = MilestoneFivePointThreeFrontierCertificationAdapter::frontier_planning_and_serial_fallback_parity_test();
    for spec in FRONTIER_REJECTION_ROW_SPECS {
        let row = matrix
            .rejection_rows
            .iter()
            .find(|row| row.row_name == spec.row_name)
            .expect("rejection row");
        assert_eq!(row.hostile_lane.failure_class, spec.failure_class);
    }
}

#[test]
fn frontier_certification_artifact_is_deterministic() {
    let left =
        MilestoneFivePointThreeFrontierCertificationAdapter::frontier_planning_and_serial_fallback_parity_artifact();
    let right =
        MilestoneFivePointThreeFrontierCertificationAdapter::frontier_planning_and_serial_fallback_parity_artifact();

    assert_eq!(
        left.certification_bundle_digest,
        right.certification_bundle_digest
    );
    assert_eq!(left.coverage_matrix_digest, right.coverage_matrix_digest);
}
