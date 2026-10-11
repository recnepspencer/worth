use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome as Outcome,
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryApplicationCommitDenialKind as Kind,
        WorthQueryApplicationUncommitted as Uncommitted,
        WorthQueryExecutionPlacementForTest as Placement,
    },
    runtime::ExecutionAllocationPolicy as Allocation,
};

struct Restore(Placement);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
    }
}

#[test]
fn request_backing_refuses_unchecked_invariants_and_preserves_a_serial_retry() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let _restore = Restore(place(Placement::Serial));
    let observe = || {
        request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap()
    };
    let before = observe();
    let input = || {
        crate::PlanarEdit(crate::PlanarMutation {
            scope_key: "anchor-a".into(),
            operation: worth_query_consumer_values::PlanarOperation::Adjust(vec![
                worth_query_consumer_values::PlanarAdjustment {
                    body_key: "anchor-a".into(),
                    replacement_y: length(3),
                },
            ]),
        })
    };
    for workers in [1, 2, 4] {
        place(Placement::Leased(NonZeroUsize::new(workers).unwrap()));
        reports();
        let contacts = crate::handler::handler_contacts();
        let outcome = request
            .mutate(input())
            .expect_source(before.observed_sources()[0].clone())
            .idempotency(&9_766_u64)
            .execute_in_program(&application, Allocation::SystemAllocation)
            .unwrap();
        let Outcome::Commit(Uncommitted::Denied(denial)) = outcome else {
            panic!("the unchecked invariant cannot consume leased execution");
        };
        assert_eq!(
            denial.kind(),
            Kind::ExecutionUncheckedCustomKernel {
                partition_identity: None,
            }
        );
        assert_eq!(crate::handler::handler_contacts(), contacts + 1);
        assert_eq!(reports().len(), 1);
        place(Placement::Serial);
        assert_eq!(
            observe().receipt().inspect().basis().version(),
            before.receipt().inspect().basis().version()
        );
    }
    reports();
    let outcome = request
        .mutate(input())
        .expect_source(before.observed_sources()[0].clone())
        .idempotency(&9_766_u64)
        .execute_in_program(&application, Allocation::SystemAllocation)
        .unwrap();
    let Outcome::Committed { .. } = &outcome else {
        panic!("a serial request retains the ordinary invariant lane: {outcome:?}");
    };
    assert_eq!(reports().len(), 1);
    assert_ne!(
        observe().receipt().inspect().basis().version(),
        before.receipt().inspect().basis().version()
    );
}
