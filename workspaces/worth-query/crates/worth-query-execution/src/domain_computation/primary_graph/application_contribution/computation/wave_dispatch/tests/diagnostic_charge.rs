//! Certification is a diagnostic rerun, never part of an advancement's bill.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::{
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};

fn reducing_member(
    calls: Arc<Mutex<Vec<std::thread::ThreadId>>>,
    rendezvous: Option<Arc<std::sync::Barrier>>,
) -> PreparedMember {
    PreparedMember {
        requests: vec![PreparedCompute::new(
            0,
            0,
            move |member| {
                if let Some(rendezvous) = &rendezvous {
                    rendezvous.wait();
                }
                member.compute(|managed, _, _| {
                let map = ExecutionMap::<_, u64>::from_keyless_partitions(
                    [3_u64, 7].into_iter().enumerate().map(|(rank, value)| (
                        PartitionIdentity::new(rank as u64 + 1),
                        worth_execution::KeylessPartition {
                            value, kernel_scratch_bytes: 0, max_result_bytes: 0,
                        },
                    )).collect(),
                ).unwrap();
                let inputs = managed.request.reserve(0).unwrap();
                let mut held = managed.request.reserve(0).unwrap();
                let (tree, report, _) = managed.request.dispatch().unwrap().reduce(
                    u64::MAX, &map, inputs, &mut held,
                    |value, context| {
                        calls.lock().unwrap().push(std::thread::current().id());
                        context.checkpoint(*value)?;
                        Ok::<_, MapKernelFailure<()>>(*value)
                    }, 0_u64, |left, right| left + right, 8,
                    crate::domain_computation::primary_graph::application_contribution::partitioned_computation::WorthQueryPartitionedComputationFullCause::Unretained,
                ).unwrap().unwrap();
                Ok::<_, ()>((*tree.result(), report.charged_work(), report.charged_span()))
            })
            },
            |_| 0,
        )],
    }
}

#[test]
fn certification_never_changes_the_advancements_work_or_span() {
    let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
    let caller = std::thread::current().id();
    for width in [1, 2] {
        let mut bills = Vec::new();
        for certified in [false, true] {
            let workers = NonZeroUsize::new(width).unwrap();
            place(if certified {
                Placement::Certified { workers, seed: 1 }
            } else {
                Placement::Leased(workers)
            });
            let calls = Arc::new(Mutex::new(Vec::new()));
            let rendezvous = (width > 1).then(|| Arc::new(std::sync::Barrier::new(width)));
            let (settled, report) = run(
                Some(width),
                (0..width)
                    .map(|_| reducing_member(calls.clone(), rendezvous.clone()))
                    .collect(),
            );
            let values = settled
                .into_iter()
                .map(|settled| {
                    settled
                        .unwrap()
                        .into_slots()
                        .pop()
                        .unwrap()
                        .into_value::<Result<(u64, u64, u64), ()>>()
                        .unwrap()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                values,
                vec![(10, 25, 22); width],
                "the declared two-input reduction costs 25 work and 22 span"
            );
            assert_eq!(
                (report.charged_work(), report.charged_span()),
                (25 * u64::try_from(width).unwrap(), 22)
            );
            let calls = calls.lock().unwrap();
            assert_eq!(
                calls.len(),
                width * if certified { 6 } else { 2 },
                "only an armed run executes the two certification reductions"
            );
            if width == 2 && certified {
                assert!(
                    calls.iter().any(|thread| *thread != caller),
                    "a descendant worker really certifies its owned reduction"
                );
            }
            bills.push((values, report.charged_work(), report.charged_span()));
        }
        assert_eq!(
            bills[0], bills[1],
            "a diagnostic certification never changes the advancement's work or span"
        );
    }
}
