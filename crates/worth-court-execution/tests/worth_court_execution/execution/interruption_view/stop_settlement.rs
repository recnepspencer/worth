//! Projection handoff preserves the invoking scope's first stop and custody.
use super::*;

#[test]
fn cancellation_at_every_projection_consult_refuses_and_releases_original_backing() {
    use worth_execution::with_request_consult_observer_for_test;
    fn project(stop_at: Option<usize>) -> usize {
        let cancellation = CancellationSource::new();
        let memory = SerialMemoryBudget::new(Some(16 << 10));
        let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
        let mut observed = None;
        let mut final_memory = None;
        let outer = ExecutionRequest::serial(&request).run_native_with_checkpoint(|context| {
            let base = memory.charged_memory_bytes();
            let signal = cancellation.clone();
            let (result, count) = with_request_consult_observer_for_test(
                move |index| {
                    if stop_at == Some(index) {
                        signal.cancel();
                    }
                },
                || context.retain_interruption_view(None),
            );
            observed = Some((result, count));
            if stop_at.is_some() {
                final_memory = Some(memory.charged_memory_bytes() == base);
            }
        });
        let (result, count) = observed.expect("projection was reached");
        if stop_at.is_some() {
            assert!(matches!(
                result,
                Err(ExecutionInterruptionViewDenial::Request(
                    WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
                ))
            ));
            assert!(matches!(
                outer,
                Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
            ));
            assert_eq!(final_memory, Some(true));
        } else {
            assert!(outer.is_ok());
            assert_eq!(result.as_ref().unwrap().check_live(), Ok(()));
        }
        drop(result);
        assert_eq!(memory.charged_memory_bytes(), 0);
        count
    }
    let count = project(None);
    assert!(count > 0);
    for stop_at in 1..=count {
        project(Some(stop_at));
    }
}

#[test]
fn projection_refuses_untaken_computation_stops_and_permits_explicit_take() {
    use worth_execution::NestedComputationStop;
    for take in [false, true] {
        let lease = ExecutionResourceLease::serial(lease_request()).unwrap();
        let request = ExecutionRequest::leased(&lease);
        let mut observed = None;
        let mut taken = None;
        let mut domain = None;
        request
            .run_native_with_checkpoint(|context| {
                domain = Some(request.prepare_native(|_| Err::<(), _>("original domain")));
                if take {
                    taken = context.take_nested_computation_stop();
                }
                observed = Some(context.retain_interruption_view(Some(&lease)));
                if !take {
                    taken = context.take_nested_computation_stop();
                }
            })
            .unwrap();
        assert_eq!(domain, Some(Ok(Err("original domain"))));
        assert_eq!(taken, Some(NestedComputationStop::DomainFailure));
        let observed = observed.expect("projection was reached");
        if take {
            assert_eq!(observed.unwrap().check_live(), Ok(()));
        } else {
            assert!(matches!(
                observed,
                Err(ExecutionInterruptionViewDenial::Request(
                    WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)
                ))
            ));
        }
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
