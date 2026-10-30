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
use std::num::NonZeroU64;
use std::path::Path;

use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
use {
    worth_store::physical_runtime::OrderedJournalPublicationFailure,
    worth_store::physical_runtime::OrderedJournalPublished,
    worth_store::physical_runtime::OrderedJournalResolution,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::open_with_faults, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
use {crate::production_entry::ordered_journals::two_journals::assert_holds, crate::production_entry::ordered_journals::two_journals::digest, crate::production_entry::ordered_journals::two_journals::publish, crate::production_entry::ordered_journals::two_journals::Journal};
use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

#[test]
fn every_media_operation_of_a_branch_transition_recovers_to_the_old_or_new_head() {
    let cases = fault_cases();
    let scheduled = cases.len();
    let exercised = worth_store_test_support::sweep::run(cases, |case| {
        run_case(case.target, case.role, case.ordinal, case.directive)
    });
    assert_eq!(
        exercised.len(),
        scheduled,
        "scheduled transition cases were lost"
    );
    let unresolved_refusals: usize = exercised
        .iter()
        .map(|case| usize::from(case.unresolved_refusal))
        .sum();
    let retries_after_unresolved: usize = exercised
        .into_iter()
        .map(|case| usize::from(case.retry_after_unresolved))
        .sum();
    assert!(
        unresolved_refusals > 0,
        "no indeterminate publication left a serving runtime to refuse its journal"
    );
    assert!(
        retries_after_unresolved > 0,
        "no indeterminate publication was retried after recovery found it absent"
    );
}
