use std::sync::Arc;

use super::RelationalCheckpointManifest;
use crate::durability::medium::RelationalDurableColdSource;

/// An admitted directory and the read-only source selected with it. No member
/// or historical envelope is retained by this recovery carrier.
#[derive(Debug, Clone)]
pub(crate) struct RelationalCheckpointDirectory {
    pub(crate) manifest: RelationalCheckpointManifest,
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
}

impl PartialEq for RelationalCheckpointDirectory {
    fn eq(&self, other: &Self) -> bool {
        self.manifest == other.manifest
    }
}
impl Eq for RelationalCheckpointDirectory {}
