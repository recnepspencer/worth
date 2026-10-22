//! Certification reruns canonical native reduction without creating accounting.
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
            member.compute(|managed, _, _| {
                let invocations = std::sync::atomic::AtomicUsize::new(0);
                let map = ExecutionMap::<_, ()>::from_native_partitions(
                    [3_u64, 7].into_iter().enumerate().map(|(rank, value)| (
                        PartitionIdentity::new(rank as u64 + 1),
                        value,
                    )).collect(),
                ).unwrap();
                let kernel = |value: &u64, context: &mut MapKernelContext<'_, '_>| {
                    calls.lock().unwrap().push((std::thread::current().id(), invocations.fetch_add(1, std::sync::atomic::Ordering::Relaxed) >= 2));
                    context.poll()?;
                    Ok::<_, MapKernelFailure<()>>(*value)
                };
                let result = managed.request.dispatch().unwrap().reduce_retained(
                    None, &map, (kernel, kernel), 0_u64, |left, right| left + right,
                    crate::domain_computation::primary_graph::application_contribution::partitioned_computation::WorthQueryPartitionedComputationFullCause::Unretained,
                ).unwrap();
                assert_eq!(result.metrics.accounted_metrics(), None);
                Ok::<_, ()>((*result.tree.result(), result.report.accounted_cost()))
            })
        })],
    }
}

#[test]
fn certification_preserves_nested_results_without_billing_the_advancement() {
    let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
    let caller = std::thread::current().id();
    for width in [1, 2] {
        let mut outcomes = Vec::new();
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
                        .into_value::<Result<(u64, Option<(u64, u64)>), ()>>()
                        .unwrap()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                values,
                vec![(10, None); width],
                "the native two-input reduction preserves its result without accounting"
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
            assert_eq!(report.accounted_cost(), None);
            outcomes.push(values);
        }
        assert_eq!(
            outcomes[0], outcomes[1],
            "diagnostic certification preserves canonical nested results"
        );
    }
}
