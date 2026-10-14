use super::*;
#[test]
fn receiver_leases_disarm_unused_consumed_and_unwound_registrations() {
    let owner = Arc::new(PublicationFaultControl::default());
    let other = Arc::new(PublicationFaultControl::default());
    let guard = owner.reject_pretransaction().unwrap();
    assert!(owner.hold_snapshot_failure().is_err());
    assert!(!other.take(WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction));
    drop(guard);
    assert!(!owner.take(WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction));
    let guard = owner.reject_pretransaction().unwrap();
    assert!(owner.take(WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction));
    assert!(!owner.take(WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction));
    drop(guard);
    let _ = std::panic::catch_unwind(|| {
        let _guard = owner.hold_snapshot_failure().unwrap();
        assert!(owner.take(WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot));
        assert!(owner.take(WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot));
        panic!("unwind the owned registration");
    });
    assert!(!owner.take(WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot));
    assert!(owner.reject_pretransaction().is_ok());
}
