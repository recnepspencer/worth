//! Unprivileged persistent-name reads at an admitted branch observation.
//!
//! Currentness rests on lineage installation, not on the selected commit's
//! envelope. Each commit's lineage reaches the branch's derived name index in
//! its own settlement, before that settlement completes. Cold restore,
//! checkpoint restore and journal replay rebuild the index from the complete
//! restored event set before the runtime serves (a runtime without
//! persistent-name schemes has no names, so its index is empty and complete).
//! The one window in which an admitted observation can select a commit the
//! index has not ingested is a publication whose settlement is still in
//! flight. Any unsettled commit at or before the basis refuses the read, in
//! the runtime's pending-settlement registry, whether or not that commit
//! carried lineage and however the selected root's commit evidence was
//! admitted.

use super::RelationalObservationReadDenial;
use crate::history::data::CommitId;
use crate::lineage::access::PersistentNameObservationView;
use crate::lineage::index::PersistentNameIndexDenial;
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::RelationalRuntime;
use worth_relational_lineage::internal_port::lineage::access::name_observation_view::PersistentNameObservationViewOwnerPort as _;

/// Why a runtime refuses a persistent-name read at an observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalPersistentNameReadDenial {
    /// The runtime refused the observation or the carried request.
    Observation(RelationalObservationReadDenial),
    /// The observation selects an empty branch, which has no names.
    EmptyBranch,
    /// A commit at or before the basis is published while its settlement,
    /// which installs its lineage into the name index, is still in flight.
    /// The refusal is transient: retry once settlement completes.
    SettlementInFlight { basis: CommitId },
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
    /// Complexity: O(in-flight settlements) admission plus whatever `read`
    /// asks the view for; each view read is O(lineage path), never O(names)
    /// or O(commits).
    ///
    /// # Errors
    ///
    /// [`RelationalPersistentNameReadDenial`] when the observation is
    /// foreign, the request refuses, the branch is empty, a settlement at or
    /// before the basis is in flight, or the index is unavailable.
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
        if self
            .publication_binding()
            .has_unsettled_publication_through(basis)
        {
            return Err(RelationalPersistentNameReadDenial::SettlementInFlight { basis });
        }
        let branch_id = observation.identity().branch_id();
        self.lineage
            .read_name_index(branch_id, |index, scope| {
                read(PersistentNameObservationView::new(index, scope, basis))
            })
            .map_err(RelationalPersistentNameReadDenial::IndexUnavailable)
    }
}
