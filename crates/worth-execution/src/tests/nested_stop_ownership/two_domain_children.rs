// The mixed success/failure case was added to cover successful siblings;
// this original case separately protects two independent failing children.
use super::*;

#[test]
fn two_failing_children_are_taken_by_their_own_outer_members() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0, 1, 2, 4] {
        let parent = authority()
            .request_lease(request(width.max(1), 2_000, 100))
            .unwrap();
        let child = parent.child(request(1, 1_000, 100)).unwrap();
        let outer = run_checked_batch(
            Some(&parent),
            &batch(&[1, 2], 0),
            placement(width),
            &|value, context| {
                context.checkpoint(1)?;
                let nested = run_checked_batch(
                    Some(&child),
                    &batch(&[*value], 0),
                    placement(width),
                    &|value, child| {
                        child.checkpoint(2)?;
                        Err::<u64, _>(KernelFailure::Domain(*value))
                    },
                );
                assert_eq!(
                    context.take_nested_computation_stop(),
                    Some(NestedComputationStop::DomainFailure)
                );
                assert_eq!(context.take_nested_computation_stop(), None);
                let Some(BatchStop::Failure {
                    cause: KernelFailure::Domain(cause),
                    ..
                }) = nested.stop
                else {
                    panic!("the real child returns its declared domain failure");
                };
                Ok::<_, KernelFailure<()>>(cause)
            },
        );
        assert_eq!(outer.stop, None);
        assert_eq!(outer.values, vec![1, 2]);
        assert_eq!(outer.report.charged_work(), 2 * (1 + 2));
        assert_eq!(outer.report.charged_span(), 1 + 2);
    }
}
