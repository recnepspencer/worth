use worth_store_physical_format::{BlobTreeNodeV1, PersistedRecordIdentity};

use super::super::super::tree::BlobTreeBuilder;
use super::super::BlobIngestFailure;
use super::BlobResumeFailure;

/// Exact selected nodes validated by the dry reconstruction. The session
/// retains their protected root until finish. At most one finish node exists
/// per tree level; full nodes are reused during reconstruction itself.
pub(in crate::physical_runtime::blob::ingest) struct RetainedBlobNodes {
    nodes: [Option<RetainedNode>; BlobTreeBuilder::maximum_levels()],
}

#[derive(Clone, Copy)]
struct RetainedNode {
    ordinal: u64,
    record: PersistedRecordIdentity,
    frame_digest: [u8; 32],
}

impl RetainedBlobNodes {
    pub(in crate::physical_runtime::blob::ingest) const fn empty() -> Self {
        Self {
            nodes: [None; BlobTreeBuilder::maximum_levels()],
        }
    }

    pub(super) fn retain(
        &mut self,
        ordinal: u64,
        record: PersistedRecordIdentity,
        frame_digest: [u8; 32],
    ) -> Result<(), BlobResumeFailure> {
        let slot = self
            .nodes
            .iter_mut()
            .find(|node| node.is_none())
            .ok_or(BlobResumeFailure::TreeConflict)?;
        *slot = Some(RetainedNode {
            ordinal,
            record,
            frame_digest,
        });
        Ok(())
    }

    pub(in crate::physical_runtime::blob::ingest) fn select(
        &mut self,
        node: &BlobTreeNodeV1,
        ordinal: u64,
    ) -> Result<Option<PersistedRecordIdentity>, BlobIngestFailure> {
        if let Some(slot) = self
            .nodes
            .iter_mut()
            .find(|entry| entry.is_some_and(|entry| entry.ordinal == ordinal))
        {
            let selected = slot.expect("matched selected node");
            if node.frame_digest() != selected.frame_digest {
                return Err(BlobIngestFailure::TreeConflict);
            }
            *slot = None;
            return Ok(Some(selected.record));
        }
        if !self.is_empty() {
            // The dry pass admitted a canonical producer prefix. Missing a
            // predecessor while a later selected node remains cannot write.
            return Err(BlobIngestFailure::TreeConflict);
        }
        Ok(None)
    }

    pub(in crate::physical_runtime::blob::ingest) fn is_empty(&self) -> bool {
        self.nodes.iter().all(Option::is_none)
    }
}
