use super::*;

#[test]
fn graph_composition_denial_traces_distinguish_symbol_validation_from_lowering_failures() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_edge_runtime(request_execution)
        .workspace("tasks.graph-composition-denial-trace-classes")
        .expect("runtime should open a named workspace");

    let duplicate = workspace
        .compose_graph(request_execution, |graph| {
            let _ = graph.insert_entity("draft-task", "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-draft-one"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Draft one"),
                )
            })?;
            let _ = graph.insert_entity("draft-task", "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-draft-two"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Draft two"),
                )
            })?;
            Ok(())
        })
        .expect_err("duplicate symbol declaration should deny");

    let unresolved = workspace
        .compose_graph(request_execution, |graph| {
            let relation = WorthQueryGraphRelationSymbol::new(
                WorthQuerySymbolicTargetReference::new("leaked-edge")?
                    .in_target_collection("TaskEdge")?,
                None,
            );
            graph.update_relation(&relation, |edge| {
                edge.set_aspect(
                    test_aspect_touch("edge.kind"),
                    test_authored_string_aspect_value("blocks"),
                )
            })?;
            Ok(())
        })
        .expect_err("unresolved symbolic relation should deny");

    let duplicate = match duplicate {
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => denial,
        other => panic!("expected graph composition denial, got {other:?}"),
    };
    let unresolved = match unresolved {
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => denial,
        other => panic!("expected graph composition denial, got {other:?}"),
    };

    assert_eq!(
        duplicate.failure_stage(),
        WorthQueryGraphCompositionAdmissionTraceStage::SymbolsValidated
    );
    assert_eq!(
        unresolved.failure_stage(),
        WorthQueryGraphCompositionAdmissionTraceStage::LoweringValidated
    );
    assert_ne!(
        duplicate.admission_trace().admission_trace_digest(),
        unresolved.admission_trace().admission_trace_digest()
    );
}
