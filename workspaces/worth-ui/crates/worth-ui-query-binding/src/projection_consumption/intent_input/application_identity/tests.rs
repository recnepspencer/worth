use super::ApplicationProjectionInputIdentity;
use worth_query_host::facade::publication::domain_computation::WorthQueryApplicationQueryPublicationReceipt;

// Reference predicate used before the cutover: compare the genuine receipts,
// independently of the captured UI correlation representation.
fn receipt_authority_matches(
    left: &WorthQueryApplicationQueryPublicationReceipt,
    right: &WorthQueryApplicationQueryPublicationReceipt,
) -> bool {
    let left = left.inspect();
    let right = right.inspect();
    left.query_identity() == right.query_identity()
        && left.parameter_binding_identity() == right.parameter_binding_identity()
        && left.basis().runtime_instance() == right.basis().runtime_instance()
        && left.basis().branch() == right.basis().branch()
}

#[test]
fn application_revision_identity_matches_receipts_across_edits_and_foreign_owners() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<crate::UiProjectionInputRevision>();
    send_sync::<crate::UiProjectionOptionReference>();
    let owner = crate::WorthUiStatusSourceOwner::install().unwrap();
    let foreign = crate::WorthUiStatusSourceOwner::install().unwrap();
    let before = owner.read_status().unwrap();
    let after = owner
        .publish_source(
            crate::WorthUiScalarProjectionSourceRecord::new("ONLINE", 1).unwrap(),
            std::num::NonZeroUsize::new(64),
        )
        .unwrap();
    let other = foreign.read_status().unwrap();
    let original = ApplicationProjectionInputIdentity::capture(before.query_receipt());
    for receipt in [
        before.query_receipt(),
        after.query_receipt(),
        other.query_receipt(),
    ] {
        let captured = ApplicationProjectionInputIdentity::capture(receipt);
        assert_eq!(
            original.has_same_authority(&captured),
            receipt_authority_matches(before.query_receipt(), receipt),
        );
    }
    assert!(
        original.has_same_authority(&ApplicationProjectionInputIdentity::capture(
            after.query_receipt()
        ),)
    );
    assert_ne!(
        original,
        ApplicationProjectionInputIdentity::capture(after.query_receipt())
    );
    assert!(
        !original.has_same_authority(&ApplicationProjectionInputIdentity::capture(
            other.query_receipt()
        ),)
    );
    drop((before, after, other));
    assert!(owner.close().owner_terminal());
    assert!(foreign.close().owner_terminal());
}
