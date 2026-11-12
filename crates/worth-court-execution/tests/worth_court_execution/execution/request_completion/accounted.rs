//! Accounted completion keeps its declared meter and the original returned owner.
use super::*;

fn accounted_grant() -> LeaseRequest {
    let mut grant = worker_grant();
    grant.policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, Some(64 * 1024), Some(9)),
    );
    grant
}

#[test]
fn declared_completion_keeps_returned_custody_and_exact_work_after_late_stops() {
    for leased in [false, true] {
        for cancel in [false, true] {
            let cancellation = CancellationSource::new();
            let mut grant = accounted_grant();
            grant.cancellation = cancellation.token();
            on_request(leased, grant, |request| {
                let owner = Box::new(String::from("original accepted result"));
                let address = std::ptr::from_ref(owner.as_ref());
                let reached = Cell::new(false);
                let (business, report) = request
                    .run_completion(ExecutionWorkCeiling::new(Some(4)), |_| {
                        request
                            .run_native_with_checkpoint(|context| context.checkpoint(3))
                            .unwrap()
                            .0
                            .unwrap();
                        if cancel {
                            cancellation.cancel();
                        }
                        assert_eq!(
                            request.run_native_with_checkpoint(|context| context.checkpoint(3)),
                            Err(WorkCeilingDenial::Stopped(if cancel {
                                MapKernelStop::Cancelled
                            } else {
                                MapKernelStop::WorkCeiling
                            }))
                        );
                        reached.set(true);
                        Err::<(), _>(owner)
                    })
                    .expect("returned business custody survives maintenance stop");
                assert!(reached.get());
                let owner = business.unwrap_err();
                assert_eq!(std::ptr::from_ref(owner.as_ref()), address);
                assert_eq!(*owner, "original accepted result");
                assert_eq!(report.accounted_cost(), Some((3, 3)));
                assert!(report.physical().peak_charged_memory_bytes() > 0);
            });
        }
    }
}

#[test]
fn root_completion_admits_the_actual_child_before_foreign_lineage_or_stop() {
    let root = ExecutionResourceLease::serial(worker_grant()).unwrap();
    let cancellation = CancellationSource::new();
    let mut grant = accounted_grant();
    grant.cancellation = cancellation.token();
    let child = root.child(grant).unwrap();
    let sibling = root.child(worker_grant()).unwrap();
    let foreign = ExecutionResourceLease::serial(worker_grant()).unwrap();
    let reached = Cell::new(false);
    let outer = ExecutionRequest::leased(&child).run(ExecutionWorkCeiling::new(None), |_| {
        let (owner, report) = ExecutionRequest::leased(&root)
            .run_completion(ExecutionWorkCeiling::new(Some(4)), |active| {
                let active = active.expect("the original child remains leased");
                assert!(active.descends_from(&child));
                assert!(active.has_accounting());
                assert_eq!(active.policy().accounting(), child.policy().accounting());
                cancellation.cancel();
                for refused in [&sibling, &foreign] {
                    let invoked = Cell::new(false);
                    assert_eq!(
                        ExecutionRequest::leased(refused)
                            .run_completion(ExecutionWorkCeiling::new(Some(4)), |_| {
                                invoked.set(true)
                            }),
                        Err(WorkCeilingDenial::Admission(
                            worth_execution::LeaseDenial::UnrelatedNestedLease
                        ))
                    );
                    assert!(!invoked.get());
                }
                reached.set(true);
                Box::new(73)
            })
            .unwrap();
        assert_eq!(*owner, 73);
        assert_eq!(report.accounted_cost(), Some((0, 0)));
    });
    assert!(
        reached.get(),
        "the admitted descendant callback actually ran"
    );
    assert_eq!(outer.unwrap().1.accounted_cost(), Some((0, 0)));
}

#[test]
fn completion_reuses_the_existing_single_scope_memory_and_work_contract() {
    for leased in [false, true] {
        let mut peaks = Vec::new();
        for completion in [false, true] {
            on_request(leased, accounted_grant(), |request| {
                let operation = |_: Option<&ExecutionResourceLease<'_>>| {
                    request
                        .run_native_with_checkpoint(|context| context.checkpoint(1))
                        .unwrap()
                        .0
                        .unwrap();
                    17
                };
                let (value, report) = if completion {
                    request.run_completion(ExecutionWorkCeiling::new(Some(4)), operation)
                } else {
                    request.run(ExecutionWorkCeiling::new(Some(4)), operation)
                }
                .unwrap();
                assert_eq!(value, 17);
                assert_eq!(report.accounted_cost(), Some((1, 1)));
                peaks.push(report.physical().peak_charged_memory_bytes());
            });
        }
        assert!(peaks[0] > 0);
        assert_eq!(peaks[0], peaks[1], "completion must not wrap another scope");
    }
}
