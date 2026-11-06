use std::sync::atomic::{AtomicU64, Ordering};
use worth_signal::facade::{Aspect, AspectVersion, DependencyEdge, EvaluationContext, SignalError};
use worth_signal::facade::{SignalGraph, SignalRuntime};

#[test]
fn bulk_reads_settle_dirty_ancestors_before_returning_clean_downstream_targets() {
    for (transactional, guided) in [(false, false), (false, true), (true, false), (true, true)] {
        let aspect = Aspect::new(0);
        let mut graph = SignalGraph::new();
        let source = graph.node().build();
        let middle = graph.node().on_demand().build();
        let last = graph.node().on_demand().build();
        graph
            .set_dependencies(middle, [DependencyEdge::new(source, aspect)])
            .unwrap();
        graph
            .set_dependencies(last, [DependencyEdge::new(middle, aspect)])
            .unwrap();
        let mut runtime = SignalRuntime::build_for::<AtomicU64>(graph);
        let mut input = AtomicU64::new(2);
        let evaluate =
            |view: &mut EvaluationContext<'_, AtomicU64>| -> Result<AspectVersion, SignalError> {
                let value = if view.node() == source {
                    view.domain().load(Ordering::Relaxed)
                } else {
                    let parent = if view.node() == middle {
                        source
                    } else {
                        middle
                    };
                    view.read_aspect_version(parent, aspect)?.get(aspect) + 1
                };
                Ok(AspectVersion::from_updates([(aspect, value)]))
            };
        runtime.read(last, &input, &evaluate).unwrap();
        assert_eq!(
            runtime
                .graph()
                .node_aspect_version(last)
                .unwrap()
                .get(aspect),
            4
        );
        input.store(8, Ordering::Relaxed);
        let targets = [last, source, middle];
        let versions = if transactional {
            let basis = runtime
                .observe_signal_branch_basis(runtime.current_branch())
                .unwrap();
            let serial = worth_execution::SerialRequest::from_memory(
                worth_execution::SerialMemoryBudget::new(None),
                worth_execution::CancellationToken::new(),
                None,
            );
            let mut observed = None;
            runtime
                .advance_signal_branch(
                    worth_execution::ExecutionRequest::serial(&serial),
                    &mut input,
                    &basis,
                    |tx| {
                        tx.mark_changed(source, aspect)?;
                        tx.read(source, &evaluate)?;
                        observed = Some(if guided {
                            tx.targets(targets.to_vec())
                                .on_demand()
                                .read_many(&evaluate)?
                        } else {
                            tx.read_many(&targets, &evaluate)?
                        });
                        Ok(())
                    },
                )
                .unwrap();
            observed.unwrap()
        } else {
            worth_signal::facade::core::mark_dirty(runtime.graph_mut(), source, aspect).unwrap();
            runtime.read(source, &input, &evaluate).unwrap();
            if guided {
                runtime
                    .targets(targets.to_vec())
                    .on_demand()
                    .read_many(&input, &evaluate)
                    .unwrap()
            } else {
                runtime.read_many(&targets, &input, &evaluate).unwrap()
            }
        };
        assert_eq!(
            versions
                .into_iter()
                .map(|version| version.get(aspect))
                .collect::<Vec<_>>(),
            vec![10, 8, 9],
            "transactional={transactional}, guided={guided}"
        );
    }
}
