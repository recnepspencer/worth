//! workspace insert uses aspect native authoring and routes live delivery.
use super::*;

#[test]
fn workspace_insert_uses_aspect_native_authoring_and_routes_live_delivery() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.aspect-insert")
        .expect("task runtime should open a named workspace");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.aspect-table", |q| {
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
                .schema_basis("tasks-aspect-table")
        })
        .expect("live view should declare");

    let receipt = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Buy milk"),
            )
        })
        .expect("aspect-native insert should execute");
    let patches = workspace.observe(&live);
    let state = workspace
        .state(request_execution, &receipt)
        .expect("write receipt should expose state posture");

    assert_eq!(receipt.deltas().len(), 1);
    assert_eq!(receipt.mutation_family(), WorthQueryMutationFamily::Insert);
    assert_eq!(
        receipt.basis_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        receipt.terminal_declared_collection_projection(),
        Some("Task")
    );
    assert_eq!(receipt.declared_entity_identity(), None);
    assert_eq!(
        receipt.terminal_target_collection_projection(),
        Some("Task")
    );
    assert_eq!(
        receipt.target_evidence().declared().target_class(),
        WorthQueryMutationTargetClass::Collection
    );
    assert_eq!(
        receipt
            .target_evidence()
            .declared()
            .collection()
            .map(|collection| collection.as_str()),
        Some("Task")
    );
    assert_eq!(
        receipt.target_evidence().resolved().target_class(),
        WorthQueryMutationTargetClass::Entity
    );
    assert_eq!(
        receipt
            .target_evidence()
            .resolved()
            .collection()
            .map(|collection| collection.as_str()),
        Some("Task")
    );
    assert!(receipt
        .target_evidence()
        .resolved()
        .entity_identity()
        .is_some());
    let causality = receipt
        .causality_evidence()
        .expect("authoritative insert should retain bridge causality");
    let provenance = receipt
        .provenance_evidence()
        .expect("authoritative insert should retain bridge provenance");
    assert!(!causality.causality_digest().is_empty());
    assert!(!causality.truth_trigger_digest().is_empty());
    assert!(!provenance.execution_record_digest().is_empty());
    assert!(!provenance.feedback_provenance_digest().is_empty());
    assert!(provenance.authoritative_artifact_digest().is_some());
    assert_eq!(
        receipt.deltas()[0].admitted_touched_aspects(),
        test_aspect_touches(["identity", "title"]).as_slice()
    );
    assert_eq!(
        receipt
            .declared_aspect_operations()
            .iter()
            .map(|operation| {
                format!(
                    "{}:{}",
                    operation.kind(),
                    operation.aspect_touch().admitted_touch_digest_part()
                )
            })
            .collect::<Vec<_>>(),
        vec!["set:identity:id".to_string(), "set:title:value".to_string()]
    );
    assert_eq!(
        receipt.terminal_affected_live_view_ids_projection(),
        &["tasks.aspect-table".to_string()]
    );
    assert_eq!(
        receipt.authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(patches.query_delivery_batches.len(), 1);
    assert_eq!(
        patches.query_delivery_batches[0].patch_group_kind(),
        QueryPatchGroupKind::CollectionMembershipPatchGroup
    );
    assert_eq!(state.kind(), WorthQueryRuntimeStateKind::Ready);
    assert_eq!(
        state.authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
}
