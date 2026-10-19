//! A member's nested map spends its advancement's one bound.
use super::*;
fn partitions(count: u64) -> ExecutionMap<(), u64> {
    ExecutionMap::from_keyless_partitions(
        (0..count)
            .map(|rank| {
                (
                    PartitionIdentity::new(rank),
                    KeylessPartition {
                        value: (),
                        kernel_scratch_bytes: 0,
                        max_result_bytes: 0,
                    },
                )
            })
            .collect(),
    )
    .unwrap()
}
fn nested(
    request: &QueryRequestExecution<'_>,
    units: u64,
) -> Result<MapOutcome<(), ()>, WorkCeilingDenial> {
    let dispatch = request
        .dispatch()
        .expect("a member dispatch has the parent's exact policy");
    dispatch.map(
        units,
        &partitions(1),
        request.reserve(0).unwrap(),
        |_, context| {
            context.checkpoint(units).map_err(MapKernelFailure::Stop)?;
            Ok(())
        },
    )
}
#[test]
fn wave_and_member_descendants_share_the_advancements_work_ceiling() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    const UNITS: u64 = 3;
    for posture in [
        Placement::Serial,
        Placement::Leased(NonZeroUsize::MIN),
        Placement::Leased(NonZeroUsize::new(4).unwrap()),
    ] {
        let _restore = Restore(
            place(posture),
            bound(Some(ExecutionBudget::new(
                NonZeroUsize::new(4).unwrap(),
                None,
                Some(2 * UNITS),
            ))),
        );
        observation::advancement_requests_on_this_thread_for_test();
        let root = request();
        let outcome = root
            .run_advancement(None, None, |_| {
                let wave = root.wave();
                wave.request()
                    .execution_request()
                    .unwrap()
                    .in_scope(|lease| {
                        partitions(3).run::<(), (), _>(lease, |_, _| {
                            let member = wave.member();
                            match nested(member.request(), UNITS).unwrap() {
                                MapOutcome::Complete { .. } => Ok(()),
                                MapOutcome::Stopped {
                                    reason:
                                        MapStop::WorkExhausted { .. }
                                        | MapStop::Failure {
                                            cause:
                                                MapKernelFailure::Stop(
                                                    worth_execution::MapKernelStop::WorkCeiling,
                                                ),
                                            ..
                                        },
                                    ..
                                } => Err(MapKernelFailure::Stop(
                                    worth_execution::MapKernelStop::WorkCeiling,
                                )),
                                MapOutcome::Stopped { reason, .. } => {
                                    panic!("member's nested run lost its lineage: {reason:?}")
                                }
                            }
                        })
                    })
                    .unwrap()
            })
            .unwrap();
        assert!(matches!(outcome, MapOutcome::Stopped {
            boundary: Some(identity), reason: MapStop::WorkExhausted { .. } | MapStop::Failure { cause: MapKernelFailure::Stop(worth_execution::MapKernelStop::WorkCeiling), .. }, ..
        } if identity == PartitionIdentity::new(2)));
        let reports = observation::advancement_requests_on_this_thread_for_test();
        assert_eq!(reports.len(), 1, "one request owns every descendant");
        assert_eq!(reports[0].as_ref().unwrap().charged_work(), 2 * UNITS);
    }
}
#[test]
fn a_partition_drawn_from_the_root_is_refused_inside_a_wave_member() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for workers in [NonZeroUsize::MIN, NonZeroUsize::new(4).unwrap()] {
        let _restore = Restore(place(Placement::Leased(workers)), bound(None));
        let root = request();
        root.run_advancement(None, None, |_| {
            let wave = root.wave();
            wave.request()
                .execution_request()
                .unwrap()
                .in_scope(|lease| {
                    let result = partitions(1).run::<(), (), _>(lease, |_, _| {
                        let member = wave.member();
                        assert!(matches!(
                            nested(member.request(), 1),
                            Ok(MapOutcome::Complete { .. })
                        ));
                        assert!(
                            matches!(
                                nested(&root, 1),
                                Err(WorkCeilingDenial::Admission(
                                    worth_execution::LeaseDenial::UnrelatedNestedLease,
                                ))
                            ),
                            "drawing a child from the root cannot replace the member's lineage"
                        );
                        // The attempted foreign nested run is itself a stopped pattern.
                        Err::<(), _>(MapKernelFailure::Domain(()))
                    });
                    assert!(matches!(
                        result,
                        MapOutcome::Stopped {
                            reason: MapStop::Failure {
                                cause: MapKernelFailure::Domain(()),
                                ..
                            },
                            ..
                        }
                    ));
                })
                .unwrap()
        })
        .unwrap();
    }
}
