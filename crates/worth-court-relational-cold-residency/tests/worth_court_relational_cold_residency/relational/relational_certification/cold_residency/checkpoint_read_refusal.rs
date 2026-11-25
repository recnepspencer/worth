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
