use serde::{Deserialize, Serialize};

/// Structural work performed by checkpoint readmission. Counts are diagnostic,
/// not a replacement for canonical digest or schema verification.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointRestoreWork {
    /// Present only when recovery entered through native checkpoint bytes.
    pub native_bytes_read: Option<usize>,
    pub native_envelopes_readmitted: Option<usize>,
    pub root_images_verified: usize,
    pub root_partition_images_restored: usize,
    pub mirror_partition_images_examined: usize,
    pub mirror_partitions_reused: usize,
    pub mirror_partitions_reconstructed: usize,
    pub history_envelopes_routed: usize,
    pub branch_cells_readmitted: usize,
    pub index_definitions_readmitted: usize,
    pub index_generations_readmitted: usize,
}
