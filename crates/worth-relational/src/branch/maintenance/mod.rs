//! Route canonical Branch changes to the installed SDK maintainer.
mod application;
mod change;
pub use application::apply_change;
pub use change::CommitChange;

pub use crate::storage::substrate::maintenance::{
    ChangeWork, ChangedRecord, DigestIndex, DigestIndexError, MaintenanceRefusal, MaintenanceRoute,
    PartitionChange, ReconstructionLane, ReconstructionWork, RecordChangeState,
};
use std::sync::Arc;

/// Whether derived state must be ready before the canonical publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceConsistency {
    Synchronous,
    Asynchronous {
        maximum_lag_commits: std::num::NonZeroU64,
    },
}

/// One owner maintains a persistent state from the shared canonical change.
/// Install-time dispatch selects applicable owners; an unaffected owner is not
/// invoked and its `Arc<State>` is carried unchanged by the dispatcher.
pub trait Maintained {
    type State;
    type Retirement;
    type Error: From<MaintenanceRefusal>;
    fn routes(&self) -> &std::collections::BTreeSet<MaintenanceRoute>;
    fn consistency(&self) -> MaintenanceConsistency;
    fn apply(
        &mut self,
        previous: &Arc<Self::State>,
        change: &CommitChange,
        work: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self::State>, Self::Error>;
    fn cold(
        &mut self,
        lane: &ReconstructionLane<'_>,
        work: &mut ReconstructionWork<'_, '_>,
    ) -> Result<Arc<Self::State>, Self::Error>;
    fn retire(
        &mut self,
        previous: &Arc<Self::State>,
        horizon: &Self::Retirement,
        work: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self::State>, Self::Error>;
}
