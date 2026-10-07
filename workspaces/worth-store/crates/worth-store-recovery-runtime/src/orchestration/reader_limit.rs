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
/// refusal is not a limit of the reader: a reader counts no reads or entries
/// of its own, and a read past its grant is the grant owner's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReaderBytes(ExceededFilesystemObservationBound);

impl ReaderBytes {
    pub(crate) fn of(failure: &RecoveryDiscoveryFailure) -> Option<Self> {
        match failure {
            RecoveryDiscoveryFailure::Limit(past) => match past.dimension() {
                FilesystemObservationBound::ObservationBytes => Some(Self(*past)),
                FilesystemObservationBound::Reads | FilesystemObservationBound::Entries => None,
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
            RecoveryDiscoveryFailure::Damage(ArtifactDamage::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
            }),
        ] {
            assert_eq!(ReaderBytes::of(&failure), None);
        }
    }
}
