//! Pending-release selected-media admission bounds and charged observation.

use worth_store_physical_format::BLOB_CONTROL_FRAME_MAX_BYTES;
use worth_store_recovery_physics::WitnessedSelectedControlFrame;

use super::ResolvedPlanningBasis;

pub(super) fn record_selected_reads(
    basis: &mut ResolvedPlanningBasis,
    addressed_artifacts_read: u64,
    bytes_read: u64,
    scratch: u64,
    reservation_count: usize,
) {
    basis.observed_pages.historical_publication_reads = basis
        .observed_pages
        .historical_publication_reads
        .saturating_add(addressed_artifacts_read);
    basis.observed_pages.historical_publication_bytes_read = basis
        .observed_pages
        .historical_publication_bytes_read
        .saturating_add(bytes_read);
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(
            scratch.saturating_add((reservation_count as u64).saturating_mul(
                BLOB_CONTROL_FRAME_MAX_BYTES as u64
                    + std::mem::size_of::<WitnessedSelectedControlFrame>() as u64,
            )),
        );
}
