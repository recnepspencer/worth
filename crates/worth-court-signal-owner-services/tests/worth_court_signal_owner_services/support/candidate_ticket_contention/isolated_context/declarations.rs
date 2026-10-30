// D12 reserves checked-result custody and publication maps before callbacks.
// This ports master's post-callback refusal court (`2075ce36c1`): contention
// before admission refuses without effects; contention after admission cannot
// steal the epoch's custody; an oversized result violates its declared contract.
//
// The competitor fills the whole process memory, so this test owns its
// process and its authority: no other test can starve beside it.

pub(crate) use std::num::NonZeroUsize;
pub(crate) use std::thread::JoinHandle;
pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};
pub(crate) use {std::sync::mpsc, std::sync::Mutex, std::sync::OnceLock};

pub(crate) use {
    worth_execution::CancellationToken, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap,
    worth_execution::LeaseDenial, worth_execution::LeaseRequest, worth_execution::MapPartition,
    worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel,
    worth_execution::PreparedExecutionMap,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

pub(crate) use worth_signal::facade::adapters::NodeContract;
pub(crate) use {
    worth_signal::facade::Aspect, worth_signal::facade::AspectVersion,
    worth_signal::facade::BoundedSignalInputs, worth_signal::facade::ChangedRegion,
    worth_signal::facade::DeclaredSignalInput, worth_signal::facade::DependencyEdge,
    worth_signal::facade::NodeEvaluationResult, worth_signal::facade::NodeId,
    worth_signal::facade::PartitionSubscription, worth_signal::facade::RunMode,
    worth_signal::facade::ScopePath, worth_signal::facade::SignalError,
    worth_signal::facade::SignalExecutionFailure, worth_signal::facade::SignalExecutionStopReason,
    worth_signal::facade::SignalGraph, worth_signal::facade::SignalLeaseDenial,
    worth_signal::facade::SignalPublicationDisposition,
};

pub(crate) const VALUE: Aspect = worth_signal::facade::Aspect::new(0);
pub(crate) const HOST_MEMORY: u64 = 512 * 1024 * 1024;

pub(crate) type HostTicket = PreparedExecutionMap<'static, u8, u8, u8, ()>;

pub(crate) fn authority() -> &'static ExecutionAuthority {
    static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
    AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(HOST_MEMORY),
        })
        .expect("this test process's one authority")
    })
}

pub(crate) fn request(workers: usize, work: u64) -> LeaseRequest {
    LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::new(workers).unwrap(),
                Some(32 * 1024 * 1024),
                Some(work),
            ),
        ),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    }
}

pub(crate) fn competing_ticket(scratch_bytes: u64) -> Result<HostTicket, LeaseDenial> {
    let mut host_request = request(1, 1_000_000);
    host_request.policy = worth_foundational::ExecutionRequestPolicy::new(
        host_request.policy.posture(),
        host_request.policy.determinism(),
        worth_foundational::ExecutionBudget::new(
            host_request.policy.max_workers(),
            Some(HOST_MEMORY),
            Some(1_000_000),
        ),
    );
    let lease = authority().request_lease(host_request)?;
    let identity = worth_foundational::PartitionIdentity::new(1);
    let map = worth_execution::ExecutionMap::try_from_declared_partitions(
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

pub(crate) fn hold_remaining_host_memory(
    release: std::sync::mpsc::Receiver<()>,
    ready: std::sync::mpsc::Sender<bool>,
) {
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

pub(crate) fn subscribed_graph(maximum: u64) -> (SignalGraph, NodeId) {
    let mut graph = worth_signal::facade::SignalGraph::new();
    let producer = graph
        .node()
        .with_contract(
            worth_signal::facade::adapters::NodeContract::wildcard()
                .with_produces(VALUE)
                .with_bounded_inputs(worth_signal::facade::BoundedSignalInputs::default())
                .with_max_checked_result_heap_bytes(maximum),
        )
        .build();
    let consumer = graph
        .node()
        .with_contract(
            worth_signal::facade::adapters::NodeContract::wildcard().with_bounded_inputs(
                worth_signal::facade::BoundedSignalInputs::new([
                    worth_signal::facade::DeclaredSignalInput::scoped(
                        producer,
                        VALUE,
                        worth_signal::facade::PartitionSubscription::exact(
                            worth_signal::facade::ScopePath::one("desk").unwrap(),
                        ),
                    ),
                ]),
            ),
        )
        .build();
    graph
        .set_dependencies(
            consumer,
            [worth_signal::facade::DependencyEdge::with_partition_scope(
                producer,
                VALUE,
                worth_signal::facade::PartitionSubscription::exact(
                    worth_signal::facade::ScopePath::one("desk").unwrap(),
                ),
            )],
        )
        .unwrap();
    assert_eq!(graph.subscribers_of(producer).unwrap(), &[consumer]);
    (graph, producer)
}

pub(crate) fn saturate_host() -> (std::sync::mpsc::Sender<()>, JoinHandle<()>) {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let thread = std::thread::spawn(move || hold_remaining_host_memory(release_rx, ready_tx));
    assert!(
        ready_rx
            .recv()
            .expect("host reservation arrives before callback returns"),
        "the competitor must exhaust host memory for any later admission"
    );
    (release_tx, thread)
}

pub(crate) fn assert_declared_zero_violation(error: &SignalError) {
    match error {
        worth_signal::facade::SignalError::CheckedResultCapacityExceeded { required, declared } => {
            assert!(*required > 0);
            assert_eq!(*declared, 0);
        }
        worth_signal::facade::SignalError::ExecutionStopped(stop) => {
            assert_eq!(
                stop.disposition(),
                worth_signal::facade::SignalPublicationDisposition::WorkerLocal
            );
            assert_eq!(stop.publication_progress().completed_tasks(), 0);
            let worth_signal::facade::SignalExecutionStopReason::Failure {
                cause: worth_signal::facade::SignalExecutionFailure::Domain(nested),
                ..
            } = stop.reason()
            else {
                panic!(
                    "expected a typed callback declaration failure: {:?}",
                    stop.reason()
                );
            };
            assert_declared_zero_violation(nested);
        }
        _ => panic!("expected CheckedResultCapacityExceeded, got {error:?}"),
    }
}
