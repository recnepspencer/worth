//! Crash-consistency oracle for one branch-transitions publication published
//! after canonical commits.
//!
//! For the journal's first entry and a later one, the oracle counts the media
//! operations of every role the publication performs, then reruns it once per
//! (role, ordinal, directive) with a fault at exactly that operation, abandons
//! the runtime, recovers in the production path and requires: the transitions
//! journal holds exactly the old head or the new one (an acknowledged entry is
//! never lost, a proven-no-effect one never appears); an indeterminate
//! publication blocks its journal until reopen; the canonical commits are
//! untouched; the same publication retried after recovery is answered from
//! durable state, never by the dead runtime's attempt; the suffix publishes;
//! and a second recovery agrees.

#[allow(unused_imports)]
use crate::production_entry::ordered_journals::branch_transitions_oracle::*;

#[test]
fn every_media_operation_of_a_branch_transition_recovers_to_the_old_or_new_head() {
    let mut unresolved_refusals = 0;
    let mut retries_after_unresolved = 0;
    for target in TARGETS {
        let counts = measure(prepared_store(target).path(), target);
        assert!(
            !counts.is_empty(),
            "the publication performed media operations"
        );
        for (role, count) in counts {
            for nth in 1..=count {
                let nth = std::num::NonZeroU64::new(nth).unwrap();
                for directive in [
                    worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
                        kind: std::io::ErrorKind::Other,
                        raw_os_error: None,
                    },
                    worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
                ] {
                    let exercised = run_case(target, role, nth, directive);
                    unresolved_refusals += usize::from(exercised.unresolved_refusal);
                    retries_after_unresolved += usize::from(exercised.retry_after_unresolved);
                }
            }
        }
    }
    assert!(
        unresolved_refusals > 0,
        "no indeterminate publication left a serving runtime to refuse its journal"
    );
    assert!(
        retries_after_unresolved > 0,
        "no indeterminate publication was retried after recovery found it absent"
    );
}
