use std::collections::BTreeSet;

use super::super::transition::STATE_CHANGING_OPERATIONS;
use super::trace_support::ORACLE_SEED;

#[path = "adjacency_roster/court.rs"]
mod court;
#[path = "adjacency_roster/operation.rs"]
mod operation;
#[path = "adjacency_roster/outcome.rs"]
mod outcome;

#[test]
fn every_ordered_state_changing_pair_reaches_real_owner_and_oracle() {
    // Close and capability loss are lifecycle controls rather than public
    // methods: dropping the real runtime is their owner boundary. A second
    // terminal control therefore has the contractually stable unavailable
    // posture; all other cells call a public weak port twice.
    let mut seen = BTreeSet::new();

    for (first_index, first) in STATE_CHANGING_OPERATIONS.iter().copied().enumerate() {
        for (second_index, second) in STATE_CHANGING_OPERATIONS.iter().copied().enumerate() {
            let mut court = court::PairCourt::new(first_index, second_index);
            court.apply_pair_operation(first, 0);
            court.apply_pair_operation(second, 1);
            assert!(
                seen.insert((first, second)),
                "seed {ORACLE_SEED:#x}: duplicate adjacent pair {first:?}->{second:?}"
            );
        }
    }

    let expected_count = STATE_CHANGING_OPERATIONS.len().pow(2);
    assert_eq!(
        seen.len(),
        expected_count,
        "seed {ORACLE_SEED:#x}: adjacency court did not complete every ordered pair"
    );
    for first in STATE_CHANGING_OPERATIONS {
        for second in STATE_CHANGING_OPERATIONS {
            assert!(
                seen.contains(&(first, second)),
                "seed {ORACLE_SEED:#x}: missing ordered pair {first:?}->{second:?}"
            );
        }
    }
}
