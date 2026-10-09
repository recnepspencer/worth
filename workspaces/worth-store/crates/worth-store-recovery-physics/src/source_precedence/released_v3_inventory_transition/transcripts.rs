//! The V3 transition's transcripts and the scratch each reservation takes.

use worth_store_physical_format::{
    PhysicalInventoryTranscriptBuilderV1, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration,
};

use super::{ReleasedInventoryView, ReleasedV3InventoryTransitionDenial, RootHistoryAllowance};

pub(crate) fn transcript(
    view: ReleasedInventoryView<'_>,
    format: PhysicalRecordFormatDeclaration,
    limit: u64,
) -> Result<PhysicalInventoryTranscriptV1, ReleasedV3InventoryTransitionDenial> {
    use ReleasedV3InventoryTransitionDenial as Denial;
    let mut builder =
        PhysicalInventoryTranscriptBuilderV1::new(view.root, view.free, format, limit)
            .map_err(|_| Denial::InvalidSource)?;
    for route in view.routes {
        builder
            .include_route(*route)
            .map_err(|_| Denial::InvalidSource)?;
    }
    for segment in view.segments {
        builder
            .include_segment(*segment)
            .map_err(|_| Denial::InvalidSource)?;
    }
    for free in view.free_entries {
        builder
            .include_free(*free)
            .map_err(|_| Denial::InvalidSource)?;
    }
    builder.finish().map_err(|_| Denial::InvalidSource)
}

/// Room for `count` values beside `retained` bytes already held, refused past
/// `maximum` before the host is asked, and again for what the host gave.
pub(super) fn reserve<T>(
    count: usize,
    retained: u64,
    maximum: u64,
) -> Result<Vec<T>, ReleasedV3InventoryTransitionDenial> {
    use ReleasedV3InventoryTransitionDenial as Denial;
    // What `count` values and the bytes already retained need together.
    let within = |count: usize| {
        let needed = (count as u64)
            .checked_mul(std::mem::size_of::<T>() as u64)
            .and_then(|bytes| retained.checked_add(bytes))
            .ok_or(Denial::SizeOverflow)?;
        RootHistoryAllowance::scratch_bytes(maximum)
            .admit(needed)
            .map_err(Denial::BoundExceeded)
    };
    let requested_bytes = within(count)? - retained;
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|cause| Denial::Allocation {
            requested_bytes,
            cause,
        })?;
    within(values.capacity())?;
    Ok(values)
}

pub(super) fn transcript_reserved(
    view: ReleasedInventoryView<'_>,
    format: PhysicalRecordFormatDeclaration,
    limit: u64,
    maximum: u64,
) -> Result<PhysicalInventoryTranscriptV1, ReleasedV3InventoryTransitionDenial> {
    use ReleasedV3InventoryTransitionDenial as Denial;
    let free = reserve::<u8>(view.free.encoded_frame_bytes(), 0, maximum)?;
    let root = reserve::<u8>(
        view.root.encoded_frame_bytes(),
        free.capacity() as u64,
        maximum,
    )?;
    let mut builder = PhysicalInventoryTranscriptBuilderV1::new_in_reserved(
        view.root, view.free, format, limit, root, free,
    )
    .map_err(|_| Denial::InvalidSource)?;
    for route in view.routes {
        builder
            .include_route(*route)
            .map_err(|_| Denial::InvalidSource)?;
    }
    for segment in view.segments {
        builder
            .include_segment(*segment)
            .map_err(|_| Denial::InvalidSource)?;
    }
    for free in view.free_entries {
        builder
            .include_free(*free)
            .map_err(|_| Denial::InvalidSource)?;
    }
    builder.finish().map_err(|_| Denial::InvalidSource)
}
