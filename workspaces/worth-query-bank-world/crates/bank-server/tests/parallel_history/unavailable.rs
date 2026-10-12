//! Bank's one producer cannot supply independent computing-member witnesses.
use std::collections::BTreeSet;
const REASON: &str = "one producer, no independent computing-member wave: bank-server/src/application_definition/providers.rs:82,103; transfers are ordinary mutations";

#[test]
fn bank_unavailable_axes_are_named_rows() {
    let mut rows = BTreeSet::new();
    for posture in [0, 1, 2, 5] {
        for history in ["transfer", "assessment"] {
            for law in [
                "span-below-work",
                "overlap-witness",
                "completion-order-mutant",
            ] {
                assert!(
                    rows.insert((posture, history, law)),
                    "each unavailable Bank row has its own name"
                );
                println!("BANK_NOT_RUN:{history}:width={posture}:{law}:{REASON}");
            }
        }
    }
    assert_eq!(
        rows.len(),
        4 * 2 * 3,
        "all Bank unavailable axes are counted"
    );
}
