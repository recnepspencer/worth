use super::*;

#[test]
fn delete_existing_relation_preserves_relation_binding_family() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = stateful_bridge_task_relation_runtime(request_execution);
    let mut workspace = runtime
        .workspace("tasks.delete-existing-relation")
        .expect("workspace should open");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.relation-table", |q| {
            q.from("TaskRelation")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("kind", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("kind", "value")
                        .unwrap(),
                )
                .schema_basis("tasks-relation-table")
        })
        .expect("relation live view should declare");

    let seed = workspace
        .insert(request_execution, "TaskRelation", |relation| {
            relation
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("rel-1"),
                )
                .set_aspect(
                    test_aspect_touch("kind.value"),
                    test_authored_string_aspect_value("depends_on"),
                )
        })
        .expect("seed insert should execute");
    let binding_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1")
                .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity");
    let binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                binding_authority.clone(),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing relation target should build")
            .in_target_collection("TaskRelation")
            .expect("existing relation target collection should build"),
        )
        .expect("relation binding should build");

    let receipt = workspace
        .delete_existing(request_execution, binding)
        .expect("existing relation delete should execute");
    let evidence = receipt
        .existing_truth_binding_evidence()
        .expect("receipt should retain relation binding evidence");

    assert_eq!(
        evidence.family(),
        WorthQueryExistingTruthBindingFamily::DirectRelationIdentity
    );
    assert_eq!(
        evidence.authoritative_identity().as_str(),
        "authority:task-1"
    );
    assert_eq!(
        evidence.resolved_relation_identity(),
        &seed.deltas()[0].entity_identity
    );
    assert_eq!(
        evidence.resolved_target_identity(),
        &seed.deltas()[0].entity_identity
    );
    assert_eq!(
        evidence
            .target_collection()
            .map(|collection| collection.as_str()),
        Some("TaskRelation")
    );
}
