use super::*;
use crate::backend::{run_scope, KernelStop};
use crate::tests::{authority, request, TEST_LOCK};

#[test]
fn checked_edit_refuses_on_the_context_it_was_asked_through() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let outer = run_scope(None, 0, 0, |parent| {
        let inner = run_scope(None, 0, 0, |_| {
            // The caller still holds its parent context while a nested scope
            // is active. That context cannot admit a checked edit here.
            assert_eq!(
                checked_edit(&lease, parent, |_| panic!("refused before editing")),
                Err(PartitionUpdateDenial::Admission(
                    LeaseDenial::UnrelatedNestedLease
                ))
            );
            Ok::<_, KernelFailure<()>>(())
        });
        assert!(inner.result.is_ok());
        assert_eq!(parent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(matches!(
        outer.result,
        Err(ScopeStop::Failure(KernelFailure::Stop(
            KernelStop::NestedStopped
        )))
    ));
}
