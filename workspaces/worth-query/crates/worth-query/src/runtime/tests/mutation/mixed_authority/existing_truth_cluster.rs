use super::*;

#[test]
fn mixed_batch_preserves_existing_truth_mode_and_neighbor_aggregate_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.mixed-authority-batch")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.mixed-authority-batch-table",
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
                    .schema_basis("tasks-mixed-authority-batch-table")
            },
        )
        .expect("live view should declare");

    let assert_seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-assert"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Assert seed"),
            )
        })
        .expect("assert seed insert should execute");
    let verify_seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-verify"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Verify seed"),
            )
        })
        .expect("verify seed insert should execute");
    let update_seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-update"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Update seed"),
            )
        })
        .expect("update seed insert should execute");
    let delete_seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-delete"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Delete seed"),
            )
        })
        .expect("delete seed insert should execute");

    let assert_binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-assert").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                assert_seed.deltas()[0].entity_identity.clone(),
            )
            .expect("assert target should build")
            .in_target_collection("Task")
            .expect("assert target collection should build"),
        )
        .expect("assert binding should build");
    let verify_binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-assert").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                verify_seed.deltas()[0].entity_identity.clone(),
            )
            .expect("verify target should build")
            .in_target_collection("Task")
            .expect("verify target collection should build"),
        )
        .expect("verify binding should build");
    let update_binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-assert").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                update_seed.deltas()[0].entity_identity.clone(),
            )
            .expect("update target should build")
            .in_target_collection("Task")
            .expect("update target collection should build"),
        )
        .expect("update binding should build");
    let update_binding_digest = update_binding.binding_digest().to_string();
    let update_probe_binding = update_binding.clone();
    let delete_binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-assert").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                delete_seed.deltas()[0].entity_identity.clone(),
            )
            .expect("delete target should build")
            .in_target_collection("Task")
            .expect("delete target collection should build"),
        )
        .expect("delete binding should build");

    let receipt = workspace
        .batch(request_execution, |batch| {
            batch
                .assert_existing(assert_binding, |task| {
                    task.set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Assert seed"))
                })
                .verify_existing(verify_binding, |task| {
                    task.set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Verify seed"))
                })
                .update_existing_verified(
                    update_binding,
                    |task| task.set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Update seed")),
                    |task| {
                        task.continuity_rebind_merge_successor(crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:task-update").expect("continuity prior authority label")).expect("continuity prior authority identity"), crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new("authority:task-update-merged").expect("continuity successor authority label")).expect("continuity successor authority identity"),
                        )
                        .set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Update merged"))
                    },
                )
                .delete_existing_verified(
                    delete_binding,
                    |task| task.set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Delete seed")),
                    |delete| {
                        delete
                            .touch(test_aspect_touch("title.value"))
                            .naming_remove(crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new("persistent-name:delete").expect("naming attachment authority label")).expect("naming attachment identity"), crate::runtime::WorthQueryMutationAuthorityIdentity::naming_prior_authority(crate::runtime::WorthQueryNamingPriorAuthorityLabel::new("persistent-name:delete").expect("naming prior authority label")).expect("naming prior authority identity"))
                    },
                )
                .insert_symbolic("draft-task", "Task", |task| {
                    task.set_aspect(test_aspect_touch("identity.id"), test_authored_string_aspect_value("task-draft"))
                        .set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Draft"))
                })
                .update_symbolic(
                    WorthQuerySymbolicTargetReference::new("draft-task")
                        .expect("symbolic reference should build")
                        .in_target_collection("Task")
                        .expect("symbolic collection should build"),
                    |task| {
                        task.naming_attach_new_target(crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new("persistent-name:draft").expect("naming attachment authority label")).expect("naming attachment identity"))
                            .set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Draft named"))
                    },
                )
        })
        .expect("mixed authority batch should execute");

    let batch_evidence = receipt.batch_mutation_evidence();
    assert_eq!(batch_evidence.component_count(), 6);
    assert_eq!(batch_evidence.existing_truth_assertion_count(), 4);
    assert_eq!(batch_evidence.retained_authoritative_assertion_count(), 1);
    assert_eq!(batch_evidence.backend_verified_assertion_count(), 1);
    assert_eq!(batch_evidence.backend_verified_update_count(), 1);
    assert_eq!(batch_evidence.backend_verified_delete_count(), 1);
    assert_eq!(batch_evidence.existing_truth_binding_count(), 4);
    assert_eq!(batch_evidence.symbolic_target_reference_count(), 1);
    assert_eq!(batch_evidence.naming_mutation_count(), 2);
    assert_eq!(batch_evidence.continuity_mutation_count(), 1);

    let mode_digest = batch_evidence
        .aggregate_existing_truth_mode_digest()
        .expect("mixed batch should retain existing-truth mode digest")
        .as_str()
        .to_string();
    let assertion_digest = batch_evidence
        .aggregate_existing_truth_assertion_digest()
        .expect("mixed batch should retain assertion digest")
        .as_str()
        .to_string();
    let binding_digest = batch_evidence
        .aggregate_existing_truth_binding_digest()
        .expect("mixed batch should retain binding digest")
        .as_str()
        .to_string();
    let symbolic_digest = batch_evidence
        .aggregate_symbolic_target_reference_digest()
        .expect("mixed batch should retain symbolic digest")
        .as_str()
        .to_string();
    let naming_digest = batch_evidence
        .aggregate_naming_mutation_digest()
        .expect("mixed batch should retain naming digest")
        .as_str()
        .to_string();
    let continuity_digest = batch_evidence
        .aggregate_continuity_mutation_digest()
        .expect("mixed batch should retain continuity digest")
        .as_str()
        .to_string();
    let update_probe = workspace
        .probe_existing(
            request_execution,
            update_probe_binding,
            test_aspect_touches(["title.value"]),
        )
        .expect("updated target should remain probeable after mixed batch");
    assert_eq!(
        update_probe.mode(),
        WorthQueryExistingTruthProbeMode::BackendVerifiedProbe
    );
    assert_eq!(
        update_probe
            .field_for_touch(&test_aspect_touch("title.value"))
            .expect("probe should retain updated title")
            .foundational_value(),
        &test_string_aspect_value("Update merged")
    );
    assert_ne!(update_probe.probe_digest(), mode_digest.as_str());

    match workspace
        .inspect(request_execution, &receipt)
        .expect("batch should inspect")
    {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            let batch_evidence = inspection.batch_mutation_evidence();
            assert_eq!(
                batch_evidence
                    .aggregate_existing_truth_mode_digest()
                    .map(|digest| digest.as_str()),
                Some(mode_digest.as_str())
            );
            assert_eq!(
                batch_evidence
                    .aggregate_existing_truth_assertion_digest()
                    .map(|digest| digest.as_str()),
                Some(assertion_digest.as_str())
            );
            assert_eq!(
                batch_evidence
                    .aggregate_existing_truth_binding_digest()
                    .map(|digest| digest.as_str()),
                Some(binding_digest.as_str())
            );
            assert_eq!(
                batch_evidence
                    .aggregate_symbolic_target_reference_digest()
                    .map(|digest| digest.as_str()),
                Some(symbolic_digest.as_str())
            );
            assert_eq!(
                batch_evidence
                    .aggregate_naming_mutation_digest()
                    .map(|digest| digest.as_str()),
                Some(naming_digest.as_str())
            );
            assert_eq!(
                batch_evidence
                    .aggregate_continuity_mutation_digest()
                    .map(|digest| digest.as_str()),
                Some(continuity_digest.as_str())
            );
            assert_eq!(
                inspection.component_operations()[0].family(),
                WorthQueryMutationFamily::Assertion.as_str()
            );
            assert_eq!(
                inspection.component_operations()[1].family(),
                WorthQueryMutationFamily::Assertion.as_str()
            );
            assert_eq!(
                inspection.component_operations()[0]
                    .existing_truth_assertion_evidence()
                    .expect("retained assertion should keep assertion evidence")
                    .mode(),
                WorthQueryExistingTruthAssertionMode::RetainedAuthoritativeAssertion
            );
            assert_eq!(
                inspection.component_operations()[1]
                    .existing_truth_assertion_evidence()
                    .expect("verified assertion should keep assertion evidence")
                    .mode(),
                WorthQueryExistingTruthAssertionMode::BackendVerifiedAssertion
            );
            assert_eq!(
                inspection.component_operations()[2].family(),
                WorthQueryMutationFamily::Update.as_str()
            );
            assert_eq!(
                inspection.component_operations()[3].family(),
                WorthQueryMutationFamily::Delete.as_str()
            );
            assert_eq!(
                inspection.component_operations()[2]
                    .existing_truth_assertion_evidence()
                    .expect("verified update should keep assertion evidence")
                    .mode(),
                WorthQueryExistingTruthAssertionMode::BackendVerifiedAssertion
            );
            assert_eq!(
                inspection.component_operations()[3]
                    .existing_truth_assertion_evidence()
                    .expect("verified delete should keep assertion evidence")
                    .mode(),
                WorthQueryExistingTruthAssertionMode::BackendVerifiedAssertion
            );
            assert_eq!(
                inspection.component_operations()[2]
                    .continuity_mutation_evidence()
                    .expect("verified update should retain continuity evidence")
                    .basis_binding_digest()
                    .map(|digest| digest.as_str()),
                Some(update_binding_digest.as_str())
            );
            assert_eq!(
                inspection.component_operations()[3]
                    .naming_mutation_evidence()
                    .expect("verified delete should retain naming evidence")
                    .outcome(),
                WorthQueryNamingMutationOutcome::Removed
            );
        }
        other => panic!("expected batch write inspection, got {other:?}"),
    }
}
