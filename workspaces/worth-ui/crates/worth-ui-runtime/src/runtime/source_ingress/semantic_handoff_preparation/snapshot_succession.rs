use super::WorthUiSemanticHandoffEvidence;
use crate::capability::{CapabilitySnapshot, CapabilitySnapshotDigest};
use std::rc::Rc;

impl WorthUiSemanticHandoffEvidence {
    pub(crate) fn predecessor_snapshot_digest(&self) -> CapabilitySnapshotDigest {
        self.predecessor_snapshot_digest
    }

    pub(crate) fn successor_snapshot_digest(&self) -> CapabilitySnapshotDigest {
        self.successor_snapshot
            .as_ref()
            .map_or(self.predecessor_snapshot_digest, |snapshot| {
                snapshot.digest()
            })
    }

    pub(crate) fn successor_snapshot(&self) -> Option<Rc<CapabilitySnapshot>> {
        self.successor_snapshot.clone()
    }
}
