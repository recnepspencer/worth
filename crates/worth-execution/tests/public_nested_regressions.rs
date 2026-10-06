use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
};

use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionMap, LeaseDenial,
    LeaseRequest, MapKernelFailure, MapKernelStop, MapOutcome, MapPartition, MapStop,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: 8_000,
        })
        .expect("one authority for this integration target")
    })
}

fn request(work_ceiling: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), 4_000, work_ceiling),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

fn map(value: u64) -> ExecutionMap<u64, u64> {
    let identity = PartitionIdentity::new(1);
    ExecutionMap::try_from_declared_partitions(
        vec![identity],
        vec![MapPartition {
            identity,
            value,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        }],
    )
    .expect("valid one-partition map")
}

fn two_partition_map() -> ExecutionMap<u64, u64> {
    let identities = [PartitionIdentity::new(1), PartitionIdentity::new(2)];
    ExecutionMap::try_from_declared_partitions(
        identities.to_vec(),
        identities
            .into_iter()
            .enumerate()
            .map(|(index, identity)| MapPartition {
                identity,
                value: index as u64 + 1,
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            })
            .collect(),
    )
    .expect("valid two-partition map")
}

#[test]
fn ignored_checkpoint_rejection_still_stops_public_map() {
    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let lease = authority().request_lease(request(1)).unwrap();
    let calls = AtomicUsize::new(0);
    let outcome = map(7).run(Some(&lease), |value, context| {
        calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(context.checkpoint(2), Err(MapKernelStop::WorkCeiling));
        Ok::<_, MapKernelFailure<()>>(*value)
    });

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(matches!(&outcome, MapOutcome::Stopped {
        completed_prefix, boundary: Some(identity),
        reason: MapStop::WorkExhausted { identity: stopped }, ..
    } if completed_prefix.is_empty() && *identity == PartitionIdentity::new(1) && stopped == identity));
    assert_eq!(outcome.report().charged_work(), 0);
}

#[test]
fn lease_free_nested_run_is_denied_and_stops_its_parent() {
    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let parent = authority().request_lease(request(2)).unwrap();
    let outer = map(7);
    let inner = map(9);
    let inner_calls = AtomicUsize::new(0);
    let inner_denials = AtomicUsize::new(0);

    let outcome = outer.run(Some(&parent), |value, _| {
        let nested = inner.run(None, |_, _| {
            inner_calls.fetch_add(1, Ordering::SeqCst);
            Ok::<_, MapKernelFailure<()>>(9_u64)
        });
        if matches!(
            nested,
            MapOutcome::Stopped {
                reason: MapStop::Admission(LeaseDenial::UnrelatedNestedLease),
                ..
            }
        ) {
            inner_denials.fetch_add(1, Ordering::SeqCst);
        }
        Ok::<_, MapKernelFailure<()>>(*value)
    });

    assert_eq!(inner_calls.load(Ordering::SeqCst), 0);
    assert_eq!(inner_denials.load(Ordering::SeqCst), 1);
    assert!(matches!(outcome, MapOutcome::Stopped {
        completed_prefix, boundary: Some(identity),
        reason: MapStop::Failure {
            identity: failed,
            cause: MapKernelFailure::Stop(MapKernelStop::NestedStopped),
        }, ..
    } if completed_prefix.is_empty() && identity == PartitionIdentity::new(1) && failed == identity));
}

#[test]
fn nested_certification_charges_one_actual_unit_to_one_unit_parent() {
    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let parent = authority().request_lease(request(1)).unwrap();
    let child = parent.child(request(1)).unwrap();
    let outer = map(7);
    let inner = map(9);
    let child_calls = AtomicUsize::new(0);

    let outcome = outer.run(Some(&parent), |_, _| {
        let nested = inner.certify(&child, 17, |value, context| {
            child_calls.fetch_add(1, Ordering::SeqCst);
            context.checkpoint(1)?;
            Ok::<_, MapKernelFailure<()>>(*value)
        });
        match nested {
            Ok(MapOutcome::Complete { values, report }) => {
                assert_eq!(values, vec![9]);
                assert_eq!(report.charged_work(), 1);
                Ok::<_, MapKernelFailure<()>>(values[0])
            }
            _ => panic!("nested certification must complete"),
        }
    });

    assert_eq!(child_calls.load(Ordering::SeqCst), 2);
    assert!(matches!(&outcome, MapOutcome::Complete { values, .. } if values == &vec![9]));
    assert_eq!(outcome.report().charged_work(), 1);
    assert_eq!(outcome.report().charged_span(), 1);
}

#[test]
fn nested_run_reports_overlapping_parent_and_child_memory() {
    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let parent = authority().request_lease(request(2)).unwrap();
    let child = parent.child(request(2)).unwrap();
    let outer = map(7);
    let inner = map(9);
    let standalone_parent = outer.run(Some(&parent), |value, _| {
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    let standalone_child = inner.run(Some(&child), |value, _| {
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    assert!(matches!(standalone_parent, MapOutcome::Complete { .. }));
    assert!(matches!(standalone_child, MapOutcome::Complete { .. }));
    let standalone_peak = standalone_parent
        .report()
        .physical()
        .peak_charged_memory_bytes()
        .max(
            standalone_child
                .report()
                .physical()
                .peak_charged_memory_bytes(),
        );

    let outcome = outer.run(Some(&parent), |_, _| {
        match inner.run(Some(&child), |value, _| {
            Ok::<_, MapKernelFailure<()>>(*value)
        }) {
            MapOutcome::Complete { values, .. } => Ok::<_, MapKernelFailure<()>>(values[0]),
            MapOutcome::Stopped { reason, .. } => panic!("nested run stopped: {reason:?}"),
        }
    });
    assert!(matches!(&outcome, MapOutcome::Complete { values, .. } if values == &vec![9]));
    assert!(
        outcome.report().physical().peak_charged_memory_bytes() > standalone_peak,
        "overlapping parent and child reservations must exceed either standalone peak"
    );
}

#[test]
fn nested_discarded_work_reaches_parent_physical_report() {
    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let parent = authority().request_lease(request(10)).unwrap();
    let child = parent.child(request(10)).unwrap();
    let outer = map(7);
    let inner = two_partition_map();
    let child_discarded = AtomicU64::new(0);
    let child_calls = AtomicUsize::new(0);

    let outcome = outer.run(Some(&parent), |_, _| {
        let nested = inner.run(Some(&child), |value, context| {
            child_calls.fetch_add(1, Ordering::SeqCst);
            context.checkpoint(*value)?;
            if *value == 1 {
                Err(MapKernelFailure::Domain(()))
            } else {
                Ok(*value)
            }
        });
        if let MapOutcome::Stopped {
            completed_prefix,
            boundary: Some(identity),
            reason:
                MapStop::Failure {
                    cause: MapKernelFailure::Domain(()),
                    ..
                },
            report,
        } = nested
        {
            assert!(completed_prefix.is_empty());
            assert_eq!(identity, PartitionIdentity::new(1));
            child_discarded.store(
                report.physical().discarded_in_flight_work(),
                Ordering::SeqCst,
            );
        }
        Ok::<_, MapKernelFailure<()>>(7_u64)
    });

    assert_eq!(child_calls.load(Ordering::SeqCst), 2);
    assert_eq!(child_discarded.load(Ordering::SeqCst), 2);
    assert!(matches!(
        &outcome,
        MapOutcome::Stopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::Stop(MapKernelStop::NestedStopped),
                ..
            },
            ..
        }
    ));
    assert_eq!(outcome.report().physical().discarded_in_flight_work(), 2);
}

#[test]
fn serial_oracle_keeps_its_nested_descendant_serial() {
    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let parent = authority().request_lease(request(4)).unwrap();
    let child = parent.child(request(4)).unwrap();
    let outer = map(7);
    let inner = two_partition_map();
    let outer_calls = AtomicUsize::new(0);
    let first_child_workers = AtomicUsize::new(0);

    let outcome = outer.certify(&parent, 23, |_, _| {
        let pass = outer_calls.fetch_add(1, Ordering::SeqCst);
        let nested = inner.run(Some(&child), |value, context| {
            context.checkpoint(1)?;
            Ok::<_, MapKernelFailure<()>>(*value)
        });
        match nested {
            MapOutcome::Complete { values, report } => {
                if pass == 0 {
                    first_child_workers.store(
                        report.physical().active_workers_high_watermark(),
                        Ordering::SeqCst,
                    );
                }
                Ok::<_, MapKernelFailure<()>>(values.into_iter().sum::<u64>())
            }
            MapOutcome::Stopped { reason, .. } => {
                panic!("nested oracle descendant stopped: {reason:?}")
            }
        }
    });

    assert_eq!(outer_calls.load(Ordering::SeqCst), 2);
    assert_eq!(first_child_workers.load(Ordering::SeqCst), 1);
    assert!(matches!(outcome, Ok(MapOutcome::Complete { values, .. }) if values == vec![3]));
}
