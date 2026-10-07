//! A Ready and real membership cannot substitute for an admitted source.
use super::*;

#[test]
fn a_cached_ready_without_its_source_cannot_claim_requested_execution() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let (interest, _) = registry.fixture_admitted_work_membership(occurrence());
    let key = interest.key.clone();
    registry
        .state
        .lock()
        .unwrap()
        .records
        .get_mut(&key)
        .unwrap()
        .state = super::closed_retirement::ready();
    drop(interest);
    let before = registry.state.lock().unwrap().required_reserved_bytes;
    assert!(registry
        .claim_cached_requested_ready(&key, &mut record_admission())
        .unwrap()
        .is_none());
    let state = registry.state.lock().unwrap();
    assert!(state.records[&key].has_cached_ready());
    assert!(!state.records[&key].is_required());
    assert!(state.records[&key].readmission_source.is_none());
    assert!(!state.required_keys.contains(&key));
    assert_eq!(state.required_reserved_bytes, before);
}
