//! Why the ordered walk produced no verified history.

use worth_store::physical_runtime::RecoveryDiscoveryFailure;

use worth_store_recovery_physics::{
    AddressedReleasedControlDenial, ExceededRootHistoryBound, RootHistoryBound,
    SelectedReleaseHeadReplayDenial,
};

use crate::entry::PhysicalRecoverySelectedRecordReadDenial;
use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;
use crate::orchestration::planning::page_observation::PageObservationFailure;

/// A walk that ran out of an admitted limit says nothing about the history. A
/// walk that failed verification proves the media does not hold the selected
/// root's history. Only the second may be read as damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::orchestration::planning::page_observation) enum WalkFailure {
    ManifestEntryLimit,
    ByteLimit,
    /// The walk's own scratch allowance, admitted as staging bytes. The walk
    /// needed at least this many in all.
    ScratchLimit {
        at_least: u64,
    },
    Unverified,
}

impl WalkFailure {
    /// A step that knows only that it needed more scratch than it had left.
    pub(in crate::orchestration::planning::page_observation) const MORE_SCRATCH: Self =
        Self::ScratchLimit { at_least: 0 };

    /// A physics check the walk asked for was refused. `exceeded` is the
    /// bound it ran past, if that is why. Physics is handed the scratch the
    /// walk has left of `maximum_scratch`, so the walk holds the rest.
    pub(in crate::orchestration::planning::page_observation) fn refused(
        exceeded: Option<ExceededRootHistoryBound>,
        budget: &mut ManifestEntryBudget,
        maximum_scratch: u64,
    ) -> Self {
        match exceeded {
            None => Self::Unverified,
            Some(past) => match past.bound {
                RootHistoryBound::Entries => budget.refuse_view(past.observed).into(),
                RootHistoryBound::ScratchBytes => {
                    Self::past_scratch(past.observed, past.admitted, maximum_scratch)
                }
            },
        }
    }

    /// A head replay physics refused. Either bound it names is scratch the
    /// walk had left of `maximum_scratch`.
    pub(in crate::orchestration::planning::page_observation) fn replay_refused(
        denial: SelectedReleaseHeadReplayDenial,
        maximum_scratch: u64,
    ) -> Self {
        match denial {
            SelectedReleaseHeadReplayDenial::BoundExceeded(past) => {
                Self::past_scratch(past.observed, past.admitted, maximum_scratch)
            }
            _ => Self::Unverified,
        }
    }

    /// A check handed `admitted` of the walk's scratch needed `observed`:
    /// the walk held the rest, and so needed that much more in all.
    const fn past_scratch(observed: u64, admitted: u64, maximum_scratch: u64) -> Self {
        Self::ScratchLimit {
            at_least: observed.saturating_add(maximum_scratch.saturating_sub(admitted)),
        }
    }

    /// The admitted limit the walk ran out of, as observation reports it.
    pub(in crate::orchestration::planning::page_observation) const fn limit(
        self,
    ) -> Option<PageObservationFailure> {
        match self {
            Self::ManifestEntryLimit => Some(PageObservationFailure::ManifestEntryLimit),
            Self::ByteLimit => Some(PageObservationFailure::ByteLimit),
            Self::ScratchLimit { at_least } => {
                Some(PageObservationFailure::StagingByteLimit { at_least })
            }
            Self::Unverified => None,
        }
    }
}

/// Narrows what a walk step was refused to the walk's verdict, where the
/// refusal itself does not say which it is.
pub(in crate::orchestration::planning::page_observation) trait Verdict<T> {
    /// Absence or refusal here is failed verification.
    fn proven(self) -> Result<T, WalkFailure>;
    /// Absence or refusal here is the scratch allowance running out.
    fn in_scratch(self) -> Result<T, WalkFailure>;
}

impl<T> Verdict<T> for Option<T> {
    fn proven(self) -> Result<T, WalkFailure> {
        self.ok_or(WalkFailure::Unverified)
    }

    fn in_scratch(self) -> Result<T, WalkFailure> {
        self.ok_or(WalkFailure::MORE_SCRATCH)
    }
}

impl<T, E> Verdict<T> for Result<T, E> {
    fn proven(self) -> Result<T, WalkFailure> {
        self.map_err(|_| WalkFailure::Unverified)
    }

    fn in_scratch(self) -> Result<T, WalkFailure> {
        self.map_err(|_| WalkFailure::MORE_SCRATCH)
    }
}

impl From<PageObservationFailure> for WalkFailure {
    fn from(failure: PageObservationFailure) -> Self {
        match failure {
            PageObservationFailure::ManifestEntryLimit => Self::ManifestEntryLimit,
            PageObservationFailure::ByteLimit => Self::ByteLimit,
            PageObservationFailure::StagingByteLimit { at_least } => {
                Self::ScratchLimit { at_least }
            }
            _ => Self::Unverified,
        }
    }
}

impl From<PhysicalRecoverySelectedRecordReadDenial> for WalkFailure {
    /// A control record the walk could not read. Its resident bytes are
    /// admitted out of the walk's scratch.
    fn from(denial: PhysicalRecoverySelectedRecordReadDenial) -> Self {
        use PhysicalRecoverySelectedRecordReadDenial as Denial;
        match denial {
            Denial::ManifestRead(failure) | Denial::ChunkRead { failure, .. } => failure.into(),
            Denial::ManifestEntryLimit => Self::ManifestEntryLimit,
            Denial::ResidentBoundExceeded => Self::MORE_SCRATCH,
            _ => Self::Unverified,
        }
    }
}

impl From<AddressedReleasedControlDenial> for WalkFailure {
    /// The bound is the bytes a retained control frame may take.
    fn from(denial: AddressedReleasedControlDenial) -> Self {
        match denial {
            AddressedReleasedControlDenial::Bound => Self::MORE_SCRATCH,
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
            PageObservationFailure::StagingByteLimit { at_least: 9 },
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

    #[test]
    fn a_refusal_is_the_verdict_its_caller_names() {
        assert_eq!(Some(7).proven(), Ok(7));
        assert_eq!(Ok::<u8, ()>(7).in_scratch(), Ok(7));
        assert_eq!(None::<u8>.proven(), Err(WalkFailure::Unverified));
        assert_eq!(None::<u8>.in_scratch(), Err(WalkFailure::MORE_SCRATCH));
        assert_eq!(Err::<u8, ()>(()).proven(), Err(WalkFailure::Unverified));
        assert_eq!(
            Err::<u8, ()>(()).in_scratch(),
            Err(WalkFailure::MORE_SCRATCH)
        );
    }

    #[test]
    fn a_bound_physics_ran_past_is_that_limit_with_what_it_needed() {
        let mut budget = ManifestEntryBudget::new(10, 4);
        let past = |bound, observed, admitted| {
            Some(ExceededRootHistoryBound {
                bound,
                observed,
                admitted,
            })
        };
        assert_eq!(
            WalkFailure::refused(None, &mut budget, 100),
            WalkFailure::Unverified
        );
        assert_eq!(budget.refused_at(), None);
        // Physics had 60 of the walk's 100 bytes and needed 70: 110 in all.
        let scratch = past(RootHistoryBound::ScratchBytes, 70, 60);
        assert_eq!(
            WalkFailure::refused(scratch, &mut budget, 100),
            WalkFailure::ScratchLimit { at_least: 110 }
        );
        assert_eq!(budget.refused_at(), None);
        let entries = past(RootHistoryBound::Entries, 11, 10);
        assert_eq!(
            WalkFailure::refused(entries, &mut budget, 100),
            WalkFailure::ManifestEntryLimit
        );
        assert_eq!(budget.refused_at(), Some(11));
    }

    #[test]
    fn a_unit_bound_is_scratch_and_any_other_refusal_is_unverified() {
        use AddressedReleasedControlDenial as Control;
        assert_eq!(WalkFailure::from(Control::Bound), WalkFailure::MORE_SCRATCH);
        assert_eq!(WalkFailure::from(Control::Frame), WalkFailure::Unverified);
    }

    #[test]
    fn a_replay_bound_is_the_scratch_the_walk_needed_in_all() {
        use worth_store_recovery_physics::{ExceededHeadReplayBound, HeadReplayBound};
        use SelectedReleaseHeadReplayDenial as Replay;
        // Handed 60 of 100 bytes, the replay needed 70: the walk held 40.
        for bound in [HeadReplayBound::EffectBytes, HeadReplayBound::HeapBytes] {
            let past = ExceededHeadReplayBound {
                bound,
                observed: 70,
                admitted: 60,
            };
            assert_eq!(
                WalkFailure::replay_refused(Replay::BoundExceeded(past), 100),
                WalkFailure::ScratchLimit { at_least: 110 }
            );
        }
        assert_eq!(
            WalkFailure::replay_refused(Replay::SourcePath, 100),
            WalkFailure::Unverified
        );
    }

    #[test]
    fn a_control_record_the_walk_could_not_read_keeps_its_limit() {
        use PhysicalRecoverySelectedRecordReadDenial as Denial;
        let out_of_bytes = RecoveryDiscoveryFailure::ByteLimitExceeded {
            observed: 65_537,
            admitted: 65_536,
            scope: RecoveryDiscoveryByteLimitScope::Observation,
        };
        for (denial, failure) in [
            (Denial::ManifestEntryLimit, WalkFailure::ManifestEntryLimit),
            (Denial::ResidentBoundExceeded, WalkFailure::MORE_SCRATCH),
            (Denial::ManifestRead(out_of_bytes), WalkFailure::ByteLimit),
            (Denial::InvalidPayload, WalkFailure::Unverified),
        ] {
            assert_eq!(WalkFailure::from(denial), failure);
        }
    }
}
