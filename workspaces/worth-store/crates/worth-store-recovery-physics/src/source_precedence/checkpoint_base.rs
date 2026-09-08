use std::sync::Arc;

use worth_store_physical_integrity::{IntegrityValidatedRootManifest, VerifiedCheckpointStream};

use super::SelectedPhysicalRoot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalCheckpointBase {
    checkpoint: Arc<VerifiedCheckpointStream>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCheckpointBaseDenial {
    ForeignStore,
    RootGenerationMismatch,
    RootTreeMismatch,
    RootFormatMismatch,
    CompactionCutoffOutsideCheckpoint,
}

impl PhysicalCheckpointBase {
    pub fn admit(
        root: &SelectedPhysicalRoot,
        checkpoint: VerifiedCheckpointStream,
        source_root: &IntegrityValidatedRootManifest<'_>,
    ) -> Result<Self, PhysicalCheckpointBaseDenial> {
        let source = checkpoint.source();
        require_source_root(root.selected(), source, source_root)?;
        let cutoff = checkpoint.compaction_cutover().wal_cutoff_lsn_exclusive();
        if cutoff < source.wal().admitted_begin_lsn()
            || cutoff > source.wal().covered_end_lsn_exclusive()
        {
            return Err(PhysicalCheckpointBaseDenial::CompactionCutoffOutsideCheckpoint);
        }
        Ok(Self {
            checkpoint: Arc::new(checkpoint),
        })
    }

    pub fn checkpoint(&self) -> &VerifiedCheckpointStream {
        self.checkpoint.as_ref()
    }

    /// Shares the admitted checkpoint with a later recovery owner that must
    /// retain it beyond this borrow. The ordinary observation accessor stays
    /// representation-agnostic; this method makes the ownership transfer and
    /// its reference-counting cost explicit at the call site.
    pub fn share_checkpoint(&self) -> Arc<VerifiedCheckpointStream> {
        Arc::clone(&self.checkpoint)
    }

    pub fn wal_tail_begin_lsn(&self) -> u64 {
        self.checkpoint.source().wal().covered_end_lsn_exclusive()
    }
}

fn require_source_root(
    selected: &super::PhysicalRootSourceCandidate,
    source: worth_store_physical_format::PhysicalCheckpointSource,
    source_root: &IntegrityValidatedRootManifest<'_>,
) -> Result<(), PhysicalCheckpointBaseDenial> {
    if source.identity().store_identity() != selected.selector().store_identity()
        || source_root.scope().store_identity() != selected.selector().store_identity()
    {
        return Err(PhysicalCheckpointBaseDenial::ForeignStore);
    }
    if source.root().generation() != source_root.root_generation()
        || source_root.root_generation() > selected.manifest().generation()
    {
        return Err(PhysicalCheckpointBaseDenial::RootGenerationMismatch);
    }
    if source.root().tree_identity() != source_root.tree_identity()
        || source_root.tree_identity() != selected.manifest().tree_identity()
    {
        return Err(PhysicalCheckpointBaseDenial::RootTreeMismatch);
    }
    if source_root.record_format() != selected.selector().format() {
        return Err(PhysicalCheckpointBaseDenial::RootFormatMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
