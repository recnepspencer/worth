use super::*;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    },
    application_entry::WorthQueryApplicationRequestQueryDenial,
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    },
};

struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}

#[test]
fn query_refuses_zero_request_budgets_before_its_first_read() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let policy = super::support::CHECKPOINT_EXECUTION_POLICY.budget();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let before = reads();
        request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        assert!(reads() > before, "admitted query must contact its reader");
        for (memory, work) in [
            (policy.charged_memory_bytes(), 0),
            (0, policy.work_ceiling()),
        ] {
            bound(Some(worth_foundational::ExecutionBudget::new(
                NonZeroUsize::MIN,
                memory,
                work,
            )));
            let before = reads();
            reports();
            let denial = request
                .query(PlanarRead {
                    body_key: "anchor-a".into(),
                })
                .execute()
                .err()
                .expect("the zero budget refuses before reading");
            let WorthQueryApplicationRequestQueryDenial::ExecutionRequest(cause) = denial else {
                panic!("request admission must precede query admission: {denial:?}");
            };
            if work == 0 {
                assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
            } else if placement == Placement::Serial {
                assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit));
            } else {
                let Denial::Resource(Resource::MemoryLimit {
                    level,
                    requested,
                    admitted,
                }) = cause
                else {
                    panic!("the execution owner reports the refused bytes: {cause:?}");
                };
                assert_eq!(level, Level::Policy);
                assert_eq!(admitted, 0);
                assert!(
                    requested > 0,
                    "the active scope holds its owner's framework bytes"
                );
            }
            assert_eq!(
                reads(),
                before,
                "a refused root reaches no query read kernel"
            );
            assert_eq!(reports(), vec![Err(cause)]);
        }
    }
}

mod mutation_entries;
