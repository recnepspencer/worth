use std::cell::Cell;

use worth_execution::{ChargedBytes, ExecutionMap, MapKernelFailure, MapOutcome, MapPartition};
use worth_foundational::PartitionIdentity;

fn main() {
    owned_send_only_map();
    let map = ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(1)],
        vec![MapPartition {
            identity: PartitionIdentity::new(1),
            value: 7_u64,
            read_keys: vec![2_u64],
            write_keys: vec![3_u64],
            kernel_scratch_bytes: 8,
            max_result_bytes: 8,
        }],
    )
    .unwrap_or_else(|_| panic!("valid canonical partition"));
    let outcome = map.run(None, |value, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    match outcome {
        MapOutcome::Complete { values, report } => {
            assert_eq!(values, vec![7]);
            assert_eq!(report.charged_work(), 1);
        }
        MapOutcome::Stopped { reason, .. } => panic!("unexpected map stop: {reason:?}"),
    }
}

struct LocalValue(Cell<u64>);

impl ChargedBytes for LocalValue {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

fn owned_send_only_map() {
    let map = ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(1)],
        vec![MapPartition {
            identity: PartitionIdentity::new(1),
            value: LocalValue(Cell::new(7)),
            read_keys: Vec::<u64>::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        }],
    )
    .unwrap();
    let outcome = map.run_owned(None, |value, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<LocalValue>>(LocalValue(Cell::new(value.0.get())))
    });
    match outcome {
        MapOutcome::Complete { values, report } => {
            assert_eq!(values[0].0.get(), 7);
            assert_eq!(report.charged_work(), 1);
        }
        MapOutcome::Stopped { .. } => panic!("Send-only public map stopped"),
    }
}
