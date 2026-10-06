//! What a refused read says about recovery's limits. This is the one rule,
//! and the only place that reads a refusal's bound: a limit comes from a
//! budget the caller set, never from an artifact's own ceiling. An artifact
//! larger than its parent or its format admits is damage.

use worth_store::physical_runtime::{
    ExceededFilesystemObservationBound, FilesystemObservationBound, RecoveryDiscoveryFailure,
};

use super::recovery_budget::{ExceededRecoveryLimit, RecoveryAllowance};
use crate::entry::{PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension};

/// The addressed reads a recovery reader may make. Every read is counted
/// before it is made by the limit it belongs to: a manifest entry or block,
/// a WAL segment, or one of the artifacts every recovery reads once. So the
/// reader's own count never binds, and names no limit; observation bytes
/// bound the reads.
pub(crate) const UNCOUNTED_READS: u64 = u64::MAX;

/// The observation bytes a reader was handed ran out, in the reader's own
/// counts: from its first byte, against what it was handed. Every other
/// refusal is not a limit of the reader: requested bytes are the ceiling of
/// one read, and a reader counts no reads or entries of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReaderBytes(ExceededFilesystemObservationBound);

impl ReaderBytes {
    pub(crate) fn of(failure: &RecoveryDiscoveryFailure) -> Option<Self> {
        match failure {
            RecoveryDiscoveryFailure::Limit(past) => match past.dimension() {
                FilesystemObservationBound::ObservationBytes => Some(Self(*past)),
                FilesystemObservationBound::Reads
                | FilesystemObservationBound::Entries
                | FilesystemObservationBound::RequestedBytes => None,
            },
            RecoveryDiscoveryFailure::Damage(_) => None,
        }
    }

    /// The limit recovery ran out of, where the reader was handed what was
    /// left of recovery's declared observation bytes: the rest was observed
    /// before it. `None` where the reader was handed more than recovery
    /// admits, or a count passes every count.
    pub(super) fn in_recovery(
        self,
        limits: &PhysicalRecoveryLimitDeclaration,
    ) -> Option<ExceededRecoveryLimit> {
        RecoveryAllowance::declared(limits, PhysicalRecoveryLimitDimension::ObservationBytes)
            .beside(self.0.observed(), self.0.admitted())
    }
}

/// What one stream read may return: what is left of the caller's budget.
/// Nothing declares a ceiling for the checkpoint stream or a WAL file, so
/// only that budget bounds them. T2b: the stream readers take a grant, and
/// this goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReadCeiling {
    budget: u64,
    budget_left: u64,
}

/// An artifact larger than its own ceiling. Each reader words this damage for
/// the artifact it read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OversizedArtifact;

/// The caller's budget ended at a stream read. `observed` counts from that
/// budget's first byte to this read's last.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PastBudget {
    pub(crate) observed: u64,
    pub(crate) admitted: u64,
}

impl ReadCeiling {
    /// `budget_left` of the caller's `budget` may still be read.
    pub(crate) const fn of_budget_alone(budget: u64, budget_left: u64) -> Self {
        Self {
            budget,
            budget_left,
        }
    }

    /// The byte limit to hand the reader.
    pub(crate) const fn requested(self) -> u64 {
        self.budget_left
    }

    /// The budget's own counts at a read refused for its requested bytes.
    pub(crate) const fn passed(self, past: &ExceededFilesystemObservationBound) -> PastBudget {
        PastBudget {
            observed: (self.budget.saturating_sub(self.budget_left))
                .saturating_add(past.observed()),
            admitted: self.budget,
        }
    }
}

/// A real refusal for tests: `observed` past an allowance of `admitted`.
#[cfg(test)]
pub(crate) fn refused_past(
    bound: FilesystemObservationBound,
    observed: u64,
    admitted: u64,
) -> RecoveryDiscoveryFailure {
    RecoveryDiscoveryFailure::Limit(
        worth_store::physical_runtime::filesystem_observation_limit_for_test(
            bound, observed, admitted,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store::physical_runtime::{ArtifactDamage, RecoveryDiscoveryArtifact};
    use FilesystemObservationBound as Bound;

    const PAGE: u64 = 65_536;

    /// A real refusal of `observed` past an allowance of seven.
    fn refused(observed: u64, bound: Bound) -> RecoveryDiscoveryFailure {
        refused_past(bound, observed, 7)
    }

    #[test]
    fn only_the_readers_own_bytes_are_a_reader_limit() {
        let bytes = ReaderBytes::of(&refused(9, Bound::ObservationBytes)).unwrap();
        // Handed 7 of recovery's 10, the reader needed 9: 12 of 10.
        let limit = bytes
            .in_recovery(&PhysicalRecoveryLimitDeclaration::observing_for_test(10))
            .unwrap();
        assert_eq!(
            (limit.dimension(), limit.observed(), limit.admitted()),
            (PhysicalRecoveryLimitDimension::ObservationBytes, 12, 10),
        );
        // A reader handed more than recovery admits was no part of it.
        assert_eq!(
            bytes.in_recovery(&PhysicalRecoveryLimitDeclaration::observing_for_test(6)),
            None
        );
        for failure in [
            refused(9, Bound::Reads),
            refused(9, Bound::Entries),
            refused(9, Bound::RequestedBytes),
            RecoveryDiscoveryFailure::Damage(ArtifactDamage::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
            }),
        ] {
            assert_eq!(ReaderBytes::of(&failure), None);
        }
    }

    #[test]
    fn a_stream_is_asked_for_what_is_left_of_its_budget() {
        assert_eq!(ReadCeiling::of_budget_alone(9 * PAGE, 5).requested(), 5);
        assert_eq!(ReadCeiling::of_budget_alone(PAGE, PAGE).requested(), PAGE);
    }

    #[test]
    fn a_refused_stream_counts_what_its_budget_had_already_given() {
        let RecoveryDiscoveryFailure::Limit(past) = refused(PAGE, Bound::RequestedBytes) else {
            unreachable!("a real refusal is a limit");
        };
        for left in [3 * PAGE, PAGE, 1] {
            assert_eq!(
                ReadCeiling::of_budget_alone(3 * PAGE, left).passed(&past),
                PastBudget {
                    observed: 3 * PAGE - left + PAGE,
                    admitted: 3 * PAGE,
                },
            );
        }
        assert_eq!(
            ReadCeiling::of_budget_alone(PAGE, 2).passed(&past).observed,
            2 * PAGE - 2,
        );
    }
}
