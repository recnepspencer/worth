//! Certification is a diagnostic rerun, never part of an advancement's bill.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::{
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
use worth_execution::MapKernelContext;

fn reducing_member(
    calls: Arc<Mutex<Vec<(std::thread::ThreadId, bool)>>>,
    rendezvous: Option<Arc<(Mutex<usize>, std::sync::Condvar)>>,
) -> PreparedMember {
    PreparedMember {
        requests: vec![PreparedCompute::new(move |member| {
            if let Some(rendezvous) = &rendezvous {
                let mut entered = rendezvous.0.lock().unwrap();
                *entered += 1;
                rendezvous.1.notify_all();
                let (entered, _) = rendezvous
                    .1
                    .wait_timeout_while(entered, Duration::from_secs(60), |n| *n < 2)
                    .unwrap();
                assert_eq!(
                    *entered, 2,
                    "diagnostic member rendezvous hang guard expired: second slot unavailable"
                );
            }
            member.compute(|managed, _, _, _| {
                let invocations = std::sync::atomic::AtomicUsize::new(0);
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
                let kernel = |value: &u64, context: &mut MapKernelContext<'_, '_>| {
                    calls.lock().unwrap().push((std::thread::current().id(), invocations.fetch_add(1, std::sync::atomic::Ordering::Relaxed) >= 2));
                    context.checkpoint(*value)?;
                    Ok::<_, MapKernelFailure<()>>(*value)
                };
                let (tree, report, _) = managed.request.dispatch().unwrap().reduce(
                    u64::MAX, &map, inputs, &mut held,
                    (kernel, kernel), 0_u64, |left, right| left + right, 8,
                    crate::domain_computation::primary_graph::application_contribution::partitioned_computation::WorthQueryPartitionedComputationFullCause::Unretained,
                ).unwrap().unwrap();
                Ok::<_, ()>((*tree.result(), report.charged_work(), report.charged_span()))
            })
        })],
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
            let rendezvous =
                (width > 1).then(|| Arc::new((Mutex::new(0), std::sync::Condvar::new())));
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

            let calls = calls.lock().unwrap();
            assert_eq!(
                calls.len(),
                width * if certified { 6 } else { 2 },
                "only an armed run executes the two certification reductions"
            );
            if width == 2 && certified {
                assert!(
                    calls
                        .iter()
                        .any(|(thread, diagnostic)| *diagnostic && *thread != caller),
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
