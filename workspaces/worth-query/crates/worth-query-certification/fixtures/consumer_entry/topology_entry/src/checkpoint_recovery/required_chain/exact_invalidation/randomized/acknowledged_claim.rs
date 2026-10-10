//! A Current queue claim can later be requested as a native upstream.
use super::*;

#[test]
fn an_acknowledged_current_claim_can_be_inspected_as_an_upstream() {
    let _guard = checkpoint_recovery_test_guard();
    let mut driven = Driven::default();
    // The first seven seeded steps include a replaced successor, a racing
    // root input and a later fetch-only write. Each step is judged from its
    // declared inputs by the same independent courtroom as the full history.
    let steps = 7;
    run(0x9176_3c02, AMPLE_OBSERVATIONS, steps, &mut driven);
    assert_eq!(driven.commits.iter().sum::<usize>(), steps);
}
