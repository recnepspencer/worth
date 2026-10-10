use super::*;
use std::collections::BTreeMap;
use worth_execution::{CancellationToken, KeylessPartition, SerialRequest};
use worth_foundational::PartitionIdentity;

#[test]
fn serial_map_memory_overflow_keeps_its_typed_admission_cause() {
    let map = ExecutionMap::<(), ()>::from_keyless_partitions(BTreeMap::from([(
        PartitionIdentity::new(1),
        KeylessPartition {
            value: (),
            kernel_scratch_bytes: u64::MAX,
            max_result_bytes: 0,
        },
    )]))
    .unwrap();
    let serial = SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(1024 * 1024),
        CancellationToken::new(),
        None,
    );
    let failure = PreparedRequestMap::<_, _, (), SignalError>::prepare(
        map,
        ExecutionRequest::serial(&serial),
        ExecutionPosture::Serial,
    )
    .err()
    .unwrap();
    assert_eq!(
        failure,
        SignalError::ExecutionAdmissionDenied(
            crate::data::error::SignalLeaseDenial::ChargedBytesOverflow
        )
    );
}
