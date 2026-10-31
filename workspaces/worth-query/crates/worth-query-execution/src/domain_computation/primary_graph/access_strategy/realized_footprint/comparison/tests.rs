//! Root-free adversarial evidence for the erased fallible comparison boundary.
use super::with_checkpoint;
#[test]
fn swallowed_checkpoint_preserves_first_owned_cause_and_stops_admission() {
    #[derive(Debug, Eq, PartialEq)]
    struct Denial(String);
    let mut contacts = 0;
    let result = with_checkpoint(
        &mut |_| {
            contacts += 1;
            Err(Denial("original-domain-stop".into()))
        },
        &mut |checkpoint| {
            assert!(!checkpoint(32));
            assert!(!checkpoint(64));
            true // hostile erased callee swallows both refusals
        },
    );
    assert_eq!(result, Err(Denial("original-domain-stop".into())));
    assert_eq!(contacts, 1);
}
#[test]
fn final_poll_can_refuse_an_otherwise_equal_or_different_comparison() {
    for equal in [false, true] {
        assert_eq!(
            with_checkpoint(&mut |_| Err("final-stop"), &mut |_| equal),
            Err("final-stop")
        );
    }
}
