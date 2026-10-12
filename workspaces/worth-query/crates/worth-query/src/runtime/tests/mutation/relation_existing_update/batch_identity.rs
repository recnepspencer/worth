use super::*;

#[test]
fn batch_relation_updates_preserve_identity_binding_aggregate_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_relation_runtime(request_execution)
        .workspace("tasks.batch-existing-relation-update")
        .expect("workspace should open");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.batch-existing-relation-update-table",
            |q| {
                q.from("TaskRelation")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("kind", "value")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("status", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("kind", "value")
                            .unwrap(),
                    )
                    .schema_basis("tasks-batch-existing-relation-update-table")
            },
        )
        .expect("relation live view should declare");

    let first = workspace
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
                .set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("open"),
                )
        })
        .expect("first seed should execute");
    let second = workspace
        .insert(request_execution, "TaskRelation", |relation| {
            relation
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("rel-2"),
                )
                .set_aspect(
                    test_aspect_touch("kind.value"),
                    test_authored_string_aspect_value("blocks"),
                )
                .set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("open"),
                )
        })
        .expect("second seed should execute");

    let first_binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:rel-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                first.deltas()[0].entity_identity.clone(),
            )
            .expect("first relation target should build")
            .in_target_collection("TaskRelation")
            .expect("first relation collection should build"),
        )
        .expect("first relation binding should build");
    let second_binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:rel-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                second.deltas()[0].entity_identity.clone(),
            )
            .expect("second relation target should build")
            .in_target_collection("TaskRelation")
            .expect("second relation collection should build"),
        )
        .expect("second relation binding should build");

    let receipt = workspace
        .batch(request_execution, |batch| {
            batch
                .update_existing(first_binding, |relation| {
                    relation.set_aspect(
                        test_aspect_touch("status.value"),
                        test_authored_string_aspect_value("closed"),
                    )
                })
                .update_existing(second_binding, |relation| {
                    relation.set_aspect(
                        test_aspect_touch("kind.value"),
                        test_authored_string_aspect_value("follows"),
                    )
                })
        })
        .expect("relation batch should execute");

    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .existing_truth_binding_count(),
        2
    );
    assert!(receipt
        .batch_mutation_evidence()
        .aggregate_existing_truth_binding_digest()
        .is_some());

    match workspace
        .inspect(request_execution, &receipt)
        .expect("batch should inspect")
    {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .existing_truth_binding_count(),
                2
            );
            assert_eq!(
                inspection.component_operations()[0]
                    .existing_truth_binding_evidence()
                    .expect("first relation update should retain binding evidence")
                    .family(),
                WorthQueryExistingTruthBindingFamily::DirectRelationIdentity
            );
            assert_eq!(
                inspection.component_operations()[1]
                    .existing_truth_binding_evidence()
                    .expect("second relation update should retain binding evidence")
                    .family(),
                WorthQueryExistingTruthBindingFamily::DirectRelationIdentity
            );
        }
        other => panic!("expected batch write receipt inspection, got {other:?}"),
    }
}
