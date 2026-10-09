//! A staged adoption carries facts across host calls, never request custody.
use crate::document_retention_model::{
    host::publish_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    programs::RetentionProgramP1,
};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    },
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome as Outcome,
    },
    application_installation::WorthQueryProgramOwner,
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
fn adoption_preparation_and_publication_own_separate_requests() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let host = publish_on_first_program();
        let initial = host.current_world();
        let target = *host
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .owned_revision();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let _restore = Restore(place(placement), bound(None));
        let programs = host
            .runtime()
            .request(&principal, &scope)
            .on_branch(initial)
            .programs();
        let requirements = programs.compare(&target).unwrap();
        reports();
        let dropped = programs.adopt(&requirements).prepare(64).unwrap();
        assert_eq!(reports().len(), 1);
        drop(dropped);
        assert!(
            reports().is_empty(),
            "dropping staged facts performs no advancement"
        );
        assert_eq!(host.current_world(), initial);
        for zero_memory in [false, true] {
            bound(None);
            let programs = host
                .runtime()
                .request(&principal, &scope)
                .on_branch(initial)
                .programs();
            let prepared = programs.adopt(&requirements).prepare(64).unwrap();
            assert_eq!(reports().len(), 1);
            bound(Some(worth_foundational::ExecutionBudget::new(
                NonZeroUsize::MIN,
                if zero_memory { 0 } else { 64 * 1024 * 1024 },
                if zero_memory { 8_000_000 } else { 0 },
            )));
            let before = reads();
            let Outcome::ExecutionDenied(cause) = prepared.publish() else {
                panic!("publication reports its request admission refusal");
            };
            assert_eq!(reads(), before);
            if zero_memory && placement == Placement::Serial {
                assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit));
            } else if zero_memory {
                let Denial::Resource(Resource::MemoryLimit {
                    level,
                    requested,
                    admitted,
                }) = cause
                else {
                    panic!("lease bytes must be retained");
                };
                assert_eq!(level, Level::Policy);
                assert!(requested > 0);
                assert_eq!(admitted, 0);
            } else {
                assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
            }
            assert_eq!(reports().len(), 1);
            assert_eq!(host.current_world(), initial, "refusal publishes nothing");
        }
        bound(None);
        let programs = host
            .runtime()
            .request(&principal, &scope)
            .on_branch(initial)
            .programs();
        let prepared = programs.adopt(&requirements).prepare(64).unwrap();
        assert_eq!(reports().len(), 1);
        let before = reads();
        assert!(matches!(prepared.publish(), Outcome::Performed(_)));
        assert!(
            reads() > before,
            "admitted publication contacts its revalidation reader"
        );
        assert_eq!(
            reports().len(),
            1,
            "publish owns a fresh request after host preparation"
        );
    }
}

#[path = "request_lifetime/entry_custody.rs"]
mod entry_custody;
