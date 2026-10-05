//! What a refused read says about recovery's limits. This is the one rule,
//! and the only place that reads a refusal's scope: a limit comes from a
//! budget the caller set, never from an artifact's own ceiling. An artifact
//! larger than its parent or its format admits is damage.

use worth_store::physical_runtime::{RecoveryDiscoveryByteLimitScope, RecoveryDiscoveryFailure};
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

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

/// The reader's own allowance ran out. Every other refusal is not a limit of
/// the reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReaderLimit {
    /// The reads its reader admitted ran out.
    Reads { observed: u64, admitted: u64 },
    /// The observation bytes ran out. `observed` counts from the reader's
    /// first byte to the read that crossed.
    ObservationBytes { observed: u64, admitted: u64 },
}

impl ReaderLimit {
    pub(crate) const fn of(failure: &RecoveryDiscoveryFailure) -> Option<Self> {
        match failure {
            RecoveryDiscoveryFailure::ByteLimitExceeded {
                observed,
                admitted,
                scope,
            } => Self::of_bytes(*observed, *admitted, *scope),
            RecoveryDiscoveryFailure::EntryLimitExceeded { observed, admitted } => {
                Some(Self::Reads {
                    observed: *observed,
                    admitted: *admitted,
                })
            }
            _ => None,
        }
    }

    /// Bytes a reader refused. Only its observation allowance is its own:
    /// `Requested` is the ceiling of the one read.
    pub(crate) const fn of_bytes(
        observed: u64,
        admitted: u64,
        scope: RecoveryDiscoveryByteLimitScope,
    ) -> Option<Self> {
        match scope {
            RecoveryDiscoveryByteLimitScope::Observation => {
                Some(Self::ObservationBytes { observed, admitted })
            }
            RecoveryDiscoveryByteLimitScope::Requested => None,
        }
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
        match failure {
            RecoveryDiscoveryFailure::ByteLimitExceeded {
                observed,
                scope: RecoveryDiscoveryByteLimitScope::Requested,
                ..
            } => Some(if *observed > self.artifact {
                PastCeiling::Artifact
            } else {
                PastCeiling::Budget {
                    observed: (self.budget.saturating_sub(self.budget_left))
                        .saturating_add(*observed),
                    admitted: self.budget,
                }
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store::physical_runtime::RecoveryDiscoveryArtifact;

    const PAGE: u64 = 65_536;

    fn refused(observed: u64, scope: RecoveryDiscoveryByteLimitScope) -> RecoveryDiscoveryFailure {
        RecoveryDiscoveryFailure::ByteLimitExceeded {
            observed,
            admitted: 7,
            scope,
        }
    }

    #[test]
    fn only_the_readers_own_allowance_is_a_reader_limit() {
        assert_eq!(
            ReaderLimit::of(&refused(9, RecoveryDiscoveryByteLimitScope::Observation)),
            Some(ReaderLimit::ObservationBytes {
                observed: 9,
                admitted: 7,
            }),
        );
        assert_eq!(
            ReaderLimit::of(&RecoveryDiscoveryFailure::EntryLimitExceeded {
                observed: 3,
                admitted: 2,
            }),
            Some(ReaderLimit::Reads {
                observed: 3,
                admitted: 2,
            }),
        );
        for failure in [
            refused(9, RecoveryDiscoveryByteLimitScope::Requested),
            RecoveryDiscoveryFailure::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
            },
        ] {
            assert_eq!(ReaderLimit::of(&failure), None);
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
        let requested = RecoveryDiscoveryByteLimitScope::Requested;
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
            refused(PAGE + 1, RecoveryDiscoveryByteLimitScope::Observation),
            RecoveryDiscoveryFailure::EntryLimitExceeded {
                observed: 1,
                admitted: 0,
            },
            RecoveryDiscoveryFailure::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
            },
        ] {
            assert_eq!(ceiling.passed(&failure), None);
        }
    }
}
