use super::*;

#[test]
fn compose_graph_denies_entity_symbol_reuse_across_compositions_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.graph-composition-entity-leak")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-entity-leak-tasks",
            |q| {
                q.from("Task")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    )
                    .schema_basis("tasks-graph-composition-entity-leak-tasks")
            },
        )
        .expect("task live view should declare");
    let mut saved_entity = None;
    let _receipt = workspace
        .compose_graph(request_execution, |graph| {
            saved_entity = Some(graph.insert_entity("draft-task", "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-draft"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Draft task"),
                )
            })?);
            Ok(())
        })
        .expect("initial composition should execute");
    let saved_entity = saved_entity.expect("composition should expose a saved entity symbol");

    let error = workspace
        .compose_graph(request_execution, |graph| {
            graph.update_entity(&saved_entity, |task| {
                task.set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Leaked title"),
                )
            })?;
            Ok(())
        })
        .expect_err("entity symbol reuse across compositions should deny");

    match error {
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryGraphCompositionDenialKind::UnresolvedSymbolicReference
            );
            assert_eq!(
                denial.failure_stage(),
                WorthQueryGraphCompositionAdmissionTraceStage::LoweringValidated
            );
            assert_eq!(denial.symbol(), Some("draft-task"));
            assert_eq!(denial.target_collection(), Some("Task"));
            assert!(denial
                .message()
                .contains("same-batch symbolic target `draft-task` was not declared earlier"));
        }
        other => panic!("expected graph composition denial, got {other:?}"),
    }
}

#[test]
fn compose_graph_denies_symbolic_collection_mismatch_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_edge_runtime(request_execution)
        .workspace("tasks.graph-composition-collection-mismatch")
        .expect("runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-collection-mismatch-tasks",
            |q| {
                q.from("Task")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    )
                    .schema_basis("tasks-graph-composition-collection-mismatch-tasks")
            },
        )
        .expect("task live view should declare");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-collection-mismatch-edges",
            |q| {
                q.from("TaskEdge")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("edge", "kind")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts(
                            "edge",
                            "source_identity",
                        )
                        .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts(
                            "edge",
                            "target_identity",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("edge", "kind")
                            .unwrap(),
                    )
                    .schema_basis("tasks-graph-composition-collection-mismatch-edges")
            },
        )
        .expect("edge live view should declare");

    let error = workspace
        .compose_graph(request_execution, |graph| {
            let task = graph.insert_entity("draft-task", "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-draft"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Draft task"),
                )
            })?;
            let _ = graph.insert_symbolic_relation("draft-edge", "TaskEdge", |edge| {
                edge.set_aspect(
                    test_aspect_touch("edge.kind"),
                    test_authored_string_aspect_value("depends_on"),
                )
                .symbolic_entity_identity(test_aspect_touch("edge.source_identity"), &task)
                .existing_entity_identity(
                    test_aspect_touch("edge.target_identity"),
                    test_entity_identity("task-existing"),
                )
            })?;
            let leaked_entity = WorthQueryGraphEntitySymbol::new(
                WorthQuerySymbolicTargetReference::new("draft-edge")?
                    .in_target_collection("Task")?,
            );
            graph.update_entity(&leaked_entity, |entity| {
                entity.set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Should never lower"),
                )
            })?;
            Ok(())
        })
        .expect_err("collection-mismatched symbolic entity should deny");

    match error {
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryGraphCompositionDenialKind::SymbolicCollectionMismatch
            );
            assert_eq!(
                denial.failure_stage(),
                WorthQueryGraphCompositionAdmissionTraceStage::LoweringValidated
            );
            assert_eq!(denial.symbol(), Some("draft-edge"));
            assert_eq!(denial.target_collection(), Some("Task"));
            assert!(denial
                .message()
                .contains("resolved to collection `TaskEdge`, not `Task`"));
            assert_eq!(
                denial.admission_trace().stages(),
                &[
                    WorthQueryGraphCompositionAdmissionTraceStage::ProgramParsed,
                    WorthQueryGraphCompositionAdmissionTraceStage::SymbolsValidated,
                    WorthQueryGraphCompositionAdmissionTraceStage::LoweringValidated,
                    WorthQueryGraphCompositionAdmissionTraceStage::DeniedBeforeExecution,
                ]
            );
        }
        other => panic!("expected graph composition denial, got {other:?}"),
    }
}
