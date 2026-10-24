use super::*;
use worth_store_physical_backend::ArtifactTreeDirectory;
use worth_store_wal::WAL_ORIGIN;

pub(super) fn empty_inventory(
    directory: &ArtifactTreeDirectory,
    cutoff: PhysicalWalBindingReopenCutoff,
    record_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
) -> Result<ReopenedPhysicalWalInventory, PhysicalWalOpenFailure> {
    if cutoff.lsn().is_some() {
        return Err(PhysicalWalOpenFailure::CheckpointCutoffOutsideRetainedWal);
    }
    let segment = WAL_ORIGIN.segment();
    let generation = WAL_ORIGIN.generation();
    Ok(ReopenedPhysicalWalInventory {
        record_format,
        copy_obligations: Vec::new(),
        checkpoint_cutoff: WAL_ORIGIN.lsn().get(),
        frontier: WalAppendFrontier::empty(segment, generation),
        active_artifact: artifact(
            directory,
            WalSegmentArtifactIdentity::new(segment, generation),
        ),
        segment_count: 0,
        frame_count: 0,
        publication_groups: Vec::new(),
        release_metadata: Vec::new(),
        byte_count: 0,
        peak_buffer_bytes: 0,
        requires_inspection: false,
        release_evidence: super::super::RetainedWalReleaseEvidence::new(
            false,
            super::super::RetainedWalHistory::Empty,
        ),
        segments: PhysicalWalSegmentInventory::empty(),
        members: Vec::new(),
        retirement_spans: Vec::new(),
        retirement_records: Vec::new(),
        retained_maintenance: Vec::new(),
        retirement_locations: Vec::new(),
    })
}

#[cfg(test)]
mod tests;
