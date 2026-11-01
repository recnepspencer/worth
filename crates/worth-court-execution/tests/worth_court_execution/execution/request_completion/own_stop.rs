//! Completion observes an invoking kernel's own stop without inventing a joined cause.
use super::*;

#[test]
fn completion_preserves_a_local_computation_ceiling_without_creating_an_untaken_cause() {
    for leased in [false, true] {
        for accounted in [false, true] {
            on_request(leased, worker_grant(), |request| {
                request
                    .run_native_with_checkpoint(|ancestor| {
                        let ((_, report), _) = complete(request, accounted, |active| {
                            ExecutionWorkCeiling::new(Some(1))
                                .run_with_checkpoint(active, None, |parent| {
                                    assert_eq!(
                                        parent.checkpoint(2),
                                        Err(MapKernelStop::WorkCeiling)
                                    );
                                    assert_eq!(
                                        complete::<()>(request, accounted, |_| {
                                            panic!("original local ceiling refuses effects")
                                        }),
                                        Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
                                    );
                                    assert_eq!(parent.take_nested_computation_stop(), None);
                                    assert_eq!(parent.poll(), Err(MapKernelStop::WorkCeiling));
                                })
                                .unwrap()
                        })
                        .unwrap();
                        assert_eq!(report.accounted_cost(), Some((0, 0)));
                        assert_eq!(
                            ancestor.take_nested_computation_stop(),
                            Some(worth_execution::NestedComputationStop::WorkCeiling)
                        );
                        complete(request, accounted, |_| ()).unwrap();
                    })
                    .unwrap();
            });
        }
    }
}

#[test]
fn completion_keeps_accounting_unavailable_on_its_original_kernel_and_request_settlement() {
    for leased in [false, true] {
        for accounted in [false, true] {
            on_request(leased, worker_grant(), |request| {
                let observed = Cell::new(false);
                let outer = request.run_native_with_checkpoint(|ancestor| {
                    assert_eq!(
                        request.run_native_with_checkpoint(|parent| {
                            assert!(!parent.has_accounting());
                            assert_eq!(
                                parent.checkpoint(1),
                                Err(MapKernelStop::AccountingUnavailable)
                            );
                            assert_eq!(
                                complete::<()>(request, accounted, |_| {
                                    panic!("original accounting refusal prevents effects")
                                }),
                                Err(WorkCeilingDenial::Stopped(
                                    MapKernelStop::AccountingUnavailable
                                ))
                            );
                            assert_eq!(parent.take_nested_computation_stop(), None);
                            assert_eq!(parent.poll(), Err(MapKernelStop::AccountingUnavailable));
                        }),
                        Err(WorkCeilingDenial::Stopped(
                            MapKernelStop::AccountingUnavailable
                        ))
                    );
                    assert_eq!(ancestor.take_nested_computation_stop(), None);
                    assert_eq!(ancestor.poll(), Err(MapKernelStop::NestedStopped));
                    observed.set(true);
                });
                assert!(
                    observed.get(),
                    "the original request-owned stop must settle"
                );
                assert_eq!(
                    outer,
                    Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
                );
            });
        }
    }
}
