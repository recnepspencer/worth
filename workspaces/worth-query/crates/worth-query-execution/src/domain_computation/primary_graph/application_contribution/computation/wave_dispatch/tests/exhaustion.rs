//! The request settles admitted increments and refuses the next canonical member.
use super::*;
#[test]
fn request_exhaustion_reports_the_declared_prefix_at_every_width() {
    const CHARGES: [u64; 4] = [3, 1, 4, 2];
    for prefix in 0..4 {
        for width in [None, Some(1), Some(2), Some(4)] {
            let cancellation = WorthQueryCancellationSource::new();
            let scope = WorthQueryRequestScope::new(
                Some(Instant::now() + Duration::from_secs(60)),
                cancellation.token(),
            );
            let policy = test_policy(NonZeroUsize::new(width.unwrap_or(1)).unwrap(), None);
            let placement = width.map_or(RuntimeWorldExecutionPlacement::Serial(policy), |_| {
                RuntimeWorldExecutionPlacement::Leased {
                    authority: test_authority(),
                    policy,
                }
            });
            let root = QueryRequestExecution::open(placement, &scope);
            let members = CHARGES
                .into_iter()
                .enumerate()
                .map(|(rank, units)| PreparedMember {
                    requests: vec![PreparedCompute::new(
                        0,
                        0,
                        move |member| {
                            let (_, mut charge) = member.into_parts();
                            charge.as_mut().unwrap()(units).map(|()| rank)
                        },
                        |_| 0,
                    )],
                })
                .collect();
            let admitted = CHARGES[..prefix].iter().sum::<u64>();
            let (settled, report) = root
                .execution_request()
                .unwrap()
                .run(
                    worth_execution::ExecutionWorkCeiling::new(Some(admitted)),
                    |_| dispatch(&root, members),
                )
                .unwrap();
            assert_eq!(settled.len(), prefix + 1);
            assert!(matches!(&settled[prefix], Err(WorthQueryAdvancementDenial::Resource(
                super::super::super::super::super::WorthQueryManagedComputationResourceDenial::WorkExhausted))));
            assert_eq!(report.charged_work(), admitted);
            let values: Vec<_> = settled
                .into_iter()
                .take(prefix)
                .map(|result| {
                    result
                        .unwrap()
                        .into_slots()
                        .pop()
                        .unwrap()
                        .into_value::<Result<usize, worth_execution::MapKernelStop>>()
                        .unwrap()
                })
                .collect();
            assert_eq!(values, (0..prefix).collect::<Vec<_>>());
        }
    }
}
