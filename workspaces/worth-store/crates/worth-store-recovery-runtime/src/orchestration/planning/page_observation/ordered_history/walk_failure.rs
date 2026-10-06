//! Why the ordered walk produced no verified history.

use worth_store::physical_runtime::RecoveryDiscoveryFailure;

use worth_store_recovery_physics::{
    AddressedReleasedControlDenial, ExceededRootHistoryBound, RootHistoryBound,
    SelectedReleaseHeadReplayDenial,
};

use crate::entry::PhysicalRecoverySelectedRecordReadDenial;
use crate::orchestration::planning::manifest_entry_budget::{
    EntriesStopped, ExceededManifestEntries, ManifestEntryBudget, ViewEntryCap,
};
use crate::orchestration::planning::page_observation::{PageLimit, PageObservationFailure};
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;
use crate::orchestration::recovery_budget::{ExceededRecoveryLimit, RecoveryAllowance};

/// A walk that ran out of an admitted limit says nothing about the history. A
/// walk that failed verification proves the media does not hold the selected
/// root's history. Only the second may be read as damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::orchestration::planning::page_observation) enum WalkFailure {
    Limit(PageLimit),
    /// A count the walk kept passed every count, so no limit can state it.
    CountOverflow,
    Unverified,
}

impl WalkFailure {
    /// A physics check the walk asked for was refused. `exceeded` is the
    /// bound it ran past, if that is why. Physics is handed the scratch the
    /// walk has left of `staging`, so the walk holds the rest.
    pub(in crate::orchestration::planning::page_observation) fn refused(
        exceeded: Option<ExceededRootHistoryBound>,
        budget: &mut ManifestEntryBudget,
        staging: RecoveryAllowance,
    ) -> Self {
        match exceeded {
            None => Self::Unverified,
            Some(past) => match past.dimension() {
                RootHistoryBound::Entries => {
                    let view = ViewEntryCap::of(budget).refuse(past.observed());
                    budget.view_refused(view).into()
                }
                RootHistoryBound::ScratchBytes => {
                    Self::past_scratch(past.observed(), past.admitted(), staging)
                }
            },
        }
    }

    /// A head replay physics refused. Either bound it names is scratch the
    /// walk had left of `staging`.
    pub(in crate::orchestration::planning::page_observation) fn replay_refused(
        denial: SelectedReleaseHeadReplayDenial,
        staging: RecoveryAllowance,
    ) -> Self {
        use SelectedReleaseHeadReplayDenial as Denial;
        match denial {
            Denial::BoundExceeded(past) => {
                Self::past_scratch(past.observed(), past.admitted(), staging)
            }
            Denial::SizeOverflow => Self::CountOverflow,
            Denial::NotAdmittedUpsert
            | Denial::NotAdmittedTerminalHeadRetirement
            | Denial::SourceRoot
            | Denial::SourcePath
            | Denial::Read => Self::Unverified,
        }
    }

    /// A check handed `admitted` of the walk's scratch needed `observed`:
    /// the walk held the rest, and so needed that much more in all.
    pub(in crate::orchestration::planning::page_observation) fn past_scratch(
        observed: u64,
        admitted: u64,
        staging: RecoveryAllowance,
    ) -> Self {
        staging
            .beside(observed, admitted)
            .map_or(Self::CountOverflow, Self::recovery)
    }

    /// What the walk's scratch has left once it holds `held`; past it, the
    /// walk needed `held`.
    pub(in crate::orchestration::planning::page_observation) fn left(
        staging: RecoveryAllowance,
        held: u64,
    ) -> Result<u64, Self> {
        staging
            .admit(held)
            .map(|held| staging.admitted() - held)
            .map_err(Self::recovery)
    }

    /// `left` of the walk's scratch, less `needed`; past it, the walk needed
    /// what it held beside `left`, and `needed` more.
    pub(in crate::orchestration::planning::page_observation) fn take(
        staging: RecoveryAllowance,
        left: u64,
        needed: u64,
    ) -> Result<u64, Self> {
        left.checked_sub(needed)
            .ok_or_else(|| Self::past_scratch(needed, left, staging))
    }

    /// A walk that holds `peak` at once fits its scratch, or needed `peak`.
    pub(in crate::orchestration::planning::page_observation) fn hold(
        staging: RecoveryAllowance,
        peak: u64,
    ) -> Result<(), Self> {
        Self::left(staging, peak).map(|_| ())
    }

    /// A resident ledger handed part of the walk's scratch refused what it
    /// was asked to hold.
    pub(in crate::orchestration::planning::page_observation) fn resident(
        resident: &ResidentAllowance,
        staging: RecoveryAllowance,
    ) -> Self {
        resident
            .refused_in(staging)
            .map_or(Self::CountOverflow, Self::recovery)
    }

    /// A control record the walk could not read. Its resident bytes are
    /// admitted out of the walk's scratch; its entry is charged to `budget`.
    pub(in crate::orchestration::planning::page_observation) fn unread(
        denial: PhysicalRecoverySelectedRecordReadDenial,
        budget: &ManifestEntryBudget,
        resident: &ResidentAllowance,
        staging: RecoveryAllowance,
    ) -> Self {
        use PhysicalRecoverySelectedRecordReadDenial as Denial;
        match denial {
            Denial::ManifestRead(failure) | Denial::ChunkRead { failure, .. } => failure.into(),
            Denial::ManifestEntryLimit => {
                budget.refused().map_or(Self::CountOverflow, Self::entries)
            }
            Denial::ResidentBoundExceeded => Self::resident(resident, staging),
            Denial::InvalidRoute
            | Denial::ManifestIntegrity(_)
            | Denial::ChunkIntegrity { .. }
            | Denial::Allocation { .. }
            | Denial::InvalidPayload => Self::Unverified,
        }
    }

    /// A released control physics would not admit. Its retained bytes are
    /// admitted out of the walk's scratch.
    pub(in crate::orchestration::planning::page_observation) fn control_refused(
        denial: AddressedReleasedControlDenial,
        staging: RecoveryAllowance,
    ) -> Self {
        match denial {
            AddressedReleasedControlDenial::Bound(past) => {
                Self::past_scratch(past.observed(), past.admitted(), staging)
            }
            AddressedReleasedControlDenial::CountOverflow => Self::CountOverflow,
            AddressedReleasedControlDenial::ResultRoot
            | AddressedReleasedControlDenial::Route
            | AddressedReleasedControlDenial::Frame
            | AddressedReleasedControlDenial::Allocation => Self::Unverified,
        }
    }

    const fn recovery(limit: ExceededRecoveryLimit) -> Self {
        Self::Limit(PageLimit::Recovery(limit))
    }

    const fn entries(limit: ExceededManifestEntries) -> Self {
        Self::Limit(PageLimit::Entries(limit))
    }

    /// Why the walk stopped short of a verdict, as observation reports it;
    /// `None` where it reached one: the history is unverified.
    pub(in crate::orchestration::planning::page_observation) const fn stopped(
        self,
    ) -> Option<PageObservationFailure> {
        match self {
            Self::Limit(limit) => Some(PageObservationFailure::Limit(limit)),
            Self::CountOverflow => Some(PageObservationFailure::CountOverflow),
            Self::Unverified => None,
        }
    }
}

/// Narrows what a walk step was refused to the walk's verdict, where the
/// refusal itself does not say which it is.
pub(in crate::orchestration::planning::page_observation) trait Verdict<T> {
    /// Absence or refusal here is failed verification.
    fn proven(self) -> Result<T, WalkFailure>;
}

impl<T> Verdict<T> for Option<T> {
    fn proven(self) -> Result<T, WalkFailure> {
        self.ok_or(WalkFailure::Unverified)
    }
}

impl<T, E> Verdict<T> for Result<T, E> {
    fn proven(self) -> Result<T, WalkFailure> {
        self.map_err(|_| WalkFailure::Unverified)
    }
}

impl From<PageObservationFailure> for WalkFailure {
    fn from(failure: PageObservationFailure) -> Self {
        use PageObservationFailure as Page;
        match failure {
            Page::Limit(limit) => Self::Limit(limit),
            Page::CountOverflow => Self::CountOverflow,
            Page::Media { .. }
            | Page::MissingArtifact { .. }
            | Page::InvalidManifest { .. }
            | Page::Integrity { .. }
            | Page::InvalidTarget(_)
            | Page::HistoricalDrop { .. }
            | Page::AbsentExtentBelowFrontier { .. }
            | Page::MaterializedExtentChunkCount { .. }
            | Page::MaterializedExtentCoordinate(_)
            | Page::InvalidPage(_) => Self::Unverified,
        }
    }
}

impl From<EntriesStopped> for WalkFailure {
    fn from(stopped: EntriesStopped) -> Self {
        PageObservationFailure::from(stopped).into()
    }
}

impl From<RecoveryDiscoveryFailure> for WalkFailure {
    /// Observation's own rule says which failed read is a limit.
    fn from(failure: RecoveryDiscoveryFailure) -> Self {
        PageObservationFailure::media(None, failure).into()
    }
}

#[cfg(test)]
#[path = "walk_failure_tests.rs"]
mod tests;
