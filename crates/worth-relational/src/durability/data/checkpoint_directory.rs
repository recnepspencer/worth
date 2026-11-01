mod admission;
pub(crate) use admission::CheckpointDirectoryAdmission;
mod retirement;
pub(crate) use retirement::{CheckpointDirectoryRetirement, PreparedCheckpointDirectoryRetirement};
use std::sync::Arc;

use super::RelationalCheckpointManifest;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberCustody};

/// An admitted directory intrinsically retains its complete semantic basis.
#[derive(Debug, Clone)]
pub(crate) struct RelationalCheckpointDirectory {
    pub(crate) manifest: RelationalCheckpointManifest,
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
    pub(crate) history_members: Arc<super::RelationalHistoryMemberDirectory>,
    pub(crate) canonical_read:
        crate::durability::checkpoint_history::readmission::SelectedCheckpointRead,
    _basis: RelationalDurableMemberCustody,
}

/// Capture precedes placement: a draft cannot claim custody of new members.
#[derive(Debug, Clone)]
pub(crate) struct PreparedCheckpointDirectory {
    pub(crate) manifest: RelationalCheckpointManifest,
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
    pub(crate) history_members: Arc<super::RelationalHistoryMemberDirectory>,
}

impl PreparedCheckpointDirectory {
    pub(crate) fn readmit(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalCheckpointDirectory, super::RelationalRecoveryStop> {
        // Opening owns an ordinary pre-effect abandonment boundary. Live ACK
        // keeps this same admission state in its pending phase across refusals.
        CheckpointDirectoryAdmission::new(self)
            .advance(request)
            .map_err(admission::opening_stop)
    }
}

impl PartialEq for RelationalCheckpointDirectory {
    fn eq(&self, other: &Self) -> bool {
        self.manifest == other.manifest
    }
}
impl Eq for RelationalCheckpointDirectory {}
