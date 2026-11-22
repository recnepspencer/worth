//! A cold release is accepted once, for a pending row its fence covers, with
//! the terminal its family's R7 route defines.
use super::super::{QueryOwnedAsyncRetention, QueryResourceObligation};
use super::*;

impl QueryObligationLedger {
    /// Pending rows of the cold-releasable families, in subject order.
    pub(in crate::query_obligations) fn cold_release_candidates(
        &self,
    ) -> Vec<QueryColdReleaseSubject> {
        let runs = self
            .runs
            .values()
            .filter(|row| !row.released())
            .map(|row| QueryColdReleaseSubject::ManagedRun(row.identity));
        let cleanup = self
            .cleanup
            .rows()
            .filter(|row| !row.released() && !row.evidence_inconsistent())
            .map(|row| QueryColdReleaseSubject::ManagedCleanup(row.identity));
        let resources = self
            .resources
            .values()
            .filter(|row| !row.released())
            .map(|row| QueryColdReleaseSubject::Resource(row.identity));
        runs.chain(cleanup).chain(resources).collect()
    }

    /// The terminal the subject's R7 route records after process death.
    pub(in crate::query_obligations) fn cold_release_terminal(
        &self,
        subject: QueryColdReleaseSubject,
    ) -> Result<QueryColdReleaseTerminal, Denial> {
        Ok(match subject {
            QueryColdReleaseSubject::ManagedRun(identity) => self
                .runs
                .get(&identity)
                .ok_or(Denial::UnknownObligation)?
                .cold_release_terminal(),
            QueryColdReleaseSubject::ManagedCleanup(identity) => {
                self.cleanup
                    .row(identity)
                    .ok_or(Denial::UnknownObligation)?;
                QueryColdReleaseTerminal::ReleaseOutstanding
            }
            QueryColdReleaseSubject::Resource(identity) => {
                self.cold_resource(identity)?.cold_release_terminal()
            }
        })
    }

    pub(in crate::query_obligations) fn validate_cold_release(
        &self,
        release: &QueryColdRelease,
        position: u64,
    ) -> Result<(), Denial> {
        let subject = release.subject;
        if subject.home() != self.home() {
            return Err(Denial::ForeignHome);
        }
        // Only rows the ended incarnation retained, released after its fence.
        if subject.acquire_position() > release.fence || release.fence >= position {
            return Err(Denial::IncarnationNotEnded);
        }
        let (released, cold, terminal) = match subject {
            QueryColdReleaseSubject::ManagedRun(identity) => {
                let row = self.runs.get(&identity).ok_or(Denial::UnknownObligation)?;
                (
                    row.posture.resources_released(),
                    row.cold_release,
                    row.cold_release_terminal(),
                )
            }
            QueryColdReleaseSubject::ManagedCleanup(identity) => {
                let row = self
                    .cleanup
                    .row(identity)
                    .ok_or(Denial::UnknownObligation)?;
                if row.evidence_inconsistent() {
                    // Fresh authority never repairs inconsistent evidence.
                    return Err(Denial::CleanupInconsistent { identity });
                }
                (
                    row.release.is_some(),
                    row.cold_release,
                    QueryColdReleaseTerminal::ReleaseOutstanding,
                )
            }
            QueryColdReleaseSubject::Resource(identity) => {
                let row = self.cold_resource(identity)?;
                (
                    row.handoff.release.is_some(),
                    row.cold_release,
                    row.cold_release_terminal(),
                )
            }
        };
        if cold.is_some() {
            return Err(Denial::DuplicateObligation);
        }
        if released || release.terminal != terminal {
            return Err(Denial::ObligationSubjectMismatch);
        }
        Ok(())
    }

    pub(in crate::query_obligations) fn apply_cold_release(&mut self, release: QueryColdRelease) {
        let terminal = Some(release.terminal);
        match release.subject {
            QueryColdReleaseSubject::ManagedRun(identity) => {
                self.runs
                    .get_mut(&identity)
                    .expect("validated managed work")
                    .cold_release = terminal;
            }
            QueryColdReleaseSubject::ManagedCleanup(identity) => {
                self.cleanup.apply_cold_release(identity, release.terminal);
            }
            QueryColdReleaseSubject::Resource(identity) => {
                self.resources
                    .get_mut(&identity)
                    .expect("validated resource")
                    .cold_release = terminal;
            }
        }
    }

    /// A resource row whose request is opted-in process-local. A mandatory
    /// owned-async request presented at this door is owner-held and refused.
    fn cold_resource(
        &self,
        identity: OwnedAsyncObligationIdentity,
    ) -> Result<&QueryResourceObligation, Denial> {
        let Some(row) = self.resources.get(&identity) else {
            return Err(match self.owned_async.get(&identity) {
                Some(request)
                    if request.declaration.retention == QueryOwnedAsyncRetention::Mandatory =>
                {
                    Denial::OwnedAsyncMandatory { identity }
                }
                _ => Denial::UnknownObligation,
            });
        };
        if row.original.declaration.retention != QueryOwnedAsyncRetention::ProcessLocal {
            return Err(Denial::OwnedAsyncMandatory { identity });
        }
        Ok(row)
    }
}
