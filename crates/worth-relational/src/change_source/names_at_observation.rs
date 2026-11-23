//! Unprivileged persistent-name reads at an admitted branch observation.

use super::RelationalObservationReadDenial;
use crate::history::data::CommitId;
use crate::lineage::access::PersistentNameObservationView;
use crate::lineage::index::PersistentNameIndexDenial;
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::RelationalRuntime;

/// Why a runtime refuses a persistent-name read at an observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalPersistentNameReadDenial {
    /// The runtime refused the observation or the carried request.
    Observation(RelationalObservationReadDenial),
    /// The observation selects an empty branch, which has no names.
    EmptyBranch,
    /// The observation selects a commit whose lineage the branch's derived
    /// name index has not ingested yet: the commit is published and its
    /// settlement is still in flight. Retry after settlement.
    IndexBehindObservation {
        basis: CommitId,
        index_head: Option<CommitId>,
    },
    /// The branch's derived name index refused canonical history and stays
    /// unavailable until it is rebuilt from canonical events.
    IndexUnavailable(PersistentNameIndexDenial),
}

impl RelationalRuntime {
    /// Runs `read` over the branch's persistent names exactly as
    /// `observation`'s commit sees them.
    ///
    /// The view is read-only and pinned to the observation's commit. It is
    /// held under the lineage read lock for the duration of `read` only, so
    /// `read` must not commit through this runtime.
    ///
    /// Complexity: O(1) admission plus whatever `read` asks the view for;
    /// each view read is O(lineage path), never O(names) or O(commits).
    ///
    /// # Errors
    ///
    /// [`RelationalPersistentNameReadDenial`] when the observation is
    /// foreign, the request refuses, the branch is empty, the selected
    /// commit's lineage is not yet ingested, or the index is unavailable.
    pub fn with_persistent_names_at_observation<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &RelationalBranchObservation,
        read: impl FnOnce(PersistentNameObservationView<'_>) -> T,
    ) -> Result<T, RelationalPersistentNameReadDenial> {
        self.admit_observation_read(observation)
            .map_err(|denial| RelationalPersistentNameReadDenial::Observation(denial.into()))?;
        crate::execution::check_request(execution).map_err(|cause| {
            RelationalPersistentNameReadDenial::Observation(
                RelationalObservationReadDenial::Execution(cause),
            )
        })?;
        let basis = observation
            .commit_id()
            .ok_or(RelationalPersistentNameReadDenial::EmptyBranch)?;
        let selected_has_lineage = observation
            .canonical_commit()
            .is_some_and(|envelope| !envelope.lineage_events().is_empty());
        let branch_id = observation.identity().branch_id();
        self.lineage
            .read_name_index(branch_id, |index| {
                let index_head = index.and_then(|index| index.head());
                if selected_has_lineage && index_head.is_none_or(|head| head < basis) {
                    return Err(RelationalPersistentNameReadDenial::IndexBehindObservation {
                        basis,
                        index_head,
                    });
                }
                Ok(read(PersistentNameObservationView::new(index, basis)))
            })
            .map_err(RelationalPersistentNameReadDenial::IndexUnavailable)?
    }
}
