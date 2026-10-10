use super::*;

#[test]
fn delete_existing_preserves_naming_removal_evidence_on_receipt_and_inspection() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.naming-remove")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.naming-remove-table", |q| {
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
                .schema_basis("tasks-naming-remove-table")
        })
        .expect("live view should declare");

    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Named task"),
            )
        })
        .expect("seed insert should execute");
    let attachment_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(
            crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new("persistent-name:task-1")
                .expect("naming attachment authority label"),
        )
        .expect("naming attachment identity");
    let prior_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::naming_prior_authority(
            crate::runtime::WorthQueryNamingPriorAuthorityLabel::new("persistent-name:task-1")
                .expect("naming prior authority label"),
        )
        .expect("naming prior authority identity");
    let binding_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1")
                .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity");
    let binding = WorthQueryExistingTruthTargetBinding::direct_entity(
        binding_authority.clone(),
        seed.deltas()[0].entity_identity.clone(),
    )
    .expect("binding should build")
    .in_target_collection("Task")
    .expect("binding collection should build");

    let receipt = workspace
        .delete_existing_with(request_execution, binding, |delete| {
            delete
                .touch(test_aspect_touch("title.value"))
                .naming_remove(attachment_authority.clone(), prior_authority.clone())
        })
        .expect("existing-target naming delete should execute");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("naming delete receipt should inspect");

    let naming = receipt
        .naming_mutation_evidence()
        .expect("receipt should retain naming removal evidence");
    assert_eq!(naming.family(), WorthQueryNamingMutationFamily::Remove);
    assert_eq!(naming.outcome(), WorthQueryNamingMutationOutcome::Removed);
    assert_eq!(
        naming.attachment_identity().as_str(),
        expected_bridge_naming_attachment_label(&attachment_authority).as_str()
    );
    assert_eq!(
        naming
            .prior_authoritative_identity()
            .map(|identity| identity.as_str()),
        Some(expected_bridge_naming_authority_label(&binding_authority).as_str())
    );
    assert_eq!(naming.target_authoritative_identity(), None);
    assert_eq!(
        naming.resolved_target_entity_identity(),
        Some(&seed.deltas()[0].entity_identity)
    );

    match inspection {
        WorthQueryInspection::WriteReceipt(inspection) => {
            let naming = inspection
                .naming_mutation_evidence()
                .expect("inspection should retain naming removal evidence");
            assert_eq!(naming.family(), WorthQueryNamingMutationFamily::Remove);
            assert_eq!(naming.outcome(), WorthQueryNamingMutationOutcome::Removed);
            assert_eq!(
                naming.attachment_identity().as_str(),
                expected_bridge_naming_attachment_label(&attachment_authority).as_str()
            );
            assert_eq!(
                naming
                    .prior_authoritative_identity()
                    .map(|identity| identity.as_str()),
                Some(expected_bridge_naming_authority_label(&binding_authority).as_str())
            );
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
}

#[test]
fn preview_batch_symbolic_naming_preserves_typed_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.preview-naming")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.preview-naming-table", |q| {
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
                .schema_basis("tasks-preview-naming-table")
        })
        .expect("live view should declare");

    let mut preview = workspace
        .preview(request_execution, test_session_label("naming-preview"))
        .expect("preview should open");
    let receipt = preview
        .batch(request_execution, |batch| {
            batch
                .insert_symbolic("draft-task", "Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("task-draft"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Draft"),
                    )
                })
                .update_symbolic(
                    WorthQuerySymbolicTargetReference::new("draft-task")
                        .expect("symbolic reference should build")
                        .in_target_collection("Task")
                        .expect("symbolic collection should build"),
                    |task| {
                        task.naming_attach_new_target(
                            crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(
                                crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new(
                                    "persistent-name:draft",
                                )
                                .expect("naming attachment authority label"),
                            )
                            .expect("naming attachment identity"),
                        )
                        .set_aspect(
                            test_aspect_touch("title.value"),
                            test_authored_string_aspect_value("Preview named"),
                        )
                    },
                )
        })
        .expect("preview naming batch should execute");

    let naming = receipt.write_receipts()[1]
        .naming_mutation_evidence()
        .expect("preview symbolic component should retain naming evidence");
    assert_eq!(
        naming.outcome(),
        WorthQueryNamingMutationOutcome::AttachedToNewTarget
    );
    assert_eq!(
        naming.attachment_identity().as_str(),
        "persistent-name:draft"
    );
}
