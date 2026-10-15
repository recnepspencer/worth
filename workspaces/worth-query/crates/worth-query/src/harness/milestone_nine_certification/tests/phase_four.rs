use super::super::{MilestoneNineCertificationAdapter, MilestoneNineFailureClass};

#[test]
fn milestone_nine_phase_four_closeout_rows_are_concrete() {
    let matrix = MilestoneNineCertificationAdapter::policy_tenant_context_admission_test();
    let employee = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "employee-record-fixture-policy-basis")
        .expect("employee fixture row should exist");
    let scale = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "policy-scale-slope-honesty")
        .expect("scale slope row should exist");
    let delivery = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "delivery-width-class-honesty")
        .expect("delivery width row should exist");
    let live_drift = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "live-policy-epoch-drift-readmission")
        .expect("live drift row should exist");
    let live_density = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "live-policy-density-posture-honesty")
        .expect("live density row should exist");
    let aggregation = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "masked-aggregation-without-witness-forbidden")
        .expect("masked aggregation rejection row should exist");
    let placeholder = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "masked-placeholder-shape-forbidden")
        .expect("masked placeholder rejection row should exist");
    let cursor = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "masked-cursor-without-witness-forbidden")
        .expect("masked cursor rejection row should exist");
    let view_membership = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "masked-view-membership-without-witness-forbidden")
        .expect("masked view membership rejection row should exist");
    let allocation = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "policy-per-row-allocation-forbidden")
        .expect("per-row allocation rejection row should exist");
    let fanout = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "policy-cross-tenant-fanout-forbidden")
        .expect("cross-tenant fanout rejection row should exist");
    let saved_bypass = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "saved-query-policy-bypass-forbidden")
        .expect("saved bypass rejection row should exist");
    let workflow = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "unsupported-policy-workflow-composition-forbidden")
        .expect("unsupported workflow composition rejection row should exist");

    assert!(!employee.control_lane.employee_fixture_digest.is_empty());
    assert!(!scale
        .control_lane
        .policy_scale_counter_slope_digest
        .is_empty());
    assert!(!delivery.control_lane.delivery_width_class_digest.is_empty());
    assert!(!live_drift
        .hostile_lane
        .live_drift_evidence_digest
        .is_empty());
    assert!(!live_density
        .control_lane
        .live_drift_evidence_digest
        .is_empty());
    assert_ne!(
        live_drift.control_lane.live_drift_evidence_digest,
        live_drift.hostile_lane.live_drift_evidence_digest
    );
    assert_ne!(
        live_drift.control_lane.counter_snapshot_digest,
        live_drift.hostile_lane.counter_snapshot_digest
    );
    assert_ne!(
        live_density.control_lane.counter_snapshot_digest,
        delivery.control_lane.counter_snapshot_digest
    );
    assert_eq!(
        aggregation.hostile_lane.failure_class,
        MilestoneNineFailureClass::PolicyNarrowingDenied
    );
    assert_eq!(
        placeholder.hostile_lane.failure_class,
        MilestoneNineFailureClass::PolicyExecutionSeamDenied
    );
    assert_ne!(
        aggregation.hostile_lane.counter_snapshot_digest,
        cursor.hostile_lane.counter_snapshot_digest
    );
    assert_ne!(
        cursor.hostile_lane.counter_snapshot_digest,
        view_membership.hostile_lane.counter_snapshot_digest
    );
    assert_eq!(
        allocation.hostile_lane.failure_class,
        MilestoneNineFailureClass::PolicyExecutionSeamDenied
    );
    assert_ne!(
        allocation.hostile_lane.counter_snapshot_digest,
        fanout.hostile_lane.counter_snapshot_digest
    );
    assert_ne!(
        saved_bypass.hostile_lane.counter_snapshot_digest,
        workflow.hostile_lane.counter_snapshot_digest
    );
}
