//! Controlled member emission over actual immutable source DATA.
mod dictionary;
mod indexes;
mod lineage;
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
pub(super) use dictionary::encode_dictionary;
pub(super) use indexes::encode_indexes;
pub(super) use lineage::{
    capture_lineage, encode_lineage, encode_lineage_events, OriginalCheckpointLineage,
    OriginalLineageEvent,
};
fn overflow() -> Denial {
    Denial::Request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Denial {
    crate::durability::checkpoint_history::capture::array_capture_stop(cause)
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    crate::durability::checkpoint_history::capture::capture_stop(
        crate::durability::checkpoint_history::capture::map_stop(cause),
    )
}

mod members;
pub(super) use members::{capture_metadata_members, CheckpointMetadataMembers};

#[cfg(test)]
mod tests;
