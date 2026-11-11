//! Domain ownership is placement invariant; physical memory demand is not.
use super::*;

#[test]
fn overlapping_runs_preserve_nested_outcomes_and_release_memory() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent = authority()
        .request_lease(accounted_request(4, 100))
        .unwrap();
    let child = parent.child(accounted_request(1, 100)).unwrap();
    let rendezvous = std::sync::Barrier::new(2);
    let outer = run_checked_batch(
        Some(&parent),
        &batch(&[1, 2], 0),
        BackendKind::Native,
        &|value, context| {
            let nested = run_checked_batch(
                Some(&child),
                &batch(&[*value], 0),
                BackendKind::Native,
                &|value, nested_context| {
                    nested_context.checkpoint(2)?;
                    rendezvous.wait();
                    if *value == 1 {
                        Err::<u64, _>(KernelFailure::Domain(*value))
                    } else {
                        Ok(*value)
                    }
                },
            );
            context.take_nested_computation_stop();
            let peak = nested.report.physical().peak_charged_memory_bytes();
            match nested.stop {
                Some(BatchStop::Failure {
                    cause: KernelFailure::Domain(value),
                    ..
                }) => Ok(value),
                None => Ok(nested.values[0]),
                other => panic!("unexpected nested refusal {other:?}; peak={peak}"),
            }
            .map_err(|_: ()| KernelFailure::Domain(()))
        },
    );
    assert_eq!(outer.stop, None);
    assert_eq!(outer.values, vec![1, 2]);
    assert_eq!(parent.charged_memory_bytes(), 0);
    assert_eq!(child.charged_memory_bytes(), 0);
}
