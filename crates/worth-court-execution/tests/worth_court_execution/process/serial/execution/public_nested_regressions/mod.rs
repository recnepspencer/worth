#[allow(unused_imports)] use crate::public_nested_regressions::isolated_context::*;
use {std::num::NonZeroUsize, std::sync::atomic::AtomicU64, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Mutex, std::sync::OnceLock};

use {worth_execution::CancellationToken, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap, worth_execution::LeaseDenial, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapKernelStop, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::MapStop};
use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

#[test]
fn ignored_checkpoint_rejection_still_stops_public_map() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let lease = authority().request_lease(request(1)).unwrap();
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let outcome = map(7).run(Some(&lease), |value, context| {
        calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(context.checkpoint(2), Err(worth_execution::MapKernelStop::WorkCeiling));
        Ok::<_, MapKernelFailure<()>>(*value)
    });

    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(matches!(&outcome, worth_execution::MapOutcome::Stopped {
        completed_prefix, boundary: Some(identity),
        reason: worth_execution::MapStop::WorkExhausted { identity: stopped }, ..
    } if completed_prefix.is_empty() && *identity == worth_foundational::PartitionIdentity::new(1) && stopped == identity));
    assert_eq!(outcome.report().charged_work(), 0);
}

#[test]
fn lease_free_nested_run_is_denied_and_stops_its_parent() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let parent = authority().request_lease(request(2)).unwrap();
    let outer = map(7);
    let inner = map(9);
    let inner_calls = std::sync::atomic::AtomicUsize::new(0);
    let inner_denials = std::sync::atomic::AtomicUsize::new(0);

    let outcome = outer.run(Some(&parent), |value, _| {
        let nested = inner.run(None, |_, _| {
            inner_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok::<_, MapKernelFailure<()>>(9_u64)
        });
        if matches!(
            nested,
            worth_execution::MapOutcome::Stopped {
                reason: worth_execution::MapStop::Admission(worth_execution::LeaseDenial::UnrelatedNestedLease),
                ..
            }
        ) {
            inner_denials.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        Ok::<_, MapKernelFailure<()>>(*value)
    });

    assert_eq!(inner_calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert_eq!(inner_denials.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(matches!(outcome, worth_execution::MapOutcome::Stopped {
        completed_prefix, boundary: Some(identity),
        reason: worth_execution::MapStop::Failure {
            identity: failed,
            cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::NestedStopped),
        }, ..
    } if completed_prefix.is_empty() && identity == worth_foundational::PartitionIdentity::new(1) && failed == identity));
}

#[test]
fn nested_certification_charges_one_actual_unit_to_one_unit_parent() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let parent = authority().request_lease(request(1)).unwrap();
    let child = parent.child(request(1)).unwrap();
    let outer = map(7);
    let inner = map(9);
    let child_calls = std::sync::atomic::AtomicUsize::new(0);

    let outcome = outer.run(Some(&parent), |_, _| {
        let nested = inner.certify(&child, 17, |value, context| {
            child_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            context.checkpoint(1)?;
            Ok::<_, MapKernelFailure<()>>(*value)
        });
        match nested {
            Ok(worth_execution::MapOutcome::Complete { values, report }) => {
                assert_eq!(values, vec![9]);
                assert_eq!(report.charged_work(), 1);
                Ok::<_, MapKernelFailure<()>>(values[0])
            }
            _ => panic!("nested certification must complete"),
        }
    });

    assert_eq!(child_calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert!(matches!(&outcome, worth_execution::MapOutcome::Complete { values, .. } if values == &vec![9]));
    assert_eq!(outcome.report().charged_work(), 1);
    assert_eq!(outcome.report().charged_span(), 1);
}

#[test]
fn nested_run_reports_overlapping_parent_and_child_memory() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

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
    assert!(matches!(standalone_parent, worth_execution::MapOutcome::Complete { .. }));
    assert!(matches!(standalone_child, worth_execution::MapOutcome::Complete { .. }));
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
            worth_execution::MapOutcome::Complete { values, .. } => Ok::<_, MapKernelFailure<()>>(values[0]),
            worth_execution::MapOutcome::Stopped { reason, .. } => panic!("nested run stopped: {reason:?}"),
        }
    });
    assert!(matches!(&outcome, worth_execution::MapOutcome::Complete { values, .. } if values == &vec![9]));
    assert!(
        outcome.report().physical().peak_charged_memory_bytes() > standalone_peak,
        "overlapping parent and child reservations must exceed either standalone peak"
    );
}

#[test]
fn nested_discarded_work_reaches_parent_physical_report() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let parent = authority().request_lease(request(10)).unwrap();
    let child = parent.child(request(10)).unwrap();
    let outer = map(7);
    let inner = two_partition_map();
    let child_discarded = std::sync::atomic::AtomicU64::new(0);
    let child_calls = std::sync::atomic::AtomicUsize::new(0);

    let outcome = outer.run(Some(&parent), |_, _| {
        let nested = inner.run(Some(&child), |value, context| {
            child_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            context.checkpoint(*value)?;
            if *value == 1 {
                Err(worth_execution::MapKernelFailure::Domain(()))
            } else {
                Ok(*value)
            }
        });
        if let worth_execution::MapOutcome::Stopped {
            completed_prefix,
            boundary: Some(identity),
            reason:
                worth_execution::MapStop::Failure {
                    cause: worth_execution::MapKernelFailure::Domain(()),
                    ..
                },
            report,
        } = nested
        {
            assert!(completed_prefix.is_empty());
            assert_eq!(identity, worth_foundational::PartitionIdentity::new(1));
            child_discarded.store(
                report.physical().discarded_in_flight_work(),
                std::sync::atomic::Ordering::SeqCst,
            );
        }
        Ok::<_, MapKernelFailure<()>>(7_u64)
    });

    assert_eq!(child_calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(child_discarded.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert!(matches!(
        &outcome,
        worth_execution::MapOutcome::Stopped {
            reason: worth_execution::MapStop::Failure {
                cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::NestedStopped),
                ..
            },
            ..
        }
    ));
    assert_eq!(outcome.report().physical().discarded_in_flight_work(), 2);
}

#[test]
fn serial_oracle_keeps_its_nested_descendant_serial() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let parent = authority().request_lease(request(4)).unwrap();
    let child = parent.child(request(4)).unwrap();
    let outer = map(7);
    let inner = two_partition_map();
    let outer_calls = std::sync::atomic::AtomicUsize::new(0);
    let first_child_workers = std::sync::atomic::AtomicUsize::new(0);

    let outcome = outer.certify(&parent, 23, |_, _| {
        let pass = outer_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let nested = inner.run(Some(&child), |value, context| {
            context.checkpoint(1)?;
            Ok::<_, MapKernelFailure<()>>(*value)
        });
        match nested {
            worth_execution::MapOutcome::Complete { values, report } => {
                if pass == 0 {
                    first_child_workers.store(
                        report.physical().active_workers_high_watermark(),
                        std::sync::atomic::Ordering::SeqCst,
                    );
                }
                Ok::<_, MapKernelFailure<()>>(values.into_iter().sum::<u64>())
            }
            worth_execution::MapOutcome::Stopped { reason, .. } => {
                panic!("nested oracle descendant stopped: {reason:?}")
            }
        }
    });

    assert_eq!(outer_calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(first_child_workers.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(matches!(outcome, Ok(worth_execution::MapOutcome::Complete { values, .. }) if values == vec![3]));
}
#[path = "decompose_scope/mod.rs"]
pub(crate) mod decompose_scope;
