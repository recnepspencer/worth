//! A historical result's checks run past a bound they were handed are that
//! limit of recovery's, and only a refusal without one is damage.

use worth_store_recovery_physics::{test_support::root_history_limit_for_test, RootHistoryBound};

use super::{transition_refused, HistoricalFailure, PageLimit};
use crate::entry::PhysicalRecoveryLimitDeclaration;
use crate::entry::PhysicalRecoveryLimitDimension::{ManifestEntries, StagingBytes};
use crate::orchestration::recovery_budget::recovery_limit_for_test;

/// Recovery declared 8 manifest entries and 64 staging bytes, and one of
/// every other count.
fn declared() -> PhysicalRecoveryLimitDeclaration {
    let mut limits = PhysicalRecoveryLimitDeclaration::from_values_for_test([1; 19]);
    limits.manifest_entries = 8;
    limits.staging_bytes = 64;
    limits
}

#[test]
fn a_bound_the_checks_ran_past_is_the_declared_limit_it_was_handed() {
    let limits = declared();
    let entries = root_history_limit_for_test(RootHistoryBound::Entries, 9, 8);
    assert_eq!(
        transition_refused(Some(entries), &limits),
        HistoricalFailure::Limit(PageLimit::Recovery(recovery_limit_for_test(
            ManifestEntries,
            9,
            8
        ))),
    );
    let scratch = root_history_limit_for_test(RootHistoryBound::ScratchBytes, 80, 64);
    assert_eq!(
        transition_refused(Some(scratch), &limits),
        HistoricalFailure::Limit(PageLimit::Recovery(recovery_limit_for_test(
            StagingBytes,
            80,
            64
        ))),
    );
    assert_eq!(
        transition_refused(None, &limits),
        HistoricalFailure::Invalid
    );
}
