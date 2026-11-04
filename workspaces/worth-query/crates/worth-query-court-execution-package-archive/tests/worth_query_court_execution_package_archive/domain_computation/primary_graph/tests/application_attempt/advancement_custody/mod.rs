//! Admission is observed through the real product transaction, before revalidation reads.
pub use crate::process::domain_computation::primary_graph::tests::application_attempt::*;
pub use std::num::NonZeroUsize;
pub use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Denial;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
pub use worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test as reports;
pub use worth_query_execution::facade::domain_computation::bound_advancement_requests_on_this_thread_for_test as bound;
pub use worth_query_execution::facade::domain_computation::installed_source_reads_on_this_thread_for_test as reads;
pub use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
pub use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;

pub struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}

#[test]
fn product_transaction_refuses_zero_budgets_before_revalidation() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        let control = installed_authorization_world(true);
        let scope = live_scope();
        let principal = control
            .application
            .with_host_advancement(|phase| {
                authenticated_principal(&phase.test_host_request_for_test(), &control, &scope)
            })
            .expect("installed host call opens");
        let account = control
            .application
            .with_host_advancement(|phase| {
                resolved_account(&phase.test_host_request_for_test(), &control, "open", &scope)
            })
            .expect("installed host call opens");
        let candidate = control
            .application
            .with_host_advancement(|phase| {
                admitted_program(&phase, &control, &principal, &account, &scope, "closed")
            })
            .expect("installed host call opens");
        let before = reads();
        let outcome = control
            .application
            .on_branch(candidate.product_branch())
            .transaction()
            .apply(
                worth_query_execution::facade::product::WorthQueryAdmittedChange::new(
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
            let principal = world
                .application
                .with_host_advancement(|phase| {
                    authenticated_principal(&phase.test_host_request_for_test(), &world, &request)
                })
                .expect("installed host call opens");
            let account = world
                .application
                .with_host_advancement(|phase| {
                    resolved_account(&phase.test_host_request_for_test(), &world, "open", &request)
                })
                .expect("installed host call opens");
            let program = world
                .application
                .with_host_advancement(|phase| {
                    admitted_program(&phase, &world, &principal, &account, &request, "closed")
                })
                .expect("installed host call opens");
            let before_commit = world
                .application
                .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
                .expect("installed host call opens")
                .product()
                .selected_commit()
                .clone();
            let _restore = Restore(
                place(placement),
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory { Some(0) } else { None },
                    Some(if zero_memory { 8_000_000 } else { 0 }),
                ))),
            );
            let reads_before = reads();
            reports();
            let outcome = world
                .application
                .on_branch(program.product_branch())
                .transaction()
                .apply(
                    worth_query_execution::facade::product::WorthQueryAdmittedChange::new(
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
                    Placement::World | Placement::Certified { .. } => {
                        unreachable!("this probe declares its placement")
                    }
                    Placement::Serial | Placement::Leased(_) => {
                        let Resource::MemoryLimit {
                            level,
                            requested,
                            admitted,
                        } = cause
                        else {
                            panic!("the lease reports its exact bytes");
                        };
                        assert_eq!(level, worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy);
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
                &world
                    .application
                    .product_runtime()
                    .with_product_observation_for_test(
                        world.application.product_runtime().default_branch_for_test(),
                        |observation| Ok(observation.selected_commit().clone()),
                    )
                    .expect("the owner returns the current fixed-size commit identity"),
                &before_commit
            );
        }
    }
}

#[path = "runtime_identity/mod.rs"]
pub mod runtime_identity;
