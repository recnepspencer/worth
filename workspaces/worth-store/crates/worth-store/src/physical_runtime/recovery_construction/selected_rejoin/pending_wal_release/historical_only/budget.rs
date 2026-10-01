//! Conservative peak-residency reservation for the historical-only control
//! reread. These charges cover the temporary collections *before* the two
//! discovery phases allocate them; discovery's separate byte cap receives
//! only the unreserved remainder.

use worth_store_physical_format::{CurrentPhysicalRecordPlacement, PersistedRecordIdentity};

use super::super::super::{
    control_frames::SelectedArtifactSlice, tier::routes::MAX_BLOCKS,
    SelectedMediaRejoinDenial as Denial, MAX_CONTROL_FRAME_BYTES,
};

const BTREE_LINK_BYTES: u64 = 8 * std::mem::size_of::<usize>() as u64;
const ALLOCATION_MARGIN: u64 = 4;

fn node(bytes: u64) -> Result<u64, Denial> {
    bytes
        .checked_mul(ALLOCATION_MARGIN)
        .ok_or(Denial::BoundExceeded)
}

fn charged(count: u64, bytes: u64) -> Result<u64, Denial> {
    count.checked_mul(bytes).ok_or(Denial::BoundExceeded)
}

/// Peak of the route walker: every record may be in its uniqueness set and
/// selected-control map; seen nodes, pending references and fingerprints are
/// capped by the same 65,536-block limit enforced by the walker.
pub(super) fn route_walk(
    record_count: u64,
    routing_reference_bytes: u64,
    available: u64,
) -> Result<u64, Denial> {
    let record_bytes =
        node(std::mem::size_of::<PersistedRecordIdentity>() as u64 + BTREE_LINK_BYTES)?
            .checked_add(node(
                std::mem::size_of::<PersistedRecordIdentity>() as u64
                    + std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64
                    + BTREE_LINK_BYTES,
            )?)
            .ok_or(Denial::BoundExceeded)?;
    let block_bytes = node(
        2 * std::mem::size_of::<u64>() as u64
            + BTREE_LINK_BYTES
            + routing_reference_bytes
            + std::mem::size_of::<SelectedArtifactSlice>() as u64,
    )?;
    let peak = charged(record_count, record_bytes)?
        .checked_add(charged(MAX_BLOCKS as u64, block_bytes)?)
        .ok_or(Denial::BoundExceeded)?;
    (peak < available)
        .then_some(peak)
        .ok_or(Denial::BoundExceeded)
}

/// Peak after route traversal: partition duplicates at most two route maps;
/// failed-ingest verification retains fixed-size manifest/descriptor facts
/// and the selected released-ID set, and at most four concurrent 64 MiB
/// frame/decode/encode buffers. Integrity slices are charged separately from
/// their actual manifest-declared chunk count, not guessed per control. The
/// decoded drop-vector is transient and included in those frame buffers.
pub(super) fn control_decode(
    selected_controls: u64,
    admitted_released_ids: u64,
    retained_routes: u64,
    available: u64,
) -> Result<u64, Denial> {
    let route_node = node(
        std::mem::size_of::<PersistedRecordIdentity>() as u64
            + std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64
            + BTREE_LINK_BYTES,
    )?;
    // The retained facts in no_release_controls are scalar fields. 512 bytes
    // exceeds its Manifest and Descriptor shapes, including BTree links.
    const DECODED_FACT_BYTES: u64 = 512;
    let fact_node = node(
        std::mem::size_of::<PersistedRecordIdentity>() as u64
            + DECODED_FACT_BYTES
            + BTREE_LINK_BYTES,
    )?;
    let per_control = charged(2, route_node)?
        .checked_add(charged(4, fact_node)?)
        .ok_or(Denial::BoundExceeded)?;
    let admitted_id_node =
        node(std::mem::size_of::<PersistedRecordIdentity>() as u64 + BTREE_LINK_BYTES)?;
    let frame_peak = if selected_controls == 0 {
        0
    } else {
        charged(4, MAX_CONTROL_FRAME_BYTES)?
    };
    let peak = retained_routes
        .checked_add(charged(selected_controls, per_control)?)
        .and_then(|value| value.checked_add(charged(admitted_released_ids, admitted_id_node).ok()?))
        .and_then(|value| value.checked_add(frame_peak))
        .ok_or(Denial::BoundExceeded)?;
    (peak < available)
        .then_some(peak)
        .ok_or(Denial::BoundExceeded)
}

/// Both the reservation partition and the final fingerprint vector are live
/// while failed-ingest source pairs are reread. Each may contain this many
/// slices; the fourfold margin covers Vec capacity growth and allocator
/// over-allocation. The discovery byte limit is cumulative I/O, whereas its
/// largest simultaneously resident frames are included in `control_decode`.
pub(super) fn slice_limit(fixed_peak: u64, available: u64) -> Result<usize, Denial> {
    let bytes = available
        .checked_sub(fixed_peak)
        .ok_or(Denial::BoundExceeded)?;
    let per_slice = (std::mem::size_of::<SelectedArtifactSlice>() as u64)
        .checked_mul(ALLOCATION_MARGIN)
        .and_then(|value| value.checked_mul(2))
        .ok_or(Denial::BoundExceeded)?;
    usize::try_from(bytes / per_slice).map_err(|_| Denial::BoundExceeded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduced_budget_denies_before_route_or_control_allocation() {
        assert!(matches!(route_walk(8, 64, 1), Err(Denial::BoundExceeded)));
        assert!(matches!(
            control_decode(8, 12, 4096, 1),
            Err(Denial::BoundExceeded)
        ));
    }

    #[test]
    fn ordinary_bounded_roster_fits_the_shared_recovery_window() {
        let window = 1 << 30;
        let route_peak = route_walk(128, 64, window).unwrap();
        assert!(control_decode(12, 9, route_peak, window).is_ok());
        assert!(slice_limit(route_peak, window).unwrap() > 16);
    }
}
