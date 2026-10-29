// The checked door refuses before any callback or graph publication.
#[allow(unused_imports)]
pub(crate) use crate::fast::request_differential::request_custody::*;

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
            CancellationToken::new(),
            None,
        );
        let calls = AtomicUsize::new(0);
        let before = graph.node_aspect_version(targets[0]).unwrap();
        let outcome = graph.evaluate_checked(
            &targets,
            RunMode::Default,
            &(),
            &|_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(AspectVersion::zero().with(worth_signal::facade::Aspect::new(0), 7))
            },
            ExecutionRequest::serial(&serial),
        );
        match outcome {
            Ok(report) => {
                completed = true;
                assert_eq!(report.tasks_executed, 1);
                assert_eq!(calls.load(Ordering::SeqCst), 1);
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
                    calls.load(Ordering::SeqCst),
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
