#[allow(unused_imports)]
use crate::map_admission::ordinary_context::*;
use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};

use worth_foundational::PartitionIdentity;
use {
    worth_execution::ExecutionMap, worth_execution::MapDenial, worth_execution::MapKernelFailure,
    worth_execution::MapOutcome, worth_execution::MapPartition,
};

#[test]
fn exact_coverage_and_access_proof_bind_the_dispatched_values() {
    let map = worth_execution::ExecutionMap::try_from_declared_partitions(
        vec![
            worth_foundational::PartitionIdentity::new(2),
            worth_foundational::PartitionIdentity::new(7),
        ],
        vec![partition(2, 11, &[3], &[5]), partition(7, 13, &[5], &[8])],
    );
    assert_eq!(
        map.err(),
        Some(worth_execution::MapDenial::ReadWriteConflict {
            reader: worth_foundational::PartitionIdentity::new(7),
            writer: worth_foundational::PartitionIdentity::new(2),
            read_key: 0,
        })
    );

    let map = worth_execution::ExecutionMap::try_from_declared_partitions(
        vec![
            worth_foundational::PartitionIdentity::new(2),
            worth_foundational::PartitionIdentity::new(7),
        ],
        vec![partition(2, 11, &[5], &[5]), partition(7, 13, &[6], &[8])],
    )
    .unwrap_or_else(|reason| panic!("valid declaration denied: {reason:?}"));
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let outcome = map.run(None, |value, context| {
        calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    match outcome {
        worth_execution::MapOutcome::Complete { values, report } => {
            assert_eq!(values, vec![11, 13]);
            assert_eq!(report.charged_work(), 2);
        }
        worth_execution::MapOutcome::Stopped { reason, .. } => {
            panic!("admitted map stopped: {reason:?}")
        }
    }
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 2);
}

#[test]
fn missing_duplicate_or_reordered_coverage_is_denied() {
    let cases = [
        (
            vec![
                worth_foundational::PartitionIdentity::new(1),
                worth_foundational::PartitionIdentity::new(2),
            ],
            vec![partition(1, 10, &[], &[])],
            worth_execution::MapDenial::CoverageMismatch,
        ),
        (
            vec![
                worth_foundational::PartitionIdentity::new(1),
                worth_foundational::PartitionIdentity::new(1),
            ],
            vec![partition(1, 10, &[], &[]), partition(1, 20, &[], &[])],
            worth_execution::MapDenial::ExpectedIdentitiesNotCanonical,
        ),
        (
            vec![
                worth_foundational::PartitionIdentity::new(1),
                worth_foundational::PartitionIdentity::new(2),
            ],
            vec![partition(2, 20, &[], &[]), partition(1, 10, &[], &[])],
            worth_execution::MapDenial::CoverageMismatch,
        ),
    ];
    for (expected, partitions, reason) in cases {
        assert_eq!(
            worth_execution::ExecutionMap::try_from_declared_partitions(expected, partitions).err(),
            Some(reason)
        );
    }
}

#[test]
fn overlapping_writes_and_noncanonical_access_are_denied_before_dispatch() {
    assert_eq!(
        worth_execution::ExecutionMap::try_from_declared_partitions(
            vec![
                worth_foundational::PartitionIdentity::new(1),
                worth_foundational::PartitionIdentity::new(2)
            ],
            vec![partition(1, 10, &[], &[4]), partition(2, 20, &[], &[4])],
        )
        .err(),
        Some(worth_execution::MapDenial::WriteSetOverlap {
            earlier: worth_foundational::PartitionIdentity::new(1),
            later: worth_foundational::PartitionIdentity::new(2),
            later_key: 0,
        })
    );
    assert_eq!(
        worth_execution::ExecutionMap::try_from_declared_partitions(
            vec![worth_foundational::PartitionIdentity::new(1)],
            vec![partition(1, 10, &[3, 3], &[])],
        )
        .err(),
        Some(worth_execution::MapDenial::ReadKeysNotCanonical {
            partition: 0,
            earlier: 0,
            later: 1,
        })
    );
    assert_eq!(
        worth_execution::ExecutionMap::try_from_declared_partitions(
            vec![worth_foundational::PartitionIdentity::new(1)],
            vec![partition(1, 10, &[], &[9, 8])],
        )
        .err(),
        Some(worth_execution::MapDenial::WriteKeysNotCanonical {
            partition: worth_foundational::PartitionIdentity::new(1),
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
        worth_execution::ExecutionMap::try_from_declared_partitions(
            vec![
                worth_foundational::PartitionIdentity::new(1),
                worth_foundational::PartitionIdentity::new(2)
            ],
            vec![entry, second],
        )
        .err(),
        Some(worth_execution::MapDenial::MemoryOverflow)
    );
}
