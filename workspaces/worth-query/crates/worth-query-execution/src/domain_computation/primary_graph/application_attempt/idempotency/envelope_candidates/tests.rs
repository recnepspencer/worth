//! Performed handler DATA fills only an absent definition, never request intent.
use super::*;

#[test]
fn recorded_handler_definition_preserves_fresh_intent_and_explicit_mismatch() {
    let fresh = Binding::new([1; 32], [2; 32]);
    let recorded = fresh
        .bind_conditional_definition(Some(&[3; 32]))
        .intent_text();
    assert!(!EnvelopeCandidates::new(&[fresh])
        .unwrap()
        .match_text(&recorded)
        .unwrap());
    assert!(EnvelopeCandidates::new(&[fresh])
        .unwrap()
        .match_performed_intent(&recorded)
        .unwrap());
    for changed in [
        Binding::new([1; 32], [4; 32]),
        fresh.bind_conditional_definition(Some(&[5; 32])),
        fresh.bind_governed_input(Some(&[6; 32])),
    ] {
        assert!(!EnvelopeCandidates::new(&[changed])
            .unwrap()
            .match_performed_intent(&recorded)
            .unwrap());
    }
    assert!(EnvelopeCandidates::new(&[fresh])
        .unwrap()
        .match_performed_intent("malformed")
        .is_err());
}
