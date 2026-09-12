use super::{
    CompactionReadInterlockCounters, CompactionReadInterlockDenial, CompactionRewritePublication,
};
use crate::{PhysicalReadPlanReleaseReceipt, ReleasedOldReachability};

#[derive(Debug, Clone)]
pub struct CompactionDeferredReclaimQueue {
    publication: CompactionRewritePublication,
    counters: CompactionReadInterlockCounters,
}

#[derive(Debug, Clone)]
pub struct DrainedCompactionReclaim {
    publication: CompactionRewritePublication,
    released: ReleasedOldReachability,
    counters: CompactionReadInterlockCounters,
}

impl CompactionDeferredReclaimQueue {
    const OWNER_CASE: super::CompactionOwnerCaseDeclaration =
        super::CompactionOwnerCaseDeclaration::declared_by_owner(
            super::CompactionOwnerCaseId::DeferReclaim,
            super::CompactionCutoverState::PublicationCommitted,
            super::CompactionCutoverState::ReclaimDeferred,
        );

    pub const fn cutover_state(&self) -> super::CompactionCutoverState {
        super::CompactionCutoverState::ReclaimDeferred
    }

    pub const fn owner_case_observation(&self) -> super::CompactionOwnerCaseObservation {
        super::CompactionOwnerCaseObservation::issued_by_owner(Self::OWNER_CASE)
    }

    pub fn admit(
        publication: CompactionRewritePublication,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        if !publication.delta().plan().reclaim_deferred() {
            return Ok(Self {
                counters: publication.counters(),
                publication,
            });
        }
        Ok(Self {
            counters: publication.counters().with_blocked_reclaim(),
            publication,
        })
    }

    pub fn reject_early_reclaim(
        &self,
    ) -> (
        CompactionReadInterlockDenial,
        CompactionReadInterlockCounters,
    ) {
        (
            CompactionReadInterlockDenial::EarlyReclaimBeforeReadRelease {
                protected: self
                    .publication
                    .delta()
                    .plan()
                    .protected()
                    .footprint_basis(),
            },
            self.counters.with_denied_early_reclaim(),
        )
    }

    pub fn drain_after_release(
        self,
        release: PhysicalReadPlanReleaseReceipt,
    ) -> Result<DrainedCompactionReclaim, CompactionReadInterlockDenial> {
        let released = self
            .publication
            .publication()
            .admit_old_reachability_release(release)
            .map_err(
                |_| CompactionReadInterlockDenial::EarlyReclaimBeforeReadRelease {
                    protected: self
                        .publication
                        .delta()
                        .plan()
                        .protected()
                        .footprint_basis(),
                },
            )?;
        Ok(DrainedCompactionReclaim {
            publication: self.publication,
            released,
            counters: self.counters,
        })
    }

    pub const fn counters(&self) -> CompactionReadInterlockCounters {
        self.counters
    }

    pub(crate) const fn publication(&self) -> &CompactionRewritePublication {
        &self.publication
    }
}

impl DrainedCompactionReclaim {
    pub(super) fn matches_publication(&self, publication: &CompactionRewritePublication) -> bool {
        self.publication.delta().plan() == publication.delta().plan()
            && self.publication.publication().old_root() == publication.publication().old_root()
            && self.publication.publication().new_root() == publication.publication().new_root()
    }

    const OWNER_CASE: super::CompactionOwnerCaseDeclaration =
        super::CompactionOwnerCaseDeclaration::declared_by_owner(
            super::CompactionOwnerCaseId::DrainReclaimAfterReadRelease,
            super::CompactionCutoverState::ReclaimDeferred,
            super::CompactionCutoverState::Reclaimed,
        );

    pub const fn cutover_state(&self) -> super::CompactionCutoverState {
        super::CompactionCutoverState::Reclaimed
    }

    pub const fn owner_case_observation(&self) -> super::CompactionOwnerCaseObservation {
        super::CompactionOwnerCaseObservation::issued_by_owner(Self::OWNER_CASE)
    }

    pub const fn released(&self) -> ReleasedOldReachability {
        self.released
    }

    pub const fn counters(&self) -> CompactionReadInterlockCounters {
        self.counters
    }
}

pub(super) fn owner_cases() -> impl Iterator<Item = super::CompactionOwnerCaseDeclaration> {
    [
        CompactionDeferredReclaimQueue::OWNER_CASE,
        DrainedCompactionReclaim::OWNER_CASE,
    ]
    .into_iter()
}
