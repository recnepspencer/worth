use super::fixture::real_fixture;
use crate::inspection::RuntimeWorldRetentionKey;
#[test]
fn caller_named_unpinned_key_is_reclaimed_while_sibling_remains_pinned() {
    let fixture = real_fixture(4, 4);
    let request = crate::retention::ExactComponentPinRequest::relational(
        &fixture.basis,
        crate::retention::ComponentBasisDependencyClass::ActivePublicationAttempt,
    );
    let relational = fixture.owner.issue_component(request).unwrap();
    let signal = fixture
        .owner
        .issue_component(crate::retention::ExactComponentPinRequest::signal(
            &fixture.basis,
            crate::retention::ComponentBasisDependencyClass::ActivePublicationAttempt,
        ))
        .unwrap();
    drop(signal);
    let signal_key = RuntimeWorldRetentionKey::signal(&fixture.basis);
    let rel_key = RuntimeWorldRetentionKey::relational(&fixture.basis);
    let report = fixture
        .owner
        .reclaim_keys(&[signal_key, rel_key], 1)
        .unwrap();
    assert_eq!(report.examined(), 1);
    assert_eq!(report.reclaimed(), 1);
    assert_eq!(fixture.owner.unique_pin_count(), 1);
    drop(relational);
}
