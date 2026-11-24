//! Controlled member emission over actual immutable source DATA.
use worth_relational_contracts::internal_port as contracts;
pub mod dictionary;
pub mod indexes;
pub mod lineage;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
pub(super) use dictionary::encode_dictionary;
pub(super) use indexes::{encode_indexes, retained_selection};
pub(super) use lineage::{
    capture_lineage, encode_lineage, encode_lineage_events, OriginalCheckpointLineage,
    OriginalLineageEvent,
};
fn overflow() -> Denial {
    Denial::Request(
        contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
    )
}
fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop(cause)
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::capture_stop(
        contracts::durability::data::recovery_stop::map_stop(cause),
    )
}

pub mod members;
pub(super) use members::{capture_metadata_members, CheckpointMetadataMembers};

#[cfg(test)]
pub mod tests;
