use super::MilestoneNineTwoCertificationAdapter;
use crate::harness::certification::{
    milestone_nine_two_requirements, unmet_required_assertion_classes, HostileExpectation,
    ParityAnchor, RequiredAssertionClass,
};

#[test]
fn milestone_nine_two_rows_enforce_lifecycle_semantics() {
    let matrix = MilestoneNineTwoCertificationAdapter::
        subscription_lifecycle_sharing_and_preview_parity_test();
    let mut covered = Vec::new();

    for row in &matrix.rows {
        let control = row.control_lane.lifecycle_signature();
        let hostile = row.hostile_lane.lifecycle_signature();
        let parity = row.parity_lane.lifecycle_signature();
        match row.hostile_expectation {
            HostileExpectation::EquivalentToControl => {
                assert_eq!(control, hostile, "row '{}'", row.row_name);
                covered.push(RequiredAssertionClass::Equality);
            }
            HostileExpectation::DistinctFromControl => {
                assert_ne!(control, hostile, "row '{}'", row.row_name);
                covered.push(RequiredAssertionClass::Inequality);
            }
        }
        match row.parity_anchor {
            ParityAnchor::Control => assert_eq!(parity, control, "row '{}'", row.row_name),
            ParityAnchor::Hostile => assert_eq!(parity, hostile, "row '{}'", row.row_name),
        }
    }

    for row in &matrix.rejection_rows {
        covered.push(RequiredAssertionClass::TypedFailure);
        if row.hostile_lane.counter_snapshot != row.control_lane.counter_snapshot {
            covered.push(RequiredAssertionClass::ZeroResidue);
        }
    }
    covered.sort();
    covered.dedup();
    let missing = unmet_required_assertion_classes(
        &covered,
        milestone_nine_two_requirements().required_assertion_classes,
    );
    assert!(missing.is_empty(), "missing assertion classes: {missing:?}");
}

#[test]
fn milestone_nine_two_preview_rows_emit_real_preview_evidence() {
    let matrix = MilestoneNineTwoCertificationAdapter::
        subscription_lifecycle_sharing_and_preview_parity_test();

    let discard = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "preview-discard-zero-authoritative-residue")
        .unwrap();
    assert_ne!(discard.control_lane.preview_isolation_digest, "none");
    assert_ne!(discard.control_lane.preview_residue_digest, "none");

    let promotion = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "preview-promotion-boundary-handoff")
        .unwrap();
    assert_ne!(
        promotion.hostile_lane.preview_residue_digest,
        discard.control_lane.preview_residue_digest
    );
}

#[test]
fn milestone_nine_two_sharing_row_proves_shared_lane_and_consumer_local_delivery() {
    let matrix = MilestoneNineTwoCertificationAdapter::
        subscription_lifecycle_sharing_and_preview_parity_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "equivalent-subscription-sharing-fanout")
        .unwrap();

    for lane in [&row.control_lane, &row.hostile_lane, &row.parity_lane] {
        assert!(
            lane.counter_evidence
                .iter()
                .any(|part| part == "same_lane:true"),
            "sharing row did not prove both consumers joined one lane"
        );
        assert!(
            lane.counter_evidence
                .iter()
                .any(|part| part == "consumer_local_delivery_count:2"),
            "sharing row did not prove both consumers received local delivery evidence"
        );
        assert_ne!(lane.acknowledgement_frontier_digest, "none");
        assert_ne!(lane.delivery_receipt_digest, "none");
    }
}

#[test]
fn milestone_nine_two_performance_receipt_row_is_posture_sensitive() {
    let matrix = MilestoneNineTwoCertificationAdapter::
        subscription_lifecycle_sharing_and_preview_parity_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "performance-receipt-posture-sensitive")
        .unwrap();

    assert_eq!(
        row.control_lane.query_digest, row.hostile_lane.query_digest,
        "the row must hold functional query meaning constant"
    );
    assert_ne!(
        row.control_lane.subscription_performance_receipt_digest,
        row.hostile_lane.subscription_performance_receipt_digest,
        "performance receipt must change when density/allocation posture changes"
    );
    assert_ne!(
        row.control_lane.active_delivery_density_posture_digest,
        row.hostile_lane.active_delivery_density_posture_digest
    );
    assert_ne!(
        row.control_lane.allocation_posture_digest,
        row.hostile_lane.allocation_posture_digest
    );
}

#[test]
fn milestone_nine_two_scale_row_binds_widths_not_only_row_count() {
    let matrix = MilestoneNineTwoCertificationAdapter::
        subscription_lifecycle_sharing_and_preview_parity_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "scale-slope-width-bounded-lifecycle")
        .unwrap();

    assert_ne!(
        row.control_lane.subscription_lifecycle_scale_slope_digest,
        row.hostile_lane.subscription_lifecycle_scale_slope_digest
    );
    assert_ne!(
        row.control_lane.patch_group_digest,
        row.hostile_lane.patch_group_digest
    );
    for required_axis in [
        "scale_axis:unrelated_row_count",
        "scale_axis:active_lane_count",
        "scale_axis:consumers_per_lane",
        "scale_axis:group_count",
    ] {
        assert!(
            row.hostile_lane
                .counter_evidence
                .iter()
                .any(|part| part == required_axis),
            "scale row missing axis evidence {required_axis}"
        );
    }
}
