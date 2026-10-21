//! A take transports its own cause and changes neither work nor span.
use super::super::super::nested_failure::MemberComputeFailure;
use super::*;

fn failing_member(take: bool, request_owned: bool) -> PreparedMember {
    PreparedMember {
        requests: vec![PreparedCompute::new(
            0,
            0,
            move |member| {
                member.compute(|managed, mut charge, transport| {
                    charge.as_mut().unwrap()(1).unwrap();
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
                    let nested = || {
                        let held = managed.request.reserve(0).unwrap();
                        let outcome = managed
                            .request
                            .dispatch()
                            .unwrap()
                            .map(
                                if request_owned { u64::MAX } else { 10 },
                                &map,
                                held,
                                |_, context| {
                                    // One unit is already charged on the member.
                                    // MAX exceeds the request's remaining work,
                                    // not a narrower computation declaration.
                                    context.checkpoint(if request_owned { u64::MAX } else { 2 })?;
                                    Err::<u64, _>(MapKernelFailure::Domain(()))
                                },
                            )
                            .unwrap();
                        match outcome {
                            MapOutcome::Complete { .. } => panic!("the declared child must fail"),
                            MapOutcome::Stopped { reason, .. } => {
                                if request_owned {
                                    assert_eq!(
                                        reason,
                                        MapStop::WorkExhausted {
                                            identity: PartitionIdentity::new(1)
                                        }
                                    );
                                }
                                Err::<u64, _>(reason)
                            }
                        }
                    };
                    if take {
                        let failure = transport.call(nested);
                        assert_eq!(
                            matches!(&failure, Err(MemberComputeFailure::Nested(_))),
                            !request_owned,
                            "only the nested call's computational cause builds transported data"
                        );
                        failure.map_err(MemberComputeFailure::into_failure)
                    } else {
                        nested()
                    }
                })
            },
            |_| 0,
        )],
    }
}

#[test]
fn taking_changes_the_outcome_but_never_its_work_or_span() {
    for width in [None, Some(1), Some(2), Some(4)] {
        for request_owned in [false, true] {
            let (taken, taken_report) = run(width, vec![failing_member(true, request_owned)]);
            let (untaken, untaken_report) = run(width, vec![failing_member(false, request_owned)]);
            assert_eq!(taken.len(), 1);
            assert_eq!(untaken.len(), 1);
            assert!(matches!(
                untaken[0],
                Err(WorthQueryAdvancementDenial::NestedStopped)
            ));
            if request_owned {
                assert!(matches!(
                    taken[0],
                    Err(WorthQueryAdvancementDenial::NestedStopped)
                ));
            } else {
                let failure = taken
                    .into_iter()
                    .next()
                    .unwrap()
                    .unwrap()
                    .into_slots()
                    .pop()
                    .unwrap()
                    .into_value::<Result<u64, MapStop<()>>>();
                assert_eq!(
                    failure,
                    Err(MapStop::Failure {
                        identity: PartitionIdentity::new(1),
                        cause: MapKernelFailure::Domain(()),
                    })
                );
            }
            let declared_work = 1 + if request_owned { 0 } else { 2 };

            assert_eq!(taken_report.charged_span(), declared_work);

            assert_eq!(taken_report.charged_span(), untaken_report.charged_span());
        }
    }
}
