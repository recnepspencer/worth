//! A competing host reservation taken while an evaluator runs is met with a
//! typed refusal, never a partial publication. Since `20a4797833` a checked
//! epoch admits its callback results, apply map and candidate lookup map
//! from actual measured bytes after the callback returns, so host room taken
//! during the callback stops the epoch before anything is published, and the
//! same evaluation succeeds once the room is released.
//!
//! The competitor fills the whole process memory, so this test owns its
//! process and its authority: no other test can starve beside it.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread::JoinHandle;

use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionMap, LeaseDenial,
    LeaseRequest, MapPartition, MemoryLimitDenial, MemoryLimitLevel, PreparedExecutionMap,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

use worth_signal::facade::adapters::NodeContract;
use worth_signal::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, ChangedRegion, DeclaredSignalInput, DependencyEdge,
    NodeEvaluationResult, PartitionSubscription, RunMode, ScopePath, SignalError,
    SignalExecutionStopReason, SignalGraph, SignalLeaseDenial, SignalPublicationDisposition,
};

const VALUE: Aspect = Aspect::new(0);
const HOST_MEMORY: u64 = 512 * 1024 * 1024;

type HostTicket = PreparedExecutionMap<'static, u8, u8, u8, ()>;

fn authority() -> &'static ExecutionAuthority {
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(HOST_MEMORY),
        })
        .expect("this test process's one authority")
    })
}

fn request(workers: usize, work: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                NonZeroUsize::new(workers).unwrap(),
                Some(32 * 1024 * 1024),
                Some(work),
            ),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

fn competing_ticket(scratch_bytes: u64) -> Result<HostTicket, LeaseDenial> {
    let mut host_request = request(1, 1_000_000);
    host_request.policy = ExecutionRequestPolicy::new(
        host_request.policy.posture(),
        host_request.policy.determinism(),
        ExecutionBudget::new(
            host_request.policy.max_workers(),
            Some(HOST_MEMORY),
            Some(1_000_000),
        ),
    );
    let lease = authority().request_lease(host_request)?;
    let identity = PartitionIdentity::new(1);
    let map = ExecutionMap::try_from_declared_partitions(
        vec![identity],
        vec![MapPartition {
            identity,
            value: 0_u8,
            read_keys: vec![0_u8],
            write_keys: Vec::new(),
            kernel_scratch_bytes: scratch_bytes,
            max_result_bytes: 0,
        }],
    )
    .expect("one declared read has canonical access");
    map.prepare_run(lease)
}

fn hold_remaining_host_memory(release: mpsc::Receiver<()>, ready: mpsc::Sender<bool>) {
    let mut low = 0_u64;
    let mut high = HOST_MEMORY;
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if competing_ticket(middle).is_ok() {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    let held = competing_ticket(low).expect("the measured host capacity remains stable");
    // With the host saturated, any later admission, even an empty map, fails.
    let late_map_denied = competing_ticket(0).is_err();
    ready
        .send(late_map_denied)
        .expect("checked evaluator awaits host reservation");
    release.recv().expect("test releases competing host memory");
    drop(held);
}

#[test]
fn host_contention_during_the_callback_refuses_the_epoch_before_publication() {
    let mut graph = SignalGraph::new();
    let producer = graph
        .node()
        .with_contract(
            NodeContract::wildcard()
                .with_produces(VALUE)
                .with_bounded_inputs(BoundedSignalInputs::default()),
        )
        .build();
    let consumer = graph
        .node()
        .with_contract(
            NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new([
                DeclaredSignalInput::scoped(
                    producer,
                    VALUE,
                    PartitionSubscription::exact(ScopePath::one("desk").unwrap()),
                ),
            ])),
        )
        .build();
    graph
        .set_dependencies(
            consumer,
            [DependencyEdge::with_partition_scope(
                producer,
                VALUE,
                PartitionSubscription::exact(ScopePath::one("desk").unwrap()),
            )],
        )
        .unwrap();
    assert_eq!(graph.subscribers_of(producer).unwrap(), &[consumer]);
    let before = graph.node_aspect_version(producer).unwrap().get(VALUE);

    let lease = authority().request_lease(request(2, 2_000_000)).unwrap();
    let calls = AtomicUsize::new(0);
    let blocker = Mutex::<Option<(mpsc::Sender<()>, JoinHandle<()>)>>::new(None);
    let evaluate = |graph: &mut SignalGraph| {
        graph.evaluate_checked(
            &[producer],
            RunMode::Default,
            &(),
            &|_| {
                // Only the first evaluation meets a saturated host.
                if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                    let (ready_tx, ready_rx) = mpsc::channel();
                    let (release_tx, release_rx) = mpsc::channel();
                    let thread = std::thread::spawn(move || {
                        hold_remaining_host_memory(release_rx, ready_tx)
                    });
                    assert!(
                        ready_rx
                            .recv()
                            .expect("host reservation arrives before callback returns"),
                        "the competitor must exhaust host memory for any later admission"
                    );
                    *blocker.lock().unwrap() = Some((release_tx, thread));
                }
                Ok(
                    NodeEvaluationResult::from_version(AspectVersion::zero().with(VALUE, 7))
                        .with_changed_aspect_region(VALUE, ChangedRegion::new("desk")),
                )
            },
            worth_execution::ExecutionRequest::leased(&lease),
        )
    };
    let contended = evaluate(&mut graph);
    let (release, thread) = blocker.lock().unwrap().take().expect("callback ran once");
    release.send(()).unwrap();
    thread.join().unwrap();
    let error = contended.expect_err("measured results are admitted after the callback");
    let SignalError::ExecutionStopped(stop) = error else {
        panic!("host contention is a typed execution stop: {error:?}");
    };
    let SignalExecutionStopReason::Admission(SignalLeaseDenial::MemoryExhausted(memory)) =
        stop.reason()
    else {
        panic!(
            "host contention refuses memory admission: {:?}",
            stop.reason()
        );
    };
    assert!(
        matches!(
            memory,
            MemoryLimitDenial::Capacity { requested, admitted, level: MemoryLimitLevel::Process }
                if requested > admitted
        ),
        "{memory:?}"
    );
    assert_eq!(
        stop.disposition(),
        SignalPublicationDisposition::WorkerLocal
    );
    assert_eq!(stop.publication_progress().completed_tasks(), 0);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        graph.node_aspect_version(producer).unwrap().get(VALUE),
        before
    );

    // The refusal published nothing, so the released host admits the same epoch.
    let report = evaluate(&mut graph).expect("released host memory admits the epoch");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(report.tasks_executed, 1);
    assert_eq!(graph.node_aspect_version(producer).unwrap().get(VALUE), 7);
}
