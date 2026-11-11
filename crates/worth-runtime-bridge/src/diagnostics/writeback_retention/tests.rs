//! Metadata tests exercise the actual weak-slot and incarnation classifier.
use super::*;
use worth_foundational::facade::{HomeIdentity, SignalOwnerSlotIdentity};
fn owner(home: u8) -> DurableSignalOwnerIdentity {
    DurableSignalOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([home; 16]).unwrap(),
        SignalOwnerSlotIdentity::from_recorded(3),
    )
}
#[test]
fn latest_current_archive_has_never_retained_a_record_before_its_first_entry() {
    assert_eq!(
        WritebackDiagnosticScope::default().missing(),
        Absence::NeverRetained
    );
    assert_eq!(
        WritebackDiagnosticScope {
            owner: Some(owner(1)),
        }
        .missing(),
        Absence::NeverRetained
    );
}
#[test]
fn expired_reference_does_not_keep_record_payload_or_owner_alive() {
    let scope = Arc::new(WritebackDiagnosticScope {
        owner: Some(owner(1)),
    });
    let slot = WritebackDiagnosticSlot::new(String::from("record"));
    let reference = slot.reference(&scope);
    assert_eq!(scope.read(&reference).unwrap(), "record");
    slot.expire();
    assert_eq!(
        scope.read(&reference),
        Err(Denial::Absent(Absence::ExpiredUnderCountLimit))
    );
    drop(slot);
    assert!(reference.slot.upgrade().is_none());
    drop(scope);
    assert!(reference.origin.upgrade().is_none());
}
#[test]
fn receipt_annotation_preserves_the_same_retained_reference() {
    let scope = Arc::new(WritebackDiagnosticScope::default());
    let slot = WritebackDiagnosticSlot::new(String::from("unannotated"));
    let reference = slot.reference(&scope);
    slot.replace(String::from("annotated"));
    assert_eq!(scope.read(&reference).unwrap(), "annotated");
}
#[test]
fn reference_from_another_incarnation_is_lost_across_reopen_and_foreign_owner_is_refused() {
    let original = Arc::new(WritebackDiagnosticScope {
        owner: Some(owner(1)),
    });
    let slot = WritebackDiagnosticSlot::new(String::from("record"));
    let reference = slot.reference(&original);
    let reopened = Arc::new(WritebackDiagnosticScope {
        owner: Some(owner(1)),
    });
    assert_eq!(
        reopened.read(&reference),
        Err(Denial::Absent(Absence::NotRetainedAcrossReopen))
    );
    let foreign = Arc::new(WritebackDiagnosticScope {
        owner: Some(owner(2)),
    });
    assert_eq!(foreign.read(&reference), Err(Denial::ForeignOwner));
    let unadmitted = Arc::new(WritebackDiagnosticScope::default());
    assert_eq!(unadmitted.read(&reference), Err(Denial::ForeignOwner));
}
