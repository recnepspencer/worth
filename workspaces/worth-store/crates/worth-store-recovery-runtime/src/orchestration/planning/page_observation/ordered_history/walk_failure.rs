//! Why the ordered walk produced no verified history.

use worth_store::physical_runtime::RecoveryDiscoveryFailure;

use crate::orchestration::planning::page_observation::PageObservationFailure;

/// A walk that ran out of an admitted limit says nothing about the history. A
/// walk that failed verification proves the media does not hold the selected
/// root's history. Only the second may be read as damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::orchestration::planning::page_observation) enum WalkFailure {
    ManifestEntryLimit,
    ByteLimit,
    Unverified,
}

impl WalkFailure {
    /// The admitted limit the walk ran out of, as observation reports it.
    pub(in crate::orchestration::planning::page_observation) const fn limit(
        self,
    ) -> Option<PageObservationFailure> {
        match self {
            Self::ManifestEntryLimit => Some(PageObservationFailure::ManifestEntryLimit),
            Self::ByteLimit => Some(PageObservationFailure::ByteLimit),
            Self::Unverified => None,
        }
    }
}

impl From<PageObservationFailure> for WalkFailure {
    fn from(failure: PageObservationFailure) -> Self {
        match failure {
            PageObservationFailure::ManifestEntryLimit => Self::ManifestEntryLimit,
            PageObservationFailure::ByteLimit => Self::ByteLimit,
            _ => Self::Unverified,
        }
    }
}

impl From<RecoveryDiscoveryFailure> for WalkFailure {
    /// Observation's own rule says which failed read is a limit.
    fn from(failure: RecoveryDiscoveryFailure) -> Self {
        PageObservationFailure::media(None, failure).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store::physical_runtime::{
        RecoveryDiscoveryArtifact, RecoveryDiscoveryByteLimitScope,
    };
    use worth_store_recovery_physics::PhysicalRedoTargetIdentity;

    #[test]
    fn a_failed_read_is_a_limit_only_when_the_observation_budget_ran_out() {
        let oversized = |scope| RecoveryDiscoveryFailure::ByteLimitExceeded {
            observed: 65_537,
            admitted: 65_536,
            scope,
        };
        assert_eq!(
            WalkFailure::from(oversized(RecoveryDiscoveryByteLimitScope::Observation)),
            WalkFailure::ByteLimit,
        );
        assert_eq!(
            WalkFailure::from(RecoveryDiscoveryFailure::EntryLimitExceeded {
                observed: 1,
                admitted: 0,
            }),
            WalkFailure::ManifestEntryLimit,
        );
        for damage in [
            // A root or routing block larger than one page is damaged media.
            oversized(RecoveryDiscoveryByteLimitScope::Requested),
            RecoveryDiscoveryFailure::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
            },
        ] {
            assert_eq!(WalkFailure::from(damage), WalkFailure::Unverified);
        }
    }

    #[test]
    fn only_an_exhausted_limit_is_reported_as_a_limit() {
        for limit in [
            PageObservationFailure::ManifestEntryLimit,
            PageObservationFailure::ByteLimit,
        ] {
            assert_eq!(WalkFailure::from(limit.clone()).limit(), Some(limit));
        }
        let target = PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page: 2,
            generation: 3,
        };
        for damage in [
            PageObservationFailure::InvalidTarget(target),
            PageObservationFailure::InvalidPage(target),
        ] {
            assert_eq!(WalkFailure::from(damage), WalkFailure::Unverified);
        }
        assert_eq!(WalkFailure::Unverified.limit(), None);
    }
}
