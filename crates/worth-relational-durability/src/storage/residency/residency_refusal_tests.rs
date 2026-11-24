//! Cold region integrity and limits refuse without admitting leases.
use super::*;

#[test]
fn damaged_content_and_leaf_mismatches_are_sticky_typed_refusals() {
    let fixture = region_with_history();
    fixture.source.corrupt.store(true, Ordering::SeqCst);
    let first = head_demand(&fixture.cell, 1 << 24).unwrap_err();
    assert_eq!(
        first.subject,
        RelationalStorageSubject::Region(fixture.entity.partition_id)
    );
    assert_eq!(first.cause, Cause::IntegrityMismatch);
    fixture.source.corrupt.store(false, Ordering::SeqCst);
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    assert_eq!(full_demand(&fixture.cell, 1 << 24).unwrap_err(), first);
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), reads);
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 0);

    let mut wrong_leaf = fixture.cold;
    wrong_leaf.content_digest[0] ^= 1;
    let cell = RelationalRegionResidency::cold(wrong_leaf).unwrap();
    assert_eq!(
        head_demand(&cell, 1 << 24).unwrap_err().cause,
        Cause::IntegrityMismatch
    );
}

#[test]
fn region_limit_refuses_before_admitting_a_lease_and_recovery_preserves_the_cause() {
    let fixture = region_with_history();
    let failure = head_demand(&fixture.cell, 1).unwrap_err();
    assert!(matches!(
        failure.cause,
        Cause::LimitExceeded { limit: 1, .. }
    ));
    let contracts::durability::data::recovery_stop::RelationalRecoveryStop::Durability(error) =
        contracts::durability::data::recovery_stop::RelationalRecoveryStop::from(failure)
    else {
        panic!("durability stop")
    };
    assert_eq!(
        error.class,
        contracts::durability::data::recovery_errors::RecoveryFailureClass::ColdStorageUnavailable
    );
    assert_eq!(error.storage_unavailable, Some(failure));
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 0);
}
