use super::*;

#[test]
fn compose_graph_denies_empty_composition_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.graph-composition-empty")
        .expect("task runtime should open a named workspace");

    let error = workspace
        .compose_graph(request_execution, |_graph| Ok(()))
        .expect_err("empty graph compositions should deny");

    match error {
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryGraphCompositionDenialKind::EmptyComposition
            );
            assert_eq!(
                denial.failure_stage(),
                WorthQueryGraphCompositionAdmissionTraceStage::ProgramParsed
            );
            assert_eq!(denial.symbol(), None);
            assert!(denial.message().contains("at least one operation"));
            assert_eq!(
                denial.admission_trace().stages(),
                &[
                    WorthQueryGraphCompositionAdmissionTraceStage::ProgramParsed,
                    WorthQueryGraphCompositionAdmissionTraceStage::DeniedBeforeExecution,
                ]
            );
            assert!(!denial.admission_trace().admission_trace_digest().is_empty());
        }
        other => panic!("expected graph composition denial, got {other:?}"),
    }
}

#[test]
fn compose_graph_denies_duplicate_symbol_declarations_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.graph-composition-duplicate")
        .expect("task runtime should open a named workspace");

    let error = workspace
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
        .expect_err("duplicate graph symbols should deny");

    match error {
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryGraphCompositionDenialKind::DuplicateSymbolDeclaration
            );
            assert_eq!(
                denial.failure_stage(),
                WorthQueryGraphCompositionAdmissionTraceStage::SymbolsValidated
            );
            assert_eq!(denial.symbol(), Some("draft-task"));
            assert_eq!(denial.target_collection(), Some("Task"));
            assert!(denial.message().contains("declared more than once"));
            assert_eq!(
                denial.admission_trace().stages(),
                &[
                    WorthQueryGraphCompositionAdmissionTraceStage::ProgramParsed,
                    WorthQueryGraphCompositionAdmissionTraceStage::SymbolsValidated,
                    WorthQueryGraphCompositionAdmissionTraceStage::DeniedBeforeExecution,
                ]
            );
        }
        other => panic!("expected graph composition denial, got {other:?}"),
    }
}

#[test]
fn compose_graph_denies_relation_symbol_reuse_across_compositions_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_edge_runtime(request_execution)
        .workspace("tasks.graph-composition-relation-leak")
        .expect("runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-relation-leak-tasks",
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
                    .schema_basis("tasks-graph-composition-relation-leak-tasks")
            },
        )
        .expect("task live view should declare");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-relation-leak-edges",
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
                    .schema_basis("tasks-graph-composition-relation-leak-edges")
            },
        )
        .expect("edge live view should declare");
    let mut saved_relation = None;
    let _receipt = workspace
        .compose_graph(request_execution, |graph| {
            let draft = graph.insert_entity("draft-task", "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-draft"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Draft task"),
                )
            })?;
            saved_relation = Some(graph.insert_symbolic_relation(
                "draft-edge",
                "TaskEdge",
                |relation| {
                    relation
                        .set_aspect(
                            test_aspect_touch("edge.kind"),
                            test_authored_string_aspect_value("depends_on"),
                        )
                        .symbolic_entity_identity(test_aspect_touch("edge.source_identity"), &draft)
                        .existing_entity_identity(
                            test_aspect_touch("edge.target_identity"),
                            test_entity_identity("task-existing"),
                        )
                },
            )?);
            Ok(())
        })
        .expect("initial composition should execute");
    let saved_relation = saved_relation.expect("composition should expose a saved relation symbol");

    let error = workspace
        .compose_graph(request_execution, |graph| {
            graph.update_relation(&saved_relation, |relation| {
                relation
                    .set_aspect(
                        test_aspect_touch("edge.kind"),
                        test_authored_string_aspect_value("blocks"),
                    )
                    .existing_entity_identity(
                        test_aspect_touch("edge.target_identity"),
                        test_entity_identity("task-second-existing"),
                    )
            })?;
            Ok(())
        })
        .expect_err("relation symbol reuse across compositions should deny");

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
            assert_eq!(denial.symbol(), Some("draft-edge"));
            assert_eq!(denial.target_collection(), Some("TaskEdge"));
            assert!(denial
                .message()
                .contains("same-batch symbolic target `draft-edge` was not declared earlier"));
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
