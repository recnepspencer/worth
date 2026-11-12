//! Prepay the immutable record before any acknowledgement transfers Native custody.
use super::*;
use crate::durability::{WorldDurableFrameDenial, WorldDurableMediumDenial};

#[derive(Debug)]
pub(crate) struct ReadmittedPublicationRecordBacking {
    pub(super) memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ReadmittedPublicationRecordBacking {
    pub(crate) fn backing_bytes() -> Option<usize> {
        let counter = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>();
        [
            std::alloc::Layout::new::<ReadmittedPublicationRecord>(),
            std::alloc::Layout::new::<NativePublicationMeaning>(),
            std::alloc::Layout::new::<worth_signal::facade::durability::SignalReadmittedSourcePoint>(),
        ].into_iter().try_fold(0usize, |total, item| {
            total.checked_add(counter.extend(item).ok()?.0.pad_to_align().size())
        })
    }

    pub(crate) fn prepare(
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorldDurableMediumDenial> {
        let bytes = Self::backing_bytes()
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(WorldDurableFrameDenial::SizeLimit)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(WorldDurableMediumDenial::Request)?;
        Ok(Self { memory })
    }
}
