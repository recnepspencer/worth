use worth_signal::facade::*;

const SOURCE_TEXT: Aspect = Aspect::new(0);
const SYMBOLS: Aspect = Aspect::new(1);
const DIAGNOSTICS: Aspect = Aspect::new(2);
const BUNDLE: Aspect = Aspect::new(3);

#[derive(Default)]
struct BuildState {
    source_version: u64,
    symbols_version: u64,
    diagnostics_version: u64,
    bundle_version: u64,
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

    // One source file changes.
    // We want the symbol index, diagnostics panel, and one build target to update.
    // We also want to be able to ask the runtime why the bundle reran.
    let mut graph = SignalGraph::new();
    let source_file = graph.node().build();
    let symbol_index = graph.node().on_demand().build();
    let diagnostics_panel = graph.node().on_demand().build();
    let app_bundle = graph.node().on_demand().build();

    graph.set_dependencies(
        symbol_index,
        [DependencyEdge::new(source_file, SOURCE_TEXT)],
    )?;
    graph.set_dependencies(
        diagnostics_panel,
        [
            DependencyEdge::new(source_file, SOURCE_TEXT),
            DependencyEdge::new(symbol_index, SYMBOLS),
        ],
    )?;
    graph.set_dependencies(
        app_bundle,
        [
            DependencyEdge::new(source_file, SOURCE_TEXT),
            DependencyEdge::new(symbol_index, SYMBOLS),
        ],
    )?;

    let mut runtime = SignalRuntime::build_for::<BuildState>(graph);
    let mut state = BuildState {
        source_version: 1,
        symbols_version: 10,
        diagnostics_version: 20,
        bundle_version: 30,
    };

    let evaluate = |view: &mut EvaluationContext<'_, BuildState>| {
        let result = if view.node() == source_file {
            view.finish(NodeEvaluationResult::from_version(
                AspectVersion::from_updates([(SOURCE_TEXT, view.domain().source_version)]),
            ))
        } else if view.node() == symbol_index {
            let _text = view.read_aspect_version(source_file, SOURCE_TEXT)?;
            view.finish(NodeEvaluationResult::from_version(
                AspectVersion::from_updates([(SYMBOLS, view.domain().symbols_version)]),
            ))
        } else if view.node() == diagnostics_panel {
            let _text = view.read_aspect_version(source_file, SOURCE_TEXT)?;
            let _symbols = view.read_aspect_version(symbol_index, SYMBOLS)?;
            view.finish(NodeEvaluationResult::from_version(
                AspectVersion::from_updates([(DIAGNOSTICS, view.domain().diagnostics_version)]),
            ))
        } else {
            let _text = view.read_aspect_version(source_file, SOURCE_TEXT)?;
            let _symbols = view.read_aspect_version(symbol_index, SYMBOLS)?;
            view.finish(NodeEvaluationResult::from_version(
                AspectVersion::from_updates([(BUNDLE, view.domain().bundle_version)]),
            ))
        };
        Ok::<_, SignalError>(result)
    };

    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the live branch should admit a basis");
    let basis = runtime
        .advance_signal_branch(request_execution, &mut state, &basis, |tx| {
            tx.read_many(
                &[source_file, symbol_index, diagnostics_panel, app_bundle],
                &evaluate,
            )?;
            Ok(())
        })
        .expect("the initial build should advance the branch")
        .into_basis();

    let (snapshot, basis) = runtime
        .capture_signal_branch_snapshot(&basis)
        .expect("snapshot capture should use the exact live basis")
        .into_parts();

    state.source_version += 1;
    state.symbols_version += 1;
    state.diagnostics_version += 1;
    state.bundle_version += 1;

    let _basis = runtime
        .advance_signal_branch(request_execution, &mut state, &basis, |tx| {
            tx.mark_changed(source_file, SOURCE_TEXT)?;
            tx.read_many(&[diagnostics_panel, app_bundle], &evaluate)?;
            Ok(())
        })
        .expect("the source edit should advance the branch")
        .into_basis();

    let versions = runtime.read_many(&[diagnostics_panel, app_bundle], &state, &evaluate)?;
    assert_eq!(versions[0].get(DIAGNOSTICS), 21);
    assert_eq!(versions[1].get(BUNDLE), 31);

    let explanation = runtime.diagnostics().why(app_bundle)?;
    let rendered = format!("{explanation}");
    assert!(
        rendered.contains("source")
            || rendered.contains("upstream")
            || rendered.contains("Changed"),
        "bundle explanation should say why the target reran"
    );

    let replay = {
        let history = runtime.history();
        let branch = history.current_branch();
        history.replay_for_branch(branch.id)
    };
    assert!(
        !replay.frames.is_empty(),
        "branch replay should show the build session instead of losing the trail"
    );
    assert!(
        replay
            .frames
            .iter()
            .any(|frame| frame.snapshot_id == Some(snapshot.snapshot().meta.snapshot_id)),
        "replay should keep the saved snapshot in view"
    );

    Ok(())
}
