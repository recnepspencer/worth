//! A child denial is owner-completion data, never a wave failure.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::computation::nested_failure::MemberComputeFailure;
use crate::domain_computation::primary_graph::application_contribution::computation::wave_dispatch::tests::*;

#[test]
fn an_unrelated_error_cannot_take_an_earlier_requests_stop() {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    for width in [None, Some(1), Some(2), Some(4)] {
        let later_runs = Arc::new(AtomicUsize::new(0));
        let count = later_runs.clone();
        let member = PreparedMember {
            requests: vec![
                PreparedCompute::new_for_test(|member| {
                    member.compute_for_test(|managed, _poll, _transport| {
                        let map = ExecutionMap::<_, u64>::from_keyless_partitions(
                            [(
                                PartitionIdentity::new(1),
                                worth_execution::KeylessPartition {
                                    value: 1_u64,
                                    kernel_scratch_bytes: 0,
                                    max_result_bytes: 0,
                                },
                            )]
                            .into_iter()
                            .collect(),
                        )
                        .unwrap();
                        let held = (*managed.request_for_test()).reserve(0).unwrap();
                        let nested = (*managed.request_for_test())
                            .dispatch_for_test()
                            .unwrap()
                            .map_for_test(10, &map, held, |_, context| {
                                context.checkpoint(2)?;
                                Err::<u64, _>(MapKernelFailure::Domain(()))
                            })
                            .unwrap();
                        assert!(matches!(nested, MapOutcome::Stopped { .. }));
                        // This request deliberately ignores its own cause.
                        Ok::<_, ()>(())
                    })
                }),
                PreparedCompute::new_for_test(move |member| {
                    member.compute_for_test(|_, _, _transport| {
                        count.fetch_add(1, Ordering::SeqCst);
                        Err::<(), _>("unrelated failure")
                    })
                }),
            ],
        };
        let settled = run_stopped(width, vec![member, member_result(2, false)]);
        assert!(
            matches!(settled[0], Err(WorthQueryAdvancementDenial::NestedStopped)),
            "an unrelated failure cannot acknowledge an earlier request's nested stop"
        );
        assert_eq!(
            later_runs.load(Ordering::SeqCst),
            0,
            "the next prepared request cannot start with an untaken stop"
        );
    }
}

#[test]
fn taken_partition_denials_leave_siblings_for_owner_completion() {
    for width in [None, Some(1), Some(2), Some(4)] {
        let failed = PreparedMember {
            requests: vec![PreparedCompute::new_for_test(|member| {
                member.compute_for_test(|managed, _poll, transport| {
                    let map = ExecutionMap::<_, u64>::from_keyless_partitions(
                        vec![(
                            PartitionIdentity::new(1),
                            worth_execution::KeylessPartition {
                                value: 1_u64,
                                kernel_scratch_bytes: 0,
                                max_result_bytes: 0,
                            },
                        )]
                        .into_iter()
                        .collect(),
                    )
                    .unwrap();
                    let held = (*managed.request_for_test()).reserve(0).unwrap();
                    let dispatch = (*managed.request_for_test()).dispatch_for_test().unwrap();
                    transport
                        .call_for_test(|| {
                            let outcome = dispatch
                                .map_for_test(10, &map, held, |_, context| {
                                    context.checkpoint(2)?;
                                    Err::<u64, _>(MapKernelFailure::Domain(()))
                                })
                                .unwrap();
                            match outcome {
                                MapOutcome::Complete { .. } => Ok::<u64, MapStop<()>>(1),
                                MapOutcome::Stopped { reason, .. } => Err(reason),
                            }
                        })
                        .map_err(MemberComputeFailure::into_failure_for_test)
                })
            })],
        };
        let (members, report) = run(width, vec![failed, member_result(2, false)]);
        assert_eq!(
            members.len(),
            2,
            "the sibling has its canonical completion slot"
        );
        let mut members = members.into_iter();
        let mut denied = members.next().unwrap().unwrap().into_slots_for_test();
        assert_eq!(denied.len(), 1);
        assert_eq!(
            denied
                .pop()
                .unwrap()
                .into_value_for_test::<Result<u64, MapStop<()>>>(),
            Err(MapStop::Failure {
                identity: PartitionIdentity::new(1),
                cause: MapKernelFailure::Domain(())
            })
        );
        let mut sibling = members.next().unwrap().unwrap().into_slots_for_test();
        assert_eq!(
            sibling.pop().unwrap().into_value_for_test::<(u64, bool)>(),
            (2, false)
        );

        assert_eq!(report.accounted_cost(), None);
    }
}
