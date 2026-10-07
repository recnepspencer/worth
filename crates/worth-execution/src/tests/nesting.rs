use super::*;

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
