//! Fill the original pre-effect backing with actual post-effect evidence.
use worth_execution::{fill_admitted_framed_buffer, ExecutionByteBuffer};

use super::{write_record, Denial, WorldDurableMediumDenial, WorldRecord, MAGIC, MAXIMUM};

pub(in crate::durability) fn admitted_length(
    open: usize,
    intent: crate::durability::WorldOpenComponentIntent,
) -> Result<usize, Denial> {
    use crate::durability::WorldOpenComponentIntent;
    let owner_bytes = match intent {
        WorldOpenComponentIntent::RelationalOnly => 48,
        WorldOpenComponentIntent::SignalOnly => {
            8 + worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES
        }
        WorldOpenComponentIntent::RelationalAndSignal => {
            48 + 8
                + worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES
        }
    };
    open.checked_add(1 + owner_bytes + 8)
        .filter(|total| *total <= MAXIMUM)
        .ok_or(Denial::SizeLimit)
}

pub(in crate::durability) fn fill(
    backing: &mut ExecutionByteBuffer,
    record: &WorldRecord,
) -> Result<(), WorldDurableMediumDenial> {
    fill_admitted_framed_buffer(
        super::spec(MAGIC),
        super::WORLD_ARTIFACT_VERSION.get(),
        backing,
        |sink| {
            super::super::framed_denial::forward_payload(
                &mut |emit| write_record(record, emit),
                sink,
            )
        },
        WorldDurableMediumDenial::Allocation,
    )
}
