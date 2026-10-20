//! An author-held host handle must receive the same refusal inside partition work.
worth_execution::needs_os_threads! {
use super::*;
const HANG_GUARD:Duration=Duration::from_secs(60);
#[derive(Debug)]
struct Probe {
    marked: bool,
    thread: std::thread::ThreadId,
    refusal: Result<(), WorthQueryAdvancementDenial>,
}
impl worth_execution::ChargedBytes for Probe {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
#[test]
fn nested_partition_workers_keep_the_member_reentry_refusal() {
    let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
    let application = Arc::new(fixture::installed_authorization_world(false).application);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
    );
    let root = QueryRequestExecution::open(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Leased {
            authority: test_authority(),
            policy: test_policy(NonZeroUsize::new(4).unwrap(), None),
        },
        &scope,
    );
    let witnessed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let recorded = witnessed.clone();
    let rendezvous = Arc::new((std::sync::Mutex::new(0_usize), std::sync::Condvar::new()));
    let watchdog = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let exceeded = watchdog.clone();
    let member = PreparedMember {
        requests: vec![PreparedCompute::new(
            0,
            4096,
            move |member| {
                let (managed, _) = member.into_parts();
                let map = worth_execution::ExecutionMap::<_, u64>::from_keyless_partitions(
                    (0..4)
                        .map(|rank| {
                            (
                                worth_foundational::PartitionIdentity::new(rank),
                                worth_execution::KeylessPartition {
                                    value: rank,
                                    kernel_scratch_bytes: 0,
                                    max_result_bytes: std::mem::size_of::<Probe>() as u64,
                                },
                            )
                        })
                        .collect(),
                )
                .unwrap();
                let held = managed.request.reserve(0).unwrap();
                let dispatch = managed.request.dispatch().unwrap();
                let result = dispatch
                    .map(
                        32,
                        &map,
                        held,
                        |_, context| -> Result<_, worth_execution::MapKernelFailure<()>> {
                            context
                                .checkpoint(1)
                                .map_err(worth_execution::MapKernelFailure::Stop)?;
                            let (entered, ready) = &*rendezvous;
                            let mut entered = entered.lock().unwrap();
                            *entered += 1;
                            ready.notify_all();
                            let (entered, timed) = ready
                                .wait_timeout_while(entered, HANG_GUARD, |entered| {
                                    *entered < 2
                                })
                                .unwrap();
                            drop(entered);
                            exceeded
                                .fetch_or(timed.timed_out(), std::sync::atomic::Ordering::Release);
                            let probe = Probe {
                                marked: active(),
                                thread: std::thread::current().id(),
                                refusal: application.with_host_advancement(|_| ()),
                            };
                            recorded.lock().unwrap().push(Probe {
                                marked: probe.marked,
                                thread: probe.thread,
                                refusal: probe.refusal,
                            });
                            Ok(probe)
                        },
                    )
                    .unwrap();
                matches!(result, worth_execution::MapOutcome::Complete { .. })
            },
            |_| 0,
        )],
    };
    let (settled, _) = root
        .execution_request()
        .unwrap()
        .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
            wave_dispatch::dispatch(&root, vec![member])
        })
        .unwrap();
    assert!(
        !watchdog.load(std::sync::atomic::Ordering::Acquire),
        "the nested worker witness must enter on two workers"
    );
    let probes = witnessed.lock().unwrap();
    assert!(
        probes.iter().any(|probe| probe.thread != probes[0].thread),
        "the witness runs on distinct computing threads: {probes:?}"
    );
    assert!(!probes.is_empty(), "the nested witness entered its kernel");
    assert!(
        probes.iter().all(|probe| probe.marked
            && probe.refusal == Err(WorthQueryAdvancementDenial::NestedOpening)),
        "a nested worker lost the member marker or changed its refusal: {probes:?}"
    );
    assert_eq!(probes.len(), 4);
    assert!(
        settled.into_iter().next().unwrap().is_ok(),
        "the member finishes its refused calls"
    );
    assert!(!active());
}
}
