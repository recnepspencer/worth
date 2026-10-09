//! The actual specialized public call owns preparation and commit custody.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    },
    application_entry::WorthQueryApplicationRequestMutationDenial as Mutation,
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
fn assert_refused(denial: BankEstateProgressionDenial, placement: Placement, memory: bool) {
    let BankEstateProgressionDenial::ApplicationEntry(Mutation::ExecutionRequest(cause)) = denial
    else {
        panic!("the public entry names its opening refusal")
    };
    match (placement, memory) {
        (_, false) => assert_eq!(cause, Denial::Resource(Resource::WorkExhausted)),
        (Placement::Serial, true) => {
            assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit))
        }
        (Placement::Leased(_), true) => assert!(matches!(
            cause,
            Denial::Resource(Resource::MemoryLimit {
                level: Level::Policy,
                admitted: 0,
                ..
            })
        )),
        (Placement::World | Placement::Certified { .. }, true) => {
            panic!("the custody matrix selects serial and leased placement explicitly")
        }
    }
}

#[test]
fn revocation_refuses_before_its_admitted_principal_reader() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        for refusal in [None, Some(false), Some(true)] {
            bound(None);
            let fixture = revocation_world();
            let principal = fixture.authenticate();
            if let Some(memory) = refusal {
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if memory { 0 } else { 64 * 1024 * 1024 },
                    if memory { 8_000_000 } else { 0 },
                )));
            }
            reports();
            let before = reads();
            let outcome = fixture.runtime.revoke_estate_capability_with_key(
                &principal,
                revocation_action(),
                &idempotency(131),
                &request_scope(),
            );
            assert_eq!(
                reports().len(),
                1,
                "one executing public call owns one request"
            );
            match refusal {
                Some(memory) => {
                    assert_refused(outcome.unwrap_err(), placement, memory);
                    assert_eq!(reads(), before);
                }
                None => {
                    assert!(matches!(
                        outcome.unwrap(),
                        BankMutationCommitOutcome::Committed(_)
                    ));
                    assert!(
                        reads() > before,
                        "the same admitted entry reaches its principal reader"
                    );
                }
            }
        }
    }
}
