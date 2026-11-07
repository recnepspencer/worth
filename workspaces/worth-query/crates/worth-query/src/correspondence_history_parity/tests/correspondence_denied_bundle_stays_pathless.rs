use super::*;

#[test]
fn correspondence_denied_bundle_stays_pathless() {
    let envelope = correspondence_denied_envelope();
    let preflight = detail_preflight_bundle();
    let bundle = build_correspondence_historical_parity_bundle(
        &envelope,
        Some(preflight.plan().query().validated_query_digest().clone()),
        Some(preflight.basis().proof().digest().clone()),
    )
    .expect("correspondence denial bundle should build");

    assert_eq!(bundle.parity_variant().as_str(), "correspondence_denied");
    assert!(bundle.failure_digest().is_some());
    assert!(bundle.requested_path_digest().is_none());
    assert!(bundle.admitted_path_digest().is_none());
    assert!(bundle.resolved_path_digest().is_none());
    assert!(bundle.historical_cost_posture_digest().is_none());
    assert!(bundle.historical_compatibility_outcome().is_none());
}
