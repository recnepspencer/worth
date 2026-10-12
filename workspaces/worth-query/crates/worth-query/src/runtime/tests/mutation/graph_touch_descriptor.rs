use super::super::support::*;
use crate::intent_admission::{
    WorthQueryAdmittedIntentPlan, WorthQueryAuthoritativeMutationBatchExecutionPlan,
};

#[test]
fn graph_touch_descriptor_is_derived_from_real_graph_builder_and_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = stateful_bridge_task_edge_runtime(request_execution);
    let (commands, breadth, program) = mixed_graph_program();

    let review = runtime
        .review_authoritative_runtime_write_batch_with_graph_artifacts(commands, breadth, program)
        .expect("graph batch review should admit real builder artifacts");
    let plan = admitted_batch_plan(&review);
    let descriptor = plan
        .graph_touch_descriptor()
        .expect("real builder artifacts should derive graph touch descriptor");
    let handoff = runtime
        .resolve_reviewed_admitted_authoritative_write_batch_handoff(review)
        .expect("admitted graph batch should resolve to handoff");
    let handoff_descriptor = handoff
        .graph_touch_descriptor()
        .expect("handoff should preserve graph touch descriptor derivation");

    assert_eq!(
        descriptor.descriptor_digest(),
        handoff_descriptor.descriptor_digest()
    );
    assert_eq!(descriptor.component_count(), 4);
    assert_eq!(descriptor.insert_command_count(), 2);
    assert_eq!(descriptor.update_command_count(), 1);
    assert_eq!(descriptor.delete_command_count(), 1);
    assert_eq!(descriptor.declared_collection_count(), 2);
    assert_eq!(descriptor.declared_aspect_touch_count(), 5);
    assert_eq!(descriptor.touched_aspect_count(), 3);
    assert!(descriptor.touches_target_collection(&target_collection("Task")));
    assert!(descriptor.touches_target_collection(&target_collection("TaskEdge")));
    assert!(descriptor.touches_aspect(&test_aspect_touch("edge.kind")));
    assert!(descriptor.touches_aspect(&test_aspect_touch("edge.source_identity")));
    assert!(descriptor.touches_aspect(&test_aspect_touch("edge.target_identity")));
    assert_eq!(
        descriptor.rows()[3].lifecycle_family(),
        Some(WorthQueryGraphTouchLifecycleFamily::SameBatchSymbolicRelationRetirement)
    );
}

#[test]
fn graph_touch_descriptor_replay_is_stable_for_equivalent_real_graph_programs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_edge_runtime(request_execution);
    let first = review_descriptor(&mut runtime, mixed_graph_program());
    let second = review_descriptor(&mut runtime, mixed_graph_program());

    assert_eq!(first.descriptor_digest(), second.descriptor_digest());
}

#[test]
fn ordinary_batch_reuses_touch_vocabulary_without_graph_lifecycle_overclaim() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = stateful_bridge_task_runtime(request_execution);
    let command = WorthQueryWriteCommand::InsertAspects {
        collection: crate::runtime::WorthQueryMutationTargetCollectionIdentity::new(
            "write-command-declared",
            "Task",
        ),
        aspects: vec![
            WorthQueryAuthoredAspectMutation::new(
                test_aspect_touch("identity.id"),
                test_string_aspect_value("task-ordinary"),
            )
            .unwrap(),
            WorthQueryAuthoredAspectMutation::new(
                test_aspect_touch("title.value"),
                test_string_aspect_value("Ordinary task"),
            )
            .unwrap(),
        ],
        symbolic_aspect_references: Vec::new(),
        metadata: WorthQueryMutationMetadata::new(),
        naming_intent: None,
        continuity_intent: None,
        symbolic_target_reference: None,
    };
    let review = runtime
        .review_authoritative_runtime_write_batch(vec![command])
        .expect("ordinary batch review should admit");
    let plan = admitted_batch_plan(&review);
    let descriptor = plan
        .graph_touch_descriptor()
        .expect("ordinary batch should still derive shared touch vocabulary");

    assert_eq!(descriptor.component_count(), 0);
    assert_eq!(descriptor.rows().len(), 1);
    assert_eq!(descriptor.insert_command_count(), 1);
    assert_eq!(descriptor.update_command_count(), 0);
    assert_eq!(descriptor.delete_command_count(), 0);
    assert_eq!(descriptor.rows()[0].program_step_kind(), None);
    assert_eq!(descriptor.rows()[0].lifecycle_family(), None);
    assert!(descriptor.touches_target_collection(&target_collection("Task")));
    assert!(descriptor.touches_aspect(&test_aspect_touch("identity.id")));
    assert!(descriptor.touches_aspect(&test_aspect_touch("title.value")));
}

fn review_descriptor(
    runtime: &mut WorthQueryRuntime,
    graph_program: (
        Vec<WorthQueryWriteCommand>,
        WorthQueryGraphCompositionBreadth,
        WorthQueryGraphCompositionProgram,
    ),
) -> WorthQueryGraphTouchDescriptor {
    let (commands, breadth, program) = graph_program;
    let review = runtime
        .review_authoritative_runtime_write_batch_with_graph_artifacts(commands, breadth, program)
        .expect("graph batch review should admit");
    admitted_batch_plan(&review)
        .graph_touch_descriptor()
        .expect("real graph program should derive descriptor")
}

fn admitted_batch_plan(
    review: &crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData,
) -> WorthQueryAuthoritativeMutationBatchExecutionPlan {
    match review.admitted_plan() {
        Some(WorthQueryAdmittedIntentPlan::AuthoritativeMutationBatch(plan)) => plan.clone(),
        _ => panic!("review should produce authoritative mutation batch plan"),
    }
}

fn target_collection(value: &str) -> WorthQueryMutationTargetCollectionIdentity {
    WorthQueryMutationTargetCollectionIdentity::new("graph-touch-descriptor-test", value)
}

fn mixed_graph_program() -> (
    Vec<WorthQueryWriteCommand>,
    WorthQueryGraphCompositionBreadth,
    WorthQueryGraphCompositionProgram,
) {
    let mut graph = WorthQueryGraphCompositionBuilder::new();
    let task = graph
        .insert_entity("task", "Task", |entity| {
            entity
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-touch"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Draft"),
                )
        })
        .unwrap();
    let edge = graph
        .insert_symbolic_relation("edge", "TaskEdge", |relation| {
            relation
                .set_aspect(
                    test_aspect_touch("edge.kind"),
                    test_authored_string_aspect_value("depends_on"),
                )
                .symbolic_entity_identity(test_aspect_touch("edge.source_identity"), &task)
                .existing_entity_identity(
                    test_aspect_touch("edge.target_identity"),
                    test_entity_identity("task-existing"),
                )
        })
        .unwrap();
    graph
        .update_entity(&task, |entity| {
            entity.set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Published"),
            )
        })
        .unwrap();
    graph
        .delete_relation(&edge, |delete| {
            delete.touches(test_aspect_touches([
                "edge.kind",
                "edge.source_identity",
                "edge.target_identity",
            ]))
        })
        .unwrap();
    graph.finish().unwrap()
}
