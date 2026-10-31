use std::sync::Arc;

use super::RelationalCheckpointManifest;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberCustody};

/// An admitted directory intrinsically retains its complete semantic basis.
#[derive(Debug, Clone)]
pub(crate) struct RelationalCheckpointDirectory {
    pub(crate) manifest: RelationalCheckpointManifest,
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
    pub(crate) history_members: Arc<super::RelationalHistoryMemberDirectory>,
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
    ) -> Result<
        RelationalCheckpointDirectory,
        crate::storage::data::RelationalStorageUnavailableCause,
    > {
        let members = self.manifest.semantic_members(&self.history_members);
        let basis = self.source.retain_members(&members)?;
        Ok(RelationalCheckpointDirectory {
            manifest: self.manifest,
            source: self.source,
            history_members: self.history_members,
            _basis: basis,
        })
    }
}

impl PartialEq for RelationalCheckpointDirectory {
    fn eq(&self, other: &Self) -> bool {
        self.manifest == other.manifest
    }
}
impl Eq for RelationalCheckpointDirectory {}
