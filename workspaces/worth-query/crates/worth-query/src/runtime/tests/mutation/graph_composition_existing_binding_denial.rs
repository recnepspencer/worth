use super::super::support::*;

fn task_relation_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_task_relation_runtime(request_execution)
}

#[test]
fn compose_graph_denies_existing_target_collection_mismatch_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_relation_runtime(request_execution)
        .workspace("tasks.graph-composition-existing-target-collection-mismatch")
        .expect("runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-existing-target-collection-mismatch-relations",
            |q| {
                q.from("TaskRelation")
                    .select([
                        identity_id_field_key(),
                        kind_value_field_key(),
                        status_value_field_key(),
                    ])
                    .order_by(identity_id_field_key())
                    .schema_basis(
                        "tasks-graph-composition-existing-target-collection-mismatch-relations",
                    )
            },
        )
        .expect("relation live view should declare");
    let relation_seed = workspace
        .insert(request_execution, "TaskRelation", |relation| {
            relation
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("rel-update"),
                )
                .set_aspect(
                    test_aspect_touch("kind.value"),
                    test_authored_string_aspect_value("depends_on"),
                )
                .set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("open"),
                )
        })
        .expect("relation seed should execute");
    let binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:rel-update").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                relation_seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing relation target should build")
            .in_target_collection("Task")
            .expect("mismatched collection should still build as a declared binding"),
        )
        .expect("binding should build");

    let error = workspace
        .compose_graph(request_execution, |graph| {
            graph.update_existing(binding, |relation| {
                relation.set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("closed"),
                )
            })?;
            Ok(())
        })
        .expect_err("collection-mismatched existing target should deny");

    match error {
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryGraphCompositionDenialKind::ExistingTargetCollectionMismatch
            );
            assert_eq!(
                denial.failure_stage(),
                WorthQueryGraphCompositionAdmissionTraceStage::LoweringValidated
            );
            assert_eq!(denial.symbol(), None);
            assert_eq!(denial.target_collection(), Some("Task"));
            assert!(denial
                .message()
                .contains("belongs to collection `TaskRelation`, not `Task`"));
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

#[test]
fn compose_graph_denies_existing_target_missing_row_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_relation_runtime(request_execution)
        .workspace("tasks.graph-composition-existing-target-missing")
        .expect("runtime should open a named workspace");
    let binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:rel-update").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                test_entity_identity("TaskRelation:999"),
            )
            .expect("existing relation target should build")
            .in_target_collection("TaskRelation")
            .expect("target collection should build"),
        )
        .expect("binding should build");

    let error = workspace
        .compose_graph(request_execution, |graph| {
            graph.delete_existing(binding, |delete| {
                delete.touches(test_aspect_touches(["kind.value", "status.value"]))
            })?;
            Ok(())
        })
        .expect_err("missing existing target should deny");

    match error {
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryGraphCompositionDenialKind::ExistingTargetResolvedTargetMissing
            );
            assert_eq!(
                denial.failure_stage(),
                WorthQueryGraphCompositionAdmissionTraceStage::LoweringValidated
            );
            assert_eq!(denial.symbol(), None);
            assert_eq!(denial.target_collection(), Some("TaskRelation"));
            assert!(denial.message().contains("resolved target"));
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
