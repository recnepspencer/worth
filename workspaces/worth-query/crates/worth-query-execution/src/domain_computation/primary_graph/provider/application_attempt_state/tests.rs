use super::phase::WorthQueryApplicationAttemptPhase as AttemptPhase;
use super::WorthQueryPrimaryGraphOverlay as Overlay;

fn overlay(identity: &str) -> Overlay {
    Overlay::new(identity.to_owned(), Vec::new())
}

#[test]
fn foreign_overlay_evidence_is_noninterfering_before_exact_discard() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        let mut phase = AttemptPhase::registered();
        phase.stage_overlay(overlay("overlay:owner")).unwrap();

        assert!(phase.discard_overlay("overlay:foreign").is_none());
        assert_eq!(phase.overlay().expect("staged").identity(), "overlay:owner");

        assert!(phase.discard_overlay("overlay:owner").is_some());
        assert!(phase.accepts_overlay());
    });
}
