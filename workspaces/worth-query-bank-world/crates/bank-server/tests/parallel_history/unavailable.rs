//! Named capability rows are data, not a self-cardinality test.
pub(super) fn rows() {
    for posture in [0, 1, 2, 5] {
        for history in ["transfer", "assessment"] {
            for law in [
                "span-below-work",
                "overlap-witness",
                "completion-order-mutant",
            ] {
                let reason = match (history, law) {
                    ("transfer", "span-below-work") => "ordinary mutation has no independent required-producer wave, so this span law has no two branches",
                    ("transfer", "overlap-witness") => "ordinary transfer computes no required producer members to overlap",
                    ("transfer", "completion-order-mutant") => "ordinary transfer has no independent producer completion order to permute",
                    ("assessment", "span-below-work") => "one assessment producer has no independent sibling in this required wave",
                    ("assessment", "overlap-witness") => "one assessment producer cannot overlap two computing members",
                    ("assessment", "completion-order-mutant") => "one assessment producer has no pair whose completion order differs from canonical order",
                    _ => unreachable!(),
                };
                println!("BANK_NOT_RUN:{history}:width={posture}:{law}:{reason}; application_definition/providers.rs:82,103");
            }
        }
    }
}
