use super::adversarial::map;
use super::*;
use crate::{MapKernelFailure, MapKernelStop, MapOutcome, MapStop};

#[test]
fn nested_map_cannot_spend_work_already_spent_by_its_parent() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 1_000, 2)).unwrap();
    let child = parent.child(request(1, 500, 2)).unwrap();
    let outer = map(&[1], 0);
    let inner = map(&[2], 0);
    let observed = Mutex::new(None);
    let outcome = outer.run(Some(&parent), |_, context| {
        context.checkpoint(1)?;
        let nested = inner.run(Some(&child), |_, context| {
            context.checkpoint(2)?;
            Ok::<_, MapKernelFailure<()>>(2_u64)
        });
        *observed.lock().unwrap() = Some(nested);
        Ok::<_, MapKernelFailure<()>>(1_u64)
    });
    assert!(
        matches!(observed.into_inner().unwrap(), Some(MapOutcome::Stopped {
            reason: MapStop::WorkExhausted { identity }, ..
        }) if identity == PartitionIdentity::new(1)),
        "a nested computation must stay within its caller's remaining lease work"
    );
    assert!(matches!(
        outcome,
        MapOutcome::Stopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::Stop(MapKernelStop::NestedStopped),
                ..
            },
            ..
        }
    ));
}

#[test]
fn inline_child_reuses_parent_slot_and_charges_parent_work() {
    let _serial = TEST_LOCK.lock().unwrap();
    let authority = authority();
    let occupied = authority.request_lease(request(3, 100, 10)).unwrap();
    let three_other_slots = occupied.try_reserve(3, 0).unwrap();
    let parent = authority.request_lease(request(1, 1_000, 10)).unwrap();
    let child = parent.child(request(1, 500, 10)).unwrap();
    let admitted = batch(&[1], 5);
    let outer = run_checked_batch(
        Some(&parent),
        &admitted,
        BackendKind::Native,
        &|value, context| {
            context.checkpoint(1)?;
            let nested = run_checked_batch(
                Some(&child),
                &admitted,
                BackendKind::Native,
                &|value, context| {
                    context.checkpoint(1)?;
                    Ok::<_, KernelFailure<()>>(*value + 1)
                },
            );
            assert_eq!(nested.values, vec![2]);
            assert_eq!(nested.stop, None);
            Ok::<_, KernelFailure<()>>(*value + nested.values[0])
        },
    );
    assert_eq!(outer.values, vec![3]);
    assert_eq!(outer.stop, None);
    assert_eq!(outer.report.charged_work(), 2);
    assert_eq!(outer.report.charged_span(), 2);
    drop(three_other_slots);
    assert!(parent.try_reserve(1, 30).is_ok());
}

#[test]
fn unrelated_nested_lease_denial_stops_parent() {
    let _serial = TEST_LOCK.lock().unwrap();
    let authority = authority();
    let parent = authority.request_lease(request(1, 1_000, 10)).unwrap();
    let unrelated = authority.request_lease(request(1, 500, 10)).unwrap();
    let admitted = batch(&[1], 5);
    let outer = run_checked_batch(Some(&parent), &admitted, BackendKind::Serial, &|_, _| {
        let refused =
            run_checked_batch(Some(&unrelated), &admitted, BackendKind::Serial, &|_, _| {
                Ok::<_, KernelFailure<()>>(0_u64)
            });
        assert_eq!(
            refused.stop,
            Some(BatchStop::Admission(LeaseDenial::UnrelatedNestedLease))
        );
        Ok::<_, KernelFailure<()>>(0_u64)
    });
    assert_eq!(outer.values, Vec::<u64>::new());
    assert_eq!(
        outer.stop,
        Some(BatchStop::Failure {
            identity: PartitionIdentity::new(1),
            cause: KernelFailure::Stop(KernelStop::NestedStopped),
        })
    );
}

#[test]
fn a_request_finding_every_process_worker_held_runs_inline() {
    let _serial = TEST_LOCK.lock().unwrap();
    let authority = authority();
    let occupied = authority.request_lease(request(4, 100, 10)).unwrap();
    let every_slot = occupied.try_reserve(4, 0).unwrap();
    let lease = authority.request_lease(request(4, 1_800, 10)).unwrap();
    let admitted = batch(&[1, 2, 3], 5);
    let run = || {
        run_checked_batch(
            Some(&lease),
            &admitted,
            BackendKind::Native,
            &|value, context| {
                context.checkpoint(1)?;
                Ok::<_, KernelFailure<()>>(*value * 2)
            },
        )
    };
    let inline = run();
    assert_eq!(inline.stop, None);
    assert_eq!(inline.values, vec![2, 4, 6]);
    assert_eq!(
        inline.report.fallback(),
        Some(worth_foundational::ExecutionFallbackCause::Capacity)
    );
    assert_eq!(inline.report.physical().active_workers_high_watermark(), 1);
    drop(every_slot);
    let free = run();
    assert_eq!(free.values, inline.values);
    assert_eq!(free.report.charged_work(), inline.report.charged_work());
}

#[test]
fn a_run_inside_a_serial_oracle_names_the_oracle() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(2, 1_900, 10)).unwrap();
    let child = parent.child(request(2, 1_000, 10)).unwrap();
    let admitted = batch(&[1, 2], 5);
    let outer = run_checked_batch(Some(&parent), &admitted, BackendKind::Serial, &|_, _| {
        let nested =
            run_checked_batch(Some(&child), &admitted, BackendKind::Native, &|value, _| {
                Ok::<_, KernelFailure<()>>(*value)
            });
        assert_eq!(nested.stop, None);
        assert_eq!(
            nested.report.fallback(),
            Some(worth_foundational::ExecutionFallbackCause::OracleSerial)
        );
        Ok::<_, KernelFailure<()>>(0_u64)
    });
    assert_eq!(outer.stop, None);
}
