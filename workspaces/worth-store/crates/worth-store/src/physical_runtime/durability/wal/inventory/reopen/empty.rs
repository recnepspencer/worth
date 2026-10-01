use super::*;
use worth_store_physical_backend::ArtifactTreeDirectory;
use worth_store_wal::{WalSegmentGeneration, WalSegmentId};

pub(super) fn empty_inventory(
    directory: &ArtifactTreeDirectory,
    cutoff: PhysicalWalBindingReopenCutoff,
    record_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
) -> Result<ReopenedPhysicalWalInventory, PhysicalWalOpenFailure> {
    if cutoff.lsn().is_some() {
        return Err(PhysicalWalOpenFailure::CheckpointCutoffOutsideRetainedWal);
    }
    let segment = WalSegmentId::new(1).expect("the initial WAL segment is nonzero");
    let generation = WalSegmentGeneration::new(1).expect("the initial WAL generation is nonzero");
    Ok(ReopenedPhysicalWalInventory {
        record_format,
        copy_obligations: Vec::new(),
        checkpoint_cutoff: 0,
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
        segments: PhysicalWalSegmentInventory::empty(),
        members: Vec::new(),
        retirement_spans: Vec::new(),
        retirement_records: Vec::new(),
        retained_maintenance: Vec::new(),
        retirement_locations: Vec::new(),
    })
}
