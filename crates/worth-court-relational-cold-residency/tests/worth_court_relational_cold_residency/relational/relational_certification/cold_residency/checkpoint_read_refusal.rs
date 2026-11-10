//! Preserve the concrete storage subject of a stopped checkpoint read.
use worth_relational::facade::{
    durability::*, transactions::RelationalExecutionDenialCause as Execution,
};
pub(super) fn memory_stop(stop: RelationalRecoveryStop) -> Execution {
    match stop {
        RelationalRecoveryStop::Durability(error) => {
            Execution::StorageUnavailable(error.storage_unavailable.expect("typed image refusal"))
        }
        RelationalRecoveryStop::AdmissionImage(cause) => panic!("original image: {cause:?}"),
        RelationalRecoveryStop::Request(cause) => panic!("funded request: {cause:?}"),
        RelationalRecoveryStop::Allocation(cause) => panic!("funded allocation: {cause:?}"),
        RelationalRecoveryStop::Content(cause) => panic!("original content: {cause:?}"),
    }
}
pub(super) fn native_stop(stop: RelationalNativeCheckpointCaptureDenial) -> Execution {
    match stop {
        RelationalNativeCheckpointCaptureDenial::Durability(error) => {
            Execution::StorageUnavailable(error.storage_unavailable.expect("typed export refusal"))
        }
        RelationalNativeCheckpointCaptureDenial::AdmissionImage(cause) => {
            panic!("original image: {cause:?}")
        }
        RelationalNativeCheckpointCaptureDenial::Request(cause) => {
            panic!("funded request: {cause:?}")
        }
        RelationalNativeCheckpointCaptureDenial::Allocation(cause) => {
            panic!("funded allocation: {cause:?}")
        }
        RelationalNativeCheckpointCaptureDenial::Content(cause) => {
            panic!("original content: {cause:?}")
        }
        RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause) => {
            panic!("original frame: {cause:?}")
        }
        RelationalNativeCheckpointCaptureDenial::OwnerUnavailable => panic!("live owner"),
        RelationalNativeCheckpointCaptureDenial::OriginalAuthorityUnavailable => {
            panic!("standalone original export")
        }
        RelationalNativeCheckpointCaptureDenial::RetentionCapacityExhausted => {
            panic!("fixture has retention headroom")
        }
        RelationalNativeCheckpointCaptureDenial::RetentionIdentityExhausted => {
            panic!("fixture has identity headroom")
        }
    }
}
