//! Admission is observed through the real product transaction, before revalidation reads.
use super::*;
use crate::domain_computation::primary_graph::{
    advancement_requests_on_this_thread_for_test as reports,
    bound_advancement_requests_on_this_thread_for_test as bound,
    installed_source_reads_on_this_thread_for_test as reads,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryAdvancementDenial as Denial, WorthQueryExecutionPlacementForTest as Placement,
    WorthQueryManagedComputationResourceDenial as Resource,
};
use std::num::NonZeroUsize;

struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}

#[test]
fn product_transaction_refuses_zero_budgets_before_revalidation() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        let control = installed_authorization_world(true);
        let scope = live_scope();
        let principal = authenticated_principal(&control, &scope);
        let account = resolved_account(&control, "open", &scope);
        let candidate = admitted_program(&control, &principal, &account, &scope, "closed");
        let before = reads();
        let outcome = control
            .application
            .on_branch(candidate.product_branch())
            .transaction()
            .apply(
                crate::domain_computation::primary_graph::WorthQueryAdmittedChange::new(
                    candidate,
                    idempotency(142, 142),
                ),
            )
            .commit()
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        assert!(
            reads() > before,
            "admitted product transaction must contact its reader"
        );
        for zero_memory in [false, true] {
            let world = installed_authorization_world(true);
            let request = live_scope();
            let principal = authenticated_principal(&world, &request);
            let account = resolved_account(&world, "open", &request);
            let program = admitted_program(&world, &principal, &account, &request, "closed");
            let before_commit = world.selected_product().product().selected_commit().clone();
            let _restore = Restore(
                place(placement),
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory { 0 } else { 64 * 1_024 * 1_024 },
                    if zero_memory { 8_000_000 } else { 0 },
                ))),
            );
            let reads_before = reads();
            reports();
            let outcome = world
                .application
                .on_branch(program.product_branch())
                .transaction()
                .apply(
                    crate::domain_computation::primary_graph::WorthQueryAdmittedChange::new(
                        program,
                        idempotency(141, 141),
                    ),
                )
                .commit()
                .unwrap();
            let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
                panic!("a refused request cannot publish: {outcome:?}");
            };
            let WorthQueryApplicationCommitDenialKind::ExecutionResource {
                denial: cause,
                partition_identity,
                policy_ancestor,
            } = denial.kind()
            else {
                panic!("product admission must retain the request cause: {denial:?}");
            };
            assert_eq!(partition_identity, None);
            assert_eq!(policy_ancestor, None);
            if !zero_memory {
                assert_eq!(cause, Resource::WorkExhausted);
            } else {
                match placement {
                    Placement::Serial => assert_eq!(cause, Resource::PolicyMemoryLimit),
                    Placement::World | Placement::Certified { .. } => {
                        unreachable!("this probe declares its placement")
                    }
                    Placement::Leased(_) => {
                        let Resource::MemoryLimit {
                            level,
                            requested,
                            admitted,
                        } = cause
                        else {
                            panic!("the lease reports its exact bytes");
                        };
                        assert_eq!(level, crate::domain_computation::primary_graph::WorthQueryMemoryLimitLevel::Policy);
                        assert!(requested > 0);
                        assert_eq!(admitted, 0);
                    }
                }
            }
            assert_eq!(
                reads(),
                reads_before,
                "the real revalidation reader was never entered"
            );
            assert_eq!(reports(), vec![Err(Denial::Resource(cause))]);
            assert_eq!(
                world.selected_product().product().selected_commit(),
                &before_commit
            );
        }
    }
}

#[path = "advancement_custody/managed_recovery.rs"]
mod managed_recovery;
#[path = "advancement_custody/runtime_identity.rs"]
mod runtime_identity;
