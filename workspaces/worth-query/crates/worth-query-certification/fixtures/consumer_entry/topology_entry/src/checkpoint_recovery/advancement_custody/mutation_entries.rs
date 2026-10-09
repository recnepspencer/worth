use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial as MutationDenial;

#[test]
fn mutation_prepare_and_one_call_refuse_before_their_admitted_reader() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    let policy = support::CHECKPOINT_EXECUTION_POLICY.budget();
    let mut key = 97_600_u64;
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        for entry in 0..3 {
            for zero_memory in [false, true] {
                bound(None);
                let observed = request
                    .query(PlanarRead {
                        body_key: "anchor-a".into(),
                    })
                    .execute()
                    .unwrap();
                let intent = || {
                    crate::PlanarEdit(crate::PlanarMutation {
                        scope_key: "anchor-a".into(),
                        operation:
                            worth_query_consumer_values::PlanarOperation::VerifyCurrentOutputs(
                                vec![
                                    worth_query_consumer_values::PlanarCurrentOutputExpectation {
                                        producer_key: "anchor-a".into(),
                                        output_key: "anchor-a".into(),
                                    },
                                ],
                            ),
                    })
                };
                let before = reads();
                let mutation = request
                    .mutate(intent())
                    .expect_source(observed.observed_sources()[0].clone());
                match entry {
                    0 => {
                        drop(
                            mutation
                                .idempotency(&key)
                                .prepare_in_program(&application, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
                                .unwrap(),
                        );
                    }
                    1 => {
                        mutation
                            .idempotency(&key)
                            .execute_in_program(&application, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
                            .unwrap();
                    }
                    2 => {
                        mutation.assess_current_authorization().unwrap();
                    }
                    _ => unreachable!(),
                }
                assert!(
                    reads() > before,
                    "admitted mutation entry must reach its reader"
                );
                key += 1;
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
                )));
                let before = reads();
                let mutation = request
                    .mutate(intent())
                    .expect_source(observed.observed_sources()[0].clone());
                let denial = match entry {
                    0 => mutation
                        .idempotency(&key)
                        .prepare_in_program(&application, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
                        .err()
                        .unwrap(),
                    1 => mutation
                        .idempotency(&key)
                        .execute_in_program(&application, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
                        .err()
                        .unwrap(),
                    2 => mutation.assess_current_authorization().err().unwrap(),
                    _ => unreachable!(),
                };
                let MutationDenial::ExecutionRequest(cause) = denial else {
                    panic!("no handler ran before opening: {denial:?}");
                };
                assert_eq!(reads(), before);
                assert_opening_cause(cause, placement, zero_memory);
                key += 1;
            }
        }
    }
}

fn assert_opening_cause(cause: Denial, placement: Placement, zero_memory: bool) {
    if !zero_memory {
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
            panic!("leased opening must retain bytes: {cause:?}");
        };
        assert_eq!(level, Level::Policy);
        assert!(requested > 0);
        assert_eq!(admitted, 0);
    }
}
