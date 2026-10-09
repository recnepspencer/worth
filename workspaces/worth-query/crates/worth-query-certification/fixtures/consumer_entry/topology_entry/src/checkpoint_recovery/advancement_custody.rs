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

#[test]
fn held_mutation_commit_refuses_zero_budgets_before_revalidation() {
    use worth_query_host::facade::{
        application_entry::{
            WorthQueryApplicationMutationOutcome as Outcome,
            WorthQueryApplicationProgramMutationPreparation as Preparation,
        },
        primary_graph::{
            WorthQueryApplicationCommitDenialKind as CommitKind,
            WorthQueryApplicationUncommitted as Uncommitted,
        },
    };
    let _guard = checkpoint_recovery_test_guard();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        for zero_memory in [false, true] {
            let application = install(None);
            let (scope, principal) = authenticate(&application);
            let request = application.request(&principal, &scope);
            drop(settle(&request, &application));
            let observed = request
                .query(PlanarRead {
                    body_key: "anchor-a".into(),
                })
                .execute()
                .unwrap();
            let before = observed.receipt().inspect().basis().version();
            let mut mutation = request
                .mutate(crate::PlanarEdit(crate::PlanarMutation {
                    scope_key: "anchor-a".into(),
                    operation: worth_query_consumer_values::PlanarOperation::VerifyCurrentOutputs(
                        vec![
                            worth_query_consumer_values::PlanarCurrentOutputExpectation {
                                producer_key: "anchor-a".into(),
                                output_key: "anchor-a".into(),
                            },
                        ],
                    ),
                }))
                .expect_source(observed.observed_sources()[0].clone())
                .idempotency(&9_764_u64);
            let Preparation::Prepared(prepared) = mutation
                .prepare_in_program(
                    &application,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
            else {
                panic!("the fresh mutation prepares its unpublished candidate");
            };
            let policy = super::support::CHECKPOINT_EXECUTION_POLICY.budget();
            let restore = Restore(
                place(placement),
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory {
                        0
                    } else {
                        policy.charged_memory_bytes()
                    },
                    if zero_memory {
                        policy.work_ceiling()
                    } else {
                        0
                    },
                ))),
            );
            let reads_before = reads();
            reports();
            let Outcome::Commit(Uncommitted::Denied(denial)) = prepared.commit(
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            ) else {
                panic!("the commit's own request refuses before publishing");
            };
            let CommitKind::ExecutionResource {
                denial: resource,
                partition_identity,
                policy_ancestor,
            } = denial.kind()
            else {
                panic!("the commit retains its opening cause: {:?}", denial.kind());
            };
            if zero_memory && placement == Placement::Serial {
                assert_eq!(resource, Resource::PolicyMemoryLimit);
            } else if zero_memory {
                let Resource::MemoryLimit {
                    level,
                    requested,
                    admitted,
                } = resource
                else {
                    panic!("the exact lease bytes are retained");
                };
                assert_eq!(level, Level::Policy);
                assert_eq!(admitted, 0);
                assert!(requested > 0);
            } else {
                assert_eq!(resource, Resource::WorkExhausted);
            }
            assert_eq!(partition_identity, None);
            assert_eq!(policy_ancestor, None);
            assert_eq!(reads(), reads_before);
            assert_eq!(reports().len(), 1);
            drop(restore);
            let after = request
                .query(PlanarRead {
                    body_key: "anchor-a".into(),
                })
                .execute()
                .unwrap();
            assert_eq!(after.receipt().inspect().basis().version(), before);
            let mut mutation = request
                .mutate(crate::PlanarEdit(crate::PlanarMutation {
                    scope_key: "anchor-a".into(),
                    operation: worth_query_consumer_values::PlanarOperation::VerifyCurrentOutputs(
                        vec![
                            worth_query_consumer_values::PlanarCurrentOutputExpectation {
                                producer_key: "anchor-a".into(),
                                output_key: "anchor-a".into(),
                            },
                        ],
                    ),
                }))
                .expect_source(observed.observed_sources()[0].clone())
                .idempotency(&9_765_u64);
            let Preparation::Prepared(prepared) = mutation
                .prepare_in_program(
                    &application,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
            else {
                panic!("the fresh mutation prepares its unpublished candidate");
            };
            let before = reads();
            let _ = prepared.commit(
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
            assert!(
                reads() > before,
                "admitted commit contacts its revalidation reader"
            );
        }
    }
}

#[test]
fn mutation_handler_and_inline_commit_remain_in_one_request() {
    use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome as Outcome;
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let before = observed.receipt().inspect().basis().version();
    let policy = support::CHECKPOINT_EXECUTION_POLICY.budget();
    // This handler has no managed computation, and the ordinary Relational
    // reads are inline until part two. One is the least positive opening
    // allowance. This mutation reaches no Signal dispatch: Relational commit
    // carriage (and its exhaustion probe) belongs to part two.
    let restore = Restore(
        place(Placement::Leased(NonZeroUsize::MIN)),
        bound(Some(worth_foundational::ExecutionBudget::new(
            NonZeroUsize::MIN,
            policy.charged_memory_bytes(),
            1,
        ))),
    );
    let contacts = crate::handler::handler_contacts();
    reports();
    let outcome = request
        .mutate(crate::PlanarEdit(crate::PlanarMutation {
            scope_key: "anchor-a".into(),
            operation: worth_query_consumer_values::PlanarOperation::Adjust(vec![
                worth_query_consumer_values::PlanarAdjustment {
                    body_key: "anchor-a".into(),
                    replacement_y: super::length(3),
                },
            ]),
        }))
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&9_766_u64)
        .execute_in_program(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert_eq!(crate::handler::handler_contacts(), contacts + 1);
    assert!(matches!(outcome, Outcome::Committed { .. }));
    let opened = reports();
    assert_eq!(opened.len(), 1);
    assert_eq!(opened[0].as_ref().unwrap().charged_work(), 0);
    drop(restore);
    let after = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_ne!(after.receipt().inspect().basis().version(), before);
}

mod mutation_entries;
