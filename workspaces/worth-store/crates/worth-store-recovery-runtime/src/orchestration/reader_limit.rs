//! What a refused read says about recovery's limits. This is the one rule,
//! and the only place that reads a refusal's bound: a limit comes from a
//! budget the caller set, never from an artifact's own ceiling. An artifact
//! larger than its parent or its format admits is damage.

use worth_store::physical_runtime::{
    ExceededFilesystemObservationBound, FilesystemObservationBound, RecoveryDiscoveryFailure,
};
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

use super::recovery_budget::{ExceededRecoveryLimit, RecoveryAllowance};
use crate::entry::{PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension};

/// The addressed reads a recovery reader may make. Every read is counted
/// before it is made by the limit it belongs to: a manifest entry or block,
/// a WAL segment, or one of the artifacts every recovery reads once. So the
/// reader's own count never binds, and names no limit; observation bytes
/// bound the reads.
pub(crate) const UNCOUNTED_READS: u64 = u64::MAX;

/// The ceiling of one read of an extent arena: its manifest and each of its
/// chunk frames fit one page. A frame past it is the frame's own damage; the
/// reader's allowance is the only budget, and it says when it ran out.
pub(crate) fn extent_page_ceiling(format: PhysicalRecordFormatDeclaration) -> u64 {
    u64::from(format.page_size().bytes())
}

/// The observation bytes a reader was handed ran out, in the reader's own
/// counts: from its first byte, against what it was handed. Every other
/// refusal is not a limit of the reader: requested bytes are the ceiling of
/// one read, and a reader counts no reads or entries of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReaderBytes(ExceededFilesystemObservationBound);

impl ReaderBytes {
    pub(crate) fn of(failure: &RecoveryDiscoveryFailure) -> Option<Self> {
        match failure {
            RecoveryDiscoveryFailure::Limit(past)
                if past.dimension() == FilesystemObservationBound::ObservationBytes =>
            {
                Some(Self(*past))
            }
            _ => None,
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

/// The most one whole-artifact read may return: the artifact's own ceiling,
/// from the parent that declares its length or else from its format, and
/// what is left of a budget the caller put under the read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReadCeiling {
    artifact: u64,
    budget: u64,
    budget_left: u64,
}

/// An artifact larger than its own ceiling. Each reader words this damage for
/// the artifact it read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OversizedArtifact;

/// What a read met when the reader refused it for its own ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PastCeiling {
    /// The artifact is larger than its own ceiling: damage.
    Artifact,
    /// The artifact may fit its ceiling, and the caller's budget ended first.
    /// `observed` counts from that budget's first byte to this read's last.
    Budget { observed: u64, admitted: u64 },
}

impl ReadCeiling {
    pub(crate) const fn of_artifact(artifact: u64) -> Self {
        Self::within(artifact, u64::MAX, u64::MAX)
    }

    /// `budget_left` of the caller's `budget` may still be read.
    pub(crate) const fn within(artifact: u64, budget: u64, budget_left: u64) -> Self {
        Self {
            artifact,
            budget,
            budget_left,
        }
    }

    /// An artifact nothing declares a ceiling for: neither a parent nor its
    /// format bounds the checkpoint stream, nor a WAL file. Only the caller's
    /// budget does.
    pub(crate) const fn of_budget_alone(budget: u64, budget_left: u64) -> Self {
        Self::within(u64::MAX, budget, budget_left)
    }

    /// The byte limit to hand the reader.
    pub(crate) const fn requested(self) -> u64 {
        if self.budget_left < self.artifact {
            self.budget_left
        } else {
            self.artifact
        }
    }

    /// `None` where the reader did not refuse the read for this ceiling.
    /// The reader reports the artifact's length, or one byte past what it
    /// was asked for where it cannot tell: only a length within the
    /// artifact's own ceiling leaves the caller's budget as what ended.
    pub(crate) const fn passed(self, failure: &RecoveryDiscoveryFailure) -> Option<PastCeiling> {
        let RecoveryDiscoveryFailure::Limit(past) = failure else {
            return None;
        };
        if !matches!(past.dimension(), FilesystemObservationBound::RequestedBytes) {
            return None;
        }
        Some(if past.observed() > self.artifact {
            PastCeiling::Artifact
        } else {
            PastCeiling::Budget {
                observed: (self.budget.saturating_sub(self.budget_left))
                    .saturating_add(past.observed()),
                admitted: self.budget,
            }
        })
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
    use worth_store::physical_runtime::RecoveryDiscoveryArtifact;
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
            RecoveryDiscoveryFailure::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
            },
        ] {
            assert_eq!(ReaderBytes::of(&failure), None);
        }
    }

    #[test]
    fn the_reader_is_asked_for_the_smaller_of_the_ceiling_and_the_budget() {
        assert_eq!(ReadCeiling::of_artifact(PAGE).requested(), PAGE);
        assert_eq!(
            ReadCeiling::within(PAGE, 9 * PAGE, PAGE - 1).requested(),
            PAGE - 1
        );
        assert_eq!(ReadCeiling::within(PAGE, 9 * PAGE, PAGE).requested(), PAGE);
        assert_eq!(
            ReadCeiling::within(PAGE, 9 * PAGE, PAGE + 1).requested(),
            PAGE
        );
        assert_eq!(ReadCeiling::of_budget_alone(9 * PAGE, 5).requested(), 5);
    }

    #[test]
    fn an_artifact_past_its_own_ceiling_is_damage_under_any_budget() {
        let requested = Bound::RequestedBytes;
        for left in [3 * PAGE, PAGE, 1] {
            let ceiling = ReadCeiling::within(PAGE, 3 * PAGE, left);
            assert_eq!(
                ceiling.passed(&refused(PAGE + 1, requested)),
                Some(PastCeiling::Artifact),
            );
            // A whole page fits the artifact's ceiling, so the budget ended,
            // and the report counts what the budget had already given.
            assert_eq!(
                ceiling.passed(&refused(PAGE, requested)),
                Some(PastCeiling::Budget {
                    observed: 3 * PAGE - left + PAGE,
                    admitted: 3 * PAGE,
                }),
            );
        }
        assert_eq!(
            ReadCeiling::of_artifact(PAGE).passed(&refused(PAGE + 1, requested)),
            Some(PastCeiling::Artifact),
        );
        assert_eq!(
            ReadCeiling::of_budget_alone(PAGE, 2).passed(&refused(u64::MAX, requested)),
            Some(PastCeiling::Budget {
                observed: u64::MAX,
                admitted: PAGE,
            }),
        );
    }

    #[test]
    fn a_refusal_for_anything_but_the_ceiling_passes_none() {
        let ceiling = ReadCeiling::within(PAGE, PAGE, 1);
        for failure in [
            refused(PAGE + 1, Bound::ObservationBytes),
            refused(PAGE + 1, Bound::Reads),
            RecoveryDiscoveryFailure::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
            },
        ] {
            assert_eq!(ceiling.passed(&failure), None);
        }
    }
}
