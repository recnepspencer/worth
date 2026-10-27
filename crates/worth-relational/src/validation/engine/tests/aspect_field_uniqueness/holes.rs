//! Rejected reservations must not hide later records from global uniqueness.
use super::*;

#[test]
fn uniqueness_sees_records_past_a_refused_reservation_hole() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_summary_title_uniqueness();
    commit_entity_with_summary(execution, &runtime, "alpha", "alpha-title", "open").unwrap();
    let denied = commit_entity_with_summary(execution, &runtime, "denied", "alpha-title", "open");
    assert_unique_entity_field_conflict(denied.unwrap_err(), "alpha-title");
    let beta =
        commit_entity_with_summary(execution, &runtime, "beta", "beta-title", "open").unwrap();
    let id = beta
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::transactions::data::RecordRef::Entity(id) => Some(*id),
            crate::transactions::data::RecordRef::Relation(_) => None,
        })
        .unwrap();
    assert!(
        id.local_slot_value() > 1,
        "the successful record is beyond a burned slot"
    );
    let before = runtime_marker(&runtime);
    let duplicate =
        commit_entity_with_summary(execution, &runtime, "duplicate", "beta-title", "closed");
    let Err(denial) = duplicate else {
        panic!("the duplicate beyond a burned slot was published");
    };
    assert_unique_entity_field_conflict(denial, "beta-title");
    assert_runtime_marker_unchanged(&runtime, before);
}
