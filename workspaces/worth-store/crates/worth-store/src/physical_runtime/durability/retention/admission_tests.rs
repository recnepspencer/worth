use super::*;

#[test]
fn growth_cannot_consume_progress_headroom() {
    let profile = PhysicalRetentionProfile::new(100, 4, 40, 1).unwrap();
    let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
    let first = admission.reserve_candidate(1, 60).unwrap();
    let Err(denied) = admission.reserve_candidate(2, 1) else {
        panic!("a second generation cannot consume progress headroom");
    };
    assert_eq!(denied.remaining_bytes, 0);
    assert_eq!(denied.requested_bytes, 1);
    drop(first);
    let shared = admission.reserve_candidate(7, 60).unwrap();
    let again = admission.reserve_candidate(7, 60).unwrap();
    assert_eq!(admission.lock().charged_bytes, 60);
    drop(shared);
    assert_eq!(admission.lock().charged_bytes, 60);
    drop(again);
    assert_eq!(admission.lock().charged_bytes, 0);
    assert!(admission.reserve_candidate(3, 61).is_err());
}
