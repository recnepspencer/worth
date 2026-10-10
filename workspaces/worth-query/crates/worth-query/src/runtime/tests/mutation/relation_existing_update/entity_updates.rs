use super::*;

#[test]
fn update_existing_relation_preserves_identity_binding_and_receipt_target() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_relation_runtime(request_execution)
        .workspace("tasks.update-existing-relation")
        .expect("workspace should open");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.update-existing-relation-table",
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
                    .schema_basis("tasks-update-existing-relation-table")
            },
        )
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
                .set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("open"),
                )
        })
        .expect("seed insert should execute");
    let binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:rel-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing relation target should build")
            .in_target_collection("TaskRelation")
            .expect("existing relation target collection should build"),
        )
        .expect("relation binding should build");

    let receipt = workspace
        .update_existing(request_execution, binding, |relation| {
            relation.set_aspect(
                test_aspect_touch("kind.value"),
                test_authored_string_aspect_value("blocks"),
            )
        })
        .expect("relation update should execute");

    assert_eq!(receipt.mutation_family(), WorthQueryMutationFamily::Update);
    assert_eq!(
        receipt.terminal_target_collection_projection(),
        Some("TaskRelation")
    );
    assert_eq!(
        receipt.target_entity_identity(),
        Some(&seed.deltas()[0].entity_identity)
    );
    let evidence = receipt
        .existing_truth_binding_evidence()
        .expect("relation update should retain binding evidence");
    assert_eq!(
        evidence.family(),
        WorthQueryExistingTruthBindingFamily::DirectRelationIdentity
    );
    assert_eq!(
        evidence.authoritative_identity().as_str(),
        "authority:rel-1"
    );
    assert_eq!(
        evidence.resolved_relation_identity(),
        &seed.deltas()[0].entity_identity
    );

    match workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect")
    {
        WorthQueryInspection::WriteReceipt(inspection) => {
            let inspected = inspection
                .existing_truth_binding_evidence()
                .expect("inspection should retain relation binding evidence");
            assert_eq!(
                inspected.family(),
                WorthQueryExistingTruthBindingFamily::DirectRelationIdentity
            );
            assert_eq!(
                inspected.authoritative_identity().as_str(),
                "authority:rel-1"
            );
            assert_eq!(
                inspected.resolved_relation_identity(),
                &seed.deltas()[0].entity_identity
            );
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
}
