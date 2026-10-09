use worth_signal::facade::*;

const DOCUMENT: Aspect = Aspect::new(0);
const SUMMARY_INDEX: Aspect = Aspect::new(1);
const APPENDIX_INDEX: Aspect = Aspect::new(2);

#[derive(Default)]
struct DocumentState {
    document_version: u64,
    summary_index_version: u64,
    appendix_index_version: u64,
    changed_section: String,
}

fn summary_scope() -> PartitionSubscription {
    PartitionSubscription::whole_partition("summary")
}

fn appendix_scope() -> PartitionSubscription {
    PartitionSubscription::whole_partition("appendix")
}

fn main() -> Result<(), SignalError> {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    // One paragraph in the summary section changes.
    // We want the summary index to refresh and the appendix index to stay put.
    // This is the kind of job where region-aware invalidation pays for itself.
    let mut graph = SignalGraph::new();
    let document = graph.node().partitioned_output().build();
    let summary_index = graph.node().on_demand().build();
    let appendix_index = graph.node().on_demand().build();

    graph.set_dependencies(
        summary_index,
        [DependencyEdge::with_partition_scope(
            document,
            DOCUMENT,
            summary_scope(),
        )],
    )?;
    graph.set_dependencies(
        appendix_index,
        [DependencyEdge::with_partition_scope(
            document,
            DOCUMENT,
            appendix_scope(),
        )],
    )?;

    let mut runtime = SignalRuntime::build_for::<DocumentState>(graph);
    let mut state = DocumentState {
        document_version: 1,
        summary_index_version: 100,
        appendix_index_version: 200,
        changed_section: String::new(),
    };

    let evaluate = |view: &mut EvaluationContext<'_, DocumentState>| {
        let result = if view.node() == document {
            let mut result = NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                DOCUMENT,
                view.domain().document_version,
            )]));
            if !view.domain().changed_section.is_empty() {
                result = result.with_changed_region(ChangedRegion::new(
                    view.domain().changed_section.as_str(),
                ));
            }
            view.finish(result)
        } else if view.node() == summary_index {
            // Read through the partition scope. A plain `read_aspect_version`
            // records an unscoped dependency, and every section edit would
            // then reach this index.
            let _summary =
                view.read_partitioned_aspect_version(document, DOCUMENT, summary_scope())?;
            view.finish(NodeEvaluationResult::from_version(
                AspectVersion::from_updates([(SUMMARY_INDEX, view.domain().summary_index_version)]),
            ))
        } else {
            let _appendix =
                view.read_partitioned_aspect_version(document, DOCUMENT, appendix_scope())?;
            view.finish(NodeEvaluationResult::from_version(
                AspectVersion::from_updates([(
                    APPENDIX_INDEX,
                    view.domain().appendix_index_version,
                )]),
            ))
        };
        Ok::<_, SignalError>(result)
    };

    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the live branch should admit a basis");
    let basis = runtime
        .advance_signal_branch(request_execution, &mut state, &basis, |tx| {
            tx.read_many(&[document, summary_index, appendix_index], &evaluate)?;
            Ok(())
        })
        .expect("the initial index build should advance the branch")
        .into_basis();

    state.document_version += 1;
    state.summary_index_version += 1;
    state.changed_section = "summary".to_string();

    let _basis = runtime
        .advance_signal_branch(request_execution, &mut state, &basis, |tx| {
            tx.mark_changed_with_regions(document, DOCUMENT, &[ChangedRegion::new("summary")])?;
            tx.read_many(&[document, summary_index, appendix_index], &evaluate)?;
            Ok(())
        })
        .expect("the summary edit should advance the branch")
        .into_basis();
    state.changed_section.clear();

    let versions = runtime.read_many(&[summary_index, appendix_index], &state, &evaluate)?;
    assert_eq!(versions[0].get(SUMMARY_INDEX), 101);
    assert_eq!(versions[1].get(APPENDIX_INDEX), 200);

    let explanation = runtime.diagnostics().explain(summary_index)?;
    let rendered = format!("{explanation}");
    assert!(
        rendered.contains("summary"),
        "the explanation should keep the changed region visible"
    );

    let appendix_explanation = format!("{}", runtime.diagnostics().explain(appendix_index)?);
    assert!(
        appendix_explanation.contains("ScopeUntouched"),
        "the appendix index should report that the summary edit left its section untouched"
    );

    Ok(())
}
