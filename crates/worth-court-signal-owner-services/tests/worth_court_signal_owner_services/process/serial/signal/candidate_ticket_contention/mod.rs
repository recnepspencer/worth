// D12 reserves checked-result custody and publication maps before callbacks.
// This ports master's post-callback refusal court (`2075ce36c1`): contention
// before admission refuses without effects; contention after admission cannot
// steal the epoch's custody; an oversized result violates its declared contract.
//
// The competitor fills the whole process memory, so this test owns its
// process and its authority: no other test can starve beside it.

#[allow(unused_imports)] use crate::candidate_ticket_contention::isolated_context::*;
use {std::num::NonZeroUsize};
use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};
use {std::sync::mpsc, std::sync::Mutex, std::sync::OnceLock};
use {std::thread::JoinHandle};

use {worth_execution::CancellationToken, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap, worth_execution::LeaseDenial, worth_execution::LeaseRequest, worth_execution::MapPartition, worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel, worth_execution::PreparedExecutionMap};
use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

use {worth_signal::facade::adapters::NodeContract};
use {worth_signal::facade::Aspect, worth_signal::facade::AspectVersion, worth_signal::facade::BoundedSignalInputs, worth_signal::facade::ChangedRegion, worth_signal::facade::DeclaredSignalInput, worth_signal::facade::DependencyEdge, worth_signal::facade::NodeEvaluationResult, worth_signal::facade::NodeId, worth_signal::facade::PartitionSubscription, worth_signal::facade::RunMode, worth_signal::facade::ScopePath, worth_signal::facade::SignalError, worth_signal::facade::SignalExecutionFailure, worth_signal::facade::SignalExecutionStopReason, worth_signal::facade::SignalGraph, worth_signal::facade::SignalLeaseDenial, worth_signal::facade::SignalPublicationDisposition};

#[test]
fn checked_epoch_contention_obeys_d12_admission_and_result_contract() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let (mut graph, producer) = subscribed_graph(4096);
    let before = graph.node_aspect_version(producer).unwrap().get(VALUE);
    let lease = authority().request_lease(request(2, 2_000_000)).unwrap();
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let blocker = Mutex::<Option<(std::sync::mpsc::Sender<()>, JoinHandle<()>)>>::new(None);
    let evaluate = |graph: &mut SignalGraph, mode| {
        graph.evaluate_checked(
            &[producer],
            mode,
            &(),
            &|_| {
                // Only the first evaluation meets a saturated host.
                if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                    *blocker.lock().unwrap() = Some(saturate_host());
                }
                Ok(
                    worth_signal::facade::NodeEvaluationResult::from_version(worth_signal::facade::AspectVersion::zero().with(VALUE, 7))
                        .with_changed_aspect_region(VALUE, worth_signal::facade::ChangedRegion::new("desk")),
                )
            },
            worth_execution::ExecutionRequest::leased(&lease),
        )
    };
    // Insufficient process room at admission must prevent callback effects.
    let (release, thread) = saturate_host();
    let refused = evaluate(&mut graph, worth_signal::facade::RunMode::Default);
    release.send(()).unwrap();
    thread.join().unwrap();
    let error = refused.expect_err("D12 admits custody before callback effects");
    let worth_signal::facade::SignalError::ExecutionStopped(stop) = error else {
        panic!("host contention is a typed execution stop: {error:?}");
    };
    let worth_signal::facade::SignalExecutionStopReason::Admission(worth_signal::facade::SignalLeaseDenial::MemoryExhausted(memory)) =
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
            worth_execution::MemoryLimitDenial::Capacity { requested, admitted, level: worth_execution::MemoryLimitLevel::Process }
                if requested > admitted
        ),
        "{memory:?}"
    );
    // Admission refused before dispatch; the callback-local assertion below
    // still protects the original court's WorkerLocal publication boundary.
    assert_eq!(stop.disposition(), worth_signal::facade::SignalPublicationDisposition::NoWork);
    assert_eq!(stop.publication_progress().completed_tasks(), 0);
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert_eq!(
        graph.node_aspect_version(producer).unwrap().get(VALUE),
        before
    );

    // After admission, a competitor cannot steal the retained result/map tickets.
    let contended = evaluate(&mut graph, worth_signal::facade::RunMode::Default);
    let (release, thread) = blocker.lock().unwrap().take().expect("callback ran once");
    release.send(()).unwrap();
    thread.join().unwrap();
    let report = contended.expect("D12 settles the original reserved custody");
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(report.tasks_executed, 1);
    assert_eq!(graph.node_aspect_version(producer).unwrap().get(VALUE), 7);

    // D12's successful publication is now reusable. Explicitly force the retry
    // so the original court's second-callback assertion still proves admission.
    let report = evaluate(&mut graph, worth_signal::facade::RunMode::ForceOnDemand)
        .expect("released host memory admits the epoch");
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(report.tasks_executed, 1);
    assert_eq!(graph.node_aspect_version(producer).unwrap().get(VALUE), 7);

    // Callback effects can precede a typed declaration violation, never a late
    // memory-capacity refusal. The invalid result still publishes nothing.
    let (mut bounded, producer) = subscribed_graph(0);
    let before = bounded.node_aspect_version(producer).unwrap().get(VALUE);
    let effects = std::sync::atomic::AtomicUsize::new(0);
    let error = bounded
        .evaluate_checked(
            &[producer],
            worth_signal::facade::RunMode::Default,
            &(),
            &|context| {
                assert_eq!(context.result_capacity_bytes(), Some(0));
                effects.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(
                    worth_signal::facade::NodeEvaluationResult::from_version(worth_signal::facade::AspectVersion::zero().with(VALUE, 7))
                        .with_label("exceeds the declared zero heap bound"),
                )
            },
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .unwrap_err();
    assert_declared_zero_violation(&error);
    let worth_signal::facade::SignalError::ExecutionStopped(stop) = error else {
        panic!("result declaration violation is a typed execution stop: {error:?}");
    };
    assert_eq!(
        stop.disposition(),
        worth_signal::facade::SignalPublicationDisposition::WorkerLocal
    );
    assert_eq!(stop.publication_progress().completed_tasks(), 0);
    assert_eq!(effects.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(
        bounded.node_aspect_version(producer).unwrap().get(VALUE),
        before
    );
}
