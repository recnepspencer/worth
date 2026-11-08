//! Admit one funded region while retaining the original parent byte allocation.
use super::*;
use crate::durability::SignalDurableFrameDenial as F;

pub(super) fn readmit(
    execution: &ExecutionRequest<'_, '_>,
    bytes: ExecutionImmutableBytes,
    region: Range<usize>,
) -> Result<SignalAuthoredHomeImage, D> {
    let encoded = bytes
        .bytes()
        .get(region.clone())
        .ok_or(D::Frame(F::Malformed))?;
    frame::admit_work_bounded(
        execution,
        encoded.len().saturating_mul(2),
        SIGNAL_HOME_IMAGE_MAXIMUM_BYTES,
    )
    .map_err(D::Frame)?;
    let mut input =
        frame::payload_bounded(encoded, KIND, SIGNAL_HOME_IMAGE_MAXIMUM_BYTES).map_err(D::Frame)?;
    let bounds = SignalRecordedIdentityContribution::read_image_bounds(&mut input, execution)?;
    let mut charge = crate::data::retained_storage::arc_allocation_charge::<ImageSource>()
        .map_err(|_| D::MetadataCapacity)?
        .bytes();
    let mut memory = execution
        .reserve_retained_memory(charge)
        .map_err(D::Request)?;
    let mut read = |kind| -> Result<Vec<Range<usize>>, D> {
        let count: usize = input
            .number()
            .map_err(D::Frame)?
            .try_into()
            .map_err(|_| D::MetadataCapacity)?;
        if count > input.bytes.len() / (frame::HEADER + 8) {
            return Err(D::Frame(F::Malformed));
        }
        charge = charge
            .checked_add(
                (count as u64)
                    .checked_mul(std::mem::size_of::<Range<usize>>() as u64)
                    .ok_or(D::MetadataCapacity)?,
            )
            .ok_or(D::MetadataCapacity)?;
        if let Some(memory) = &mut memory {
            memory.resize(charge).map_err(D::Memory)?;
        }
        let mut offsets = Vec::with_capacity(count);
        for _ in 0..count {
            let start = input.position.checked_add(8).ok_or(D::MetadataCapacity)?;
            let record = input.blob().map_err(D::Frame)?;
            let mut original = frame::payload(record, kind).map_err(D::Frame)?;
            if original.owner().map_err(D::Frame)? != bounds.owner() {
                return Err(D::ForeignOwner);
            }
            let end = input.position;
            offsets.push(
                region.start.checked_add(start).ok_or(D::MetadataCapacity)?
                    ..region.start.checked_add(end).ok_or(D::MetadataCapacity)?,
            );
        }
        Ok(offsets)
    };
    let definitions = read(1)?;
    let advances = read(2)?;
    drop(read);
    input.finish().map_err(D::Frame)?;
    Ok(SignalAuthoredHomeImage {
        source: Arc::new(ImageSource {
            bytes,
            region,
            bounds,
            definitions,
            advances,
            _memory: memory,
        }),
    })
}
