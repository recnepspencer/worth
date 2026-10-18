use super::*;

#[test]
fn milestone_nine_one_rows_enforce_declared_lane_semantics() {
    let matrix = MilestoneNineOneCertificationAdapter::
        query_subscription_declaration_and_lowering_parity_test();
    let mut covered = Vec::new();

    for row in &matrix.rows {
        let control = row.control_lane.subscription_semantic_signature();
        let hostile = row.hostile_lane.subscription_semantic_signature();
        let parity = row.parity_lane.subscription_semantic_signature();

        match row.hostile_expectation {
            HostileExpectation::EquivalentToControl => assert_eq!(
                control, hostile,
                "row '{}' declares hostile equivalence but emits different subscription evidence",
                row.row_name
            ),
            HostileExpectation::DistinctFromControl => assert_ne!(
                control, hostile,
                "row '{}' declares hostile distinction but emits identical subscription evidence",
                row.row_name
            ),
        }
        match row.hostile_expectation {
            HostileExpectation::EquivalentToControl => {
                covered.push(RequiredAssertionClass::Equality)
            }
            HostileExpectation::DistinctFromControl => {
                covered.push(RequiredAssertionClass::Inequality)
            }
        }

        match row.parity_anchor {
            ParityAnchor::Control => assert_eq!(
                parity, control,
                "row '{}' parity lane must independently match the control anchor",
                row.row_name
            ),
            ParityAnchor::Hostile => assert_eq!(
                parity, hostile,
                "row '{}' parity lane must independently match the hostile anchor",
                row.row_name
            ),
        }
    }

    for row in &matrix.rejection_rows {
        assert!(!row.hostile_lane.failure_digest.is_empty());
        covered.push(RequiredAssertionClass::TypedFailure);
        if row.hostile_lane.counter_snapshot_digest != row.control_lane.counter_snapshot_digest {
            covered.push(RequiredAssertionClass::ZeroResidue);
        }
    }

    covered.sort();
    covered.dedup();
    let missing = unmet_required_assertion_classes(
        &covered,
        milestone_nine_one_requirements().required_assertion_classes,
    );
    assert!(
        missing.is_empty(),
        "missing milestone 9.1 assertion classes: {missing:?}"
    );
}
