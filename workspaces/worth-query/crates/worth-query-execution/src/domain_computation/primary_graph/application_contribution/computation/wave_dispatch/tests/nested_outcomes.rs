//! A child denial is owner-completion data, never a wave failure.
use super::super::super::nested_failure::MemberComputeFailure;
use super::*;

#[test]
fn an_unrelated_error_cannot_take_an_earlier_requests_stop() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    for width in [None, Some(1), Some(2), Some(4)] {
        let later_runs = Arc::new(AtomicUsize::new(0));
        let count = later_runs.clone();
        let member = PreparedMember {
            requests: vec![
                PreparedCompute::new(
                    0,
                    0,
                    |member| {
                        member.compute(|managed, _charge, _transport| {
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
                            let held = managed.request.reserve(0).unwrap();
                            let nested = managed
                                .request
                                .dispatch()
                                .unwrap()
                                .map(10, &map, held, |_, context| {
                                    context.checkpoint(2)?;
                                    Err::<u64, _>(MapKernelFailure::Domain(()))
                                })
                                .unwrap();
                            assert!(matches!(nested, MapOutcome::Stopped { .. }));
                            // This request deliberately ignores its own cause.
                            Ok::<_, ()>(())
                        })
                    },
                    |_| 0,
                ),
                PreparedCompute::new(
                    0,
                    0,
                    move |member| {
                        member.compute(|_, _, _transport| {
                            count.fetch_add(1, Ordering::SeqCst);
                            Err::<(), _>("unrelated failure")
                        })
                    },
                    |_| 0,
                ),
            ],
        };
        let (settled, report) = run(width, vec![member, charged(2, 1, false)]);
        assert!(
            matches!(settled[0], Err(WorthQueryAdvancementDenial::NestedStopped)),
            "an unrelated failure cannot acknowledge an earlier request's nested stop"
        );
        assert_eq!(
            later_runs.load(Ordering::SeqCst),
            0,
            "the next prepared request cannot start with an untaken stop"
        );

        assert_eq!(report.charged_span(), 2);
    }
}

#[test]
fn taken_partition_denials_leave_siblings_for_owner_completion() {
    for width in [None, Some(1), Some(2), Some(4)] {
        let failed = PreparedMember {
            requests: vec![PreparedCompute::new(
                0,
                0,
                |member| {
                    member.compute(|managed, _charge, transport| {
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
                        let held = managed.request.reserve(0).unwrap();
                        let dispatch = managed.request.dispatch().unwrap();
                        transport
                            .call(|| {
                                let outcome = dispatch
                                    .map(10, &map, held, |_, context| {
                                        context.checkpoint(2)?;
                                        Err::<u64, _>(MapKernelFailure::Domain(()))
                                    })
                                    .unwrap();
                                match outcome {
                                    MapOutcome::Complete { .. } => Ok::<u64, MapStop<()>>(1),
                                    MapOutcome::Stopped { reason, .. } => Err(reason),
                                }
                            })
                            .map_err(MemberComputeFailure::into_failure)
                    })
                },
                |_| 0,
            )],
        };
        let (members, report) = run(width, vec![failed, charged(2, 1, false)]);
        assert_eq!(
            members.len(),
            2,
            "the sibling has its canonical completion slot"
        );
        let mut members = members.into_iter();
        let mut denied = members.next().unwrap().unwrap().into_slots();
        assert_eq!(denied.len(), 1);
        assert_eq!(
            denied
                .pop()
                .unwrap()
                .into_value::<Result<u64, MapStop<()>>>(),
            Err(MapStop::Failure {
                identity: PartitionIdentity::new(1),
                cause: MapKernelFailure::Domain(())
            })
        );
        let mut sibling = members.next().unwrap().unwrap().into_slots();
        assert_eq!(
            sibling.pop().unwrap().into_value::<(u64, bool)>(),
            (2, false)
        );

        // The denied member spans two units; its sibling spans one.
        assert_eq!(report.charged_span(), 2);
    }
}
