// The checked door refuses before any callback or graph publication.
#[allow(unused_imports)]
pub(crate) use {
    crate::request_differential::ordinary_context::graph, std::sync::atomic::AtomicUsize,
    std::sync::atomic::Ordering, worth_execution::CancellationToken,
    worth_execution::ExecutionRequest, worth_signal::facade::AspectVersion,
    worth_signal::facade::RunMode,
};

#[test]
fn singleton_memory_refusal_never_runs_the_callback() {
    // Sweep physical admission boundaries, including diagnostic publication.
    // Each attempt owns its graph and memory, so an earlier success cannot
    // turn a later attempt into a cache hit.
    let mut refused = false;
    let mut completed = false;
    for bytes in [16 << 10, 64 << 10, 256 << 10, 1 << 20, 8 << 20, 32 << 20] {
        let (mut graph, targets) = graph(1);
        let memory = worth_execution::SerialMemoryBudget::new(Some(bytes));
        let serial = worth_execution::SerialRequest::from_memory(
            memory.clone(),
            worth_execution::CancellationToken::new(),
            None,
        );
        let calls = std::sync::atomic::AtomicUsize::new(0);
        let before = graph.node_aspect_version(targets[0]).unwrap();
        let outcome = graph.evaluate_checked(
            &targets,
            worth_signal::facade::RunMode::Default,
            &(),
            &|_| {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(worth_signal::facade::AspectVersion::zero()
                    .with(worth_signal::facade::Aspect::new(0), 7))
            },
            worth_execution::ExecutionRequest::serial(&serial),
        );
        match outcome {
            Ok(report) => {
                completed = true;
                assert_eq!(report.tasks_executed, 1);
                assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
                assert_eq!(
                    graph
                        .node_aspect_version(targets[0])
                        .unwrap()
                        .get(worth_signal::facade::Aspect::new(0)),
                    7
                );
            }
            Err(error) => {
                refused = true;
                assert_eq!(
                    calls.load(std::sync::atomic::Ordering::SeqCst),
                    0,
                    "{bytes}-byte admission refused after callback: {error:?}"
                );
                assert_eq!(graph.node_aspect_version(targets[0]).unwrap(), before);
            }
        }
        assert_eq!(memory.charged_memory_bytes(), 0);
    }
    assert!(refused && completed, "the sweep must cross real admission");
}
