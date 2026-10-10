//! A child denial is owner-completion data, never a wave failure.
use super::*;

#[test]
fn taken_partition_denials_leave_siblings_for_owner_completion() {
    for width in [None, Some(1), Some(2), Some(4)] {
        let failed = PreparedMember {
            requests: vec![PreparedCompute::new(
                0,
                0,
                |member| {
                    member.compute(|managed, _charge| {
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
        assert_eq!(report.charged_work(), 2 + 1);
        // The denied member spans two units; its sibling spans one.
        assert_eq!(report.charged_span(), 2);
    }
}
