use worth_execution::{ExecutionMap, MapKernelFailure, MapOutcome, MapPartition};
use worth_foundational::PartitionIdentity;

fn main() {
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
