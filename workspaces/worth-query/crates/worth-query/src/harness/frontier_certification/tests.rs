use super::{
    row_catalog::{
        FRONTIER_CANONICAL_ROW_SPECS, FRONTIER_REJECTION_ROW_SPECS,
        FRONTIER_REQUIRED_CANONICAL_ROW_NAMES, FRONTIER_REQUIRED_REJECTION_ROW_NAMES,
    },
    FrontierCloseoutStatus, FrontierRouteClass,
    MilestoneFivePointThreeFrontierCertificationAdapter,
};
use crate::harness::certification::{milestone_five_point_three_requirements, unmet_required_rows};
use std::collections::BTreeSet;

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
fn frontier_certification_adapter_emits_named_matrix() {
    let matrix =
        MilestoneFivePointThreeFrontierCertificationAdapter::frontier_planning_and_serial_fallback_parity_test();

    assert_eq!(
        matrix.suite_name,
        "Frontier Planning And Serial Fallback Parity Test"
    );
    for spec in FRONTIER_CANONICAL_ROW_SPECS {
        assert!(matrix.rows.iter().any(|row| row.row_name == spec.row_name));
    }
    for spec in FRONTIER_REJECTION_ROW_SPECS {
        assert!(matrix
            .rejection_rows
            .iter()
            .any(|row| row.row_name == spec.row_name));
    }
}

#[test]
fn frontier_certification_matrix_meets_milestone_requirements() {
    let matrix =
        MilestoneFivePointThreeFrontierCertificationAdapter::frontier_planning_and_serial_fallback_parity_test();
    let requirements = milestone_five_point_three_requirements();
    let missing = unmet_required_rows(
        &matrix,
        FRONTIER_REQUIRED_CANONICAL_ROW_NAMES,
        FRONTIER_REQUIRED_REJECTION_ROW_NAMES,
    );

    assert!(missing.is_empty(), "missing frontier rows: {missing:?}");
    let spec_missing = unmet_required_rows(
        &matrix,
        requirements.required_canonical_rows,
        requirements.required_rejection_rows,
    );
    assert!(
        spec_missing.is_empty(),
        "missing spec frontier rows: {spec_missing:?}"
    );
    assert!(matrix
        .rows
        .iter()
        .all(|row| row.control_lane.has_required_outputs()));
    assert!(matrix
        .rows
        .iter()
        .all(|row| row.hostile_lane.has_required_outputs()));
    assert!(matrix
        .rows
        .iter()
        .all(|row| row.parity_lane.has_required_outputs()));
    assert!(matrix
        .rejection_rows
        .iter()
        .all(|row| row.hostile_lane.has_required_outputs()));
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

#[test]
fn frontier_certification_rows_assert_exact_bundle_counter_shapes() {
    let matrix =
        MilestoneFivePointThreeFrontierCertificationAdapter::frontier_planning_and_serial_fallback_parity_test();
    let serial_bundle = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "exact-basis-bundle-parity")
        .expect("serial bundle row should exist");

    assert_eq!(
        serial_bundle
            .control_lane
            .counter_snapshot()
            .bundle_serial_route_count(),
        2
    );
}

#[test]
fn frontier_closeout_metadata_is_complete_and_references_known_rows() {
    let artifact =
        MilestoneFivePointThreeFrontierCertificationAdapter::frontier_planning_closeout_artifact();

    assert!(!artifact.closeout_matrix_digest.is_empty());
    assert!(!artifact.certification_bundle_digest.is_empty());
    assert!(artifact.all_requirements_marked_satisfied());
    for requirement in artifact
        .must_ship
        .iter()
        .chain(artifact.must_preserve.iter())
        .chain(artifact.proof_obligations.iter())
        .chain(artifact.acceptance_evidence.iter())
    {
        assert_eq!(requirement.status, FrontierCloseoutStatus::Satisfied);
        assert!(
            !requirement.proof_artifacts.is_empty(),
            "requirement {:?} must name at least one proof artifact",
            requirement.requirement_name
        );
        assert!(
            !requirement.notes.is_empty(),
            "requirement {:?} must include proof notes",
            requirement.requirement_name
        );
        assert!(
            !requirement.certification_rows.is_empty(),
            "requirement {:?} must reference at least one certification row",
            requirement.requirement_name
        );
    }

    let known_rows = FRONTIER_REQUIRED_CANONICAL_ROW_NAMES
        .iter()
        .chain(FRONTIER_REQUIRED_REJECTION_ROW_NAMES.iter())
        .copied()
        .collect::<BTreeSet<_>>();

    for requirement in artifact
        .must_ship
        .iter()
        .chain(artifact.must_preserve.iter())
        .chain(artifact.proof_obligations.iter())
        .chain(artifact.acceptance_evidence.iter())
    {
        for row_name in requirement.certification_rows {
            assert!(
                known_rows.contains(row_name),
                "requirement {:?} references unknown frontier row {:?}",
                requirement.requirement_name,
                row_name
            );
        }
    }

    let full_suite_requirement = artifact
        .acceptance_evidence
        .iter()
        .find(|requirement| {
            requirement.requirement_name
                == "frontier planning and serial fallback parity suite passes"
        })
        .expect("full suite acceptance requirement should exist");
    assert_eq!(
        full_suite_requirement.certification_rows.len(),
        FRONTIER_REQUIRED_CANONICAL_ROW_NAMES.len() + FRONTIER_REQUIRED_REJECTION_ROW_NAMES.len(),
        "full suite acceptance requirement must bind every required canonical and rejection row"
    );

    let denial_requirement = artifact
        .acceptance_evidence
        .iter()
        .find(|requirement| {
            requirement.requirement_name
                == "unsupported families and mixed-basis bundles fail typed"
        })
        .expect("typed denial acceptance requirement should exist");
    for row_name in [
        "unsupported-frontier-family",
        "unsupported-bundle-composition",
        "mixed-basis-bundle-denied",
    ] {
        assert!(
            denial_requirement.certification_rows.contains(&row_name),
            "typed denial requirement must bind rejection row {row_name}"
        );
    }
}
