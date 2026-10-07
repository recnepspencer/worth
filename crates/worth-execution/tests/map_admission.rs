use std::sync::atomic::{AtomicUsize, Ordering};

use worth_execution::{ExecutionMap, MapDenial, MapKernelFailure, MapOutcome, MapPartition};
use worth_foundational::PartitionIdentity;

fn partition(identity: u64, value: u64, reads: &[u64], writes: &[u64]) -> MapPartition<u64, u64> {
    MapPartition {
        identity: PartitionIdentity::new(identity),
        value,
        read_keys: reads.to_vec(),
        write_keys: writes.to_vec(),
        kernel_scratch_bytes: 0,
        max_result_bytes: 8,
    }
}

#[test]
fn exact_coverage_and_access_proof_bind_the_dispatched_values() {
    let map = ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(2), PartitionIdentity::new(7)],
        vec![partition(2, 11, &[3], &[5]), partition(7, 13, &[5], &[8])],
    );
    assert_eq!(
        map.err(),
        Some(MapDenial::ReadWriteConflict {
            reader: PartitionIdentity::new(7),
            writer: PartitionIdentity::new(2),
            read_key: 0,
        })
    );

    let map = ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(2), PartitionIdentity::new(7)],
        vec![partition(2, 11, &[5], &[5]), partition(7, 13, &[6], &[8])],
    )
    .unwrap_or_else(|reason| panic!("valid declaration denied: {reason:?}"));
    let calls = AtomicUsize::new(0);
    let outcome = map.run(None, |value, context| {
        calls.fetch_add(1, Ordering::Relaxed);
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    match outcome {
        MapOutcome::Complete { values, report } => {
            assert_eq!(values, vec![11, 13]);
            assert_eq!(report.charged_work(), 2);
        }
        MapOutcome::Stopped { reason, .. } => panic!("admitted map stopped: {reason:?}"),
    }
    assert_eq!(calls.load(Ordering::Relaxed), 2);
}

#[test]
fn missing_duplicate_or_reordered_coverage_is_denied() {
    let cases = [
        (
            vec![PartitionIdentity::new(1), PartitionIdentity::new(2)],
            vec![partition(1, 10, &[], &[])],
            MapDenial::CoverageMismatch,
        ),
        (
            vec![PartitionIdentity::new(1), PartitionIdentity::new(1)],
            vec![partition(1, 10, &[], &[]), partition(1, 20, &[], &[])],
            MapDenial::ExpectedIdentitiesNotCanonical,
        ),
        (
            vec![PartitionIdentity::new(1), PartitionIdentity::new(2)],
            vec![partition(2, 20, &[], &[]), partition(1, 10, &[], &[])],
            MapDenial::CoverageMismatch,
        ),
    ];
    for (expected, partitions, reason) in cases {
        assert_eq!(
            ExecutionMap::try_from_declared_partitions(expected, partitions).err(),
            Some(reason)
        );
    }
}

#[test]
fn overlapping_writes_and_noncanonical_access_are_denied_before_dispatch() {
    assert_eq!(
        ExecutionMap::try_from_declared_partitions(
            vec![PartitionIdentity::new(1), PartitionIdentity::new(2)],
            vec![partition(1, 10, &[], &[4]), partition(2, 20, &[], &[4])],
        )
        .err(),
        Some(MapDenial::WriteSetOverlap {
            earlier: PartitionIdentity::new(1),
            later: PartitionIdentity::new(2),
            later_key: 0,
        })
    );
    assert_eq!(
        ExecutionMap::try_from_declared_partitions(
            vec![PartitionIdentity::new(1)],
            vec![partition(1, 10, &[3, 3], &[])],
        )
        .err(),
        Some(MapDenial::ReadKeysNotCanonical {
            partition: 0,
            earlier: 0,
            later: 1,
        })
    );
    assert_eq!(
        ExecutionMap::try_from_declared_partitions(
            vec![PartitionIdentity::new(1)],
            vec![partition(1, 10, &[], &[9, 8])],
        )
        .err(),
        Some(MapDenial::WriteKeysNotCanonical {
            partition: PartitionIdentity::new(1),
            earlier: 0,
            later: 1,
        })
    );
}

#[test]
fn declared_scratch_overflow_is_denied_during_admission() {
    let mut entry = partition(1, 10, &[], &[]);
    entry.kernel_scratch_bytes = u64::MAX;
    let mut second = partition(2, 20, &[], &[]);
    second.kernel_scratch_bytes = 1;
    assert_eq!(
        ExecutionMap::try_from_declared_partitions(
            vec![PartitionIdentity::new(1), PartitionIdentity::new(2)],
            vec![entry, second],
        )
        .err(),
        Some(MapDenial::MemoryOverflow)
    );
}
