use super::harness;

#[test]
fn observer_missing_root_fails_without_emitting_a_report() {
    harness::assert_missing_root_observer_fails();
}
