use super::*;

#[test]
fn batch_update_existing_verified_preserves_aggregate_assertion_digest() {
    let mut workspace = stateful_bridge_task_runtime()
        .workspace("tasks.batch-update-existing-verified")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view("tasks.batch-update-existing-verified-table", |q| {
            q.from("Task")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("status", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                )
                .schema_basis("tasks-batch-update-existing-verified-table")
        })
        .expect("live view should declare");

    let seed_one = workspace
        .insert("Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("First"),
            )
            .set_aspect(
                test_aspect_touch("status.value"),
                test_authored_string_aspect_value("open"),
            )
        })
        .expect("first seed should execute");
    let seed_two = workspace
        .insert("Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-2"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Second"),
            )
        })
        .expect("second seed should execute");

    let binding_one = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed_one.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding one should build");
    let binding_two = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed_two.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding two should build");

    let receipt = workspace
        .batch(|batch| {
            batch
                .update_existing_verified(
                    binding_one,
                    |task| {
                        task.set_aspect(
                            test_aspect_touch("status.value"),
                            test_authored_string_aspect_value("open"),
                        )
                    },
                    |task| {
                        task.set_aspect(
                            test_aspect_touch("status.value"),
                            test_authored_string_aspect_value("closed"),
                        )
                    },
                )
                .delete_existing_with(binding_two, |delete| {
                    delete.touch(test_aspect_touch("title.value"))
                })
        })
        .expect("mixed batch should execute");

    assert_eq!(
        receipt.write_receipts()[0].mutation_family(),
        WorthQueryMutationFamily::Update
    );
    assert_eq!(
        receipt.write_receipts()[0]
            .existing_truth_assertion_evidence()
            .expect("verified update should retain assertion evidence")
            .mode(),
        WorthQueryExistingTruthAssertionMode::BackendVerifiedAssertion
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .existing_truth_assertion_count(),
        1
    );
    assert!(receipt
        .batch_mutation_evidence()
        .aggregate_existing_truth_assertion_digest()
        .is_some());
}

#[test]
fn primary_multi_verified_update_batch_shares_one_commit_boundary() {
    let attempted_writes = std::rc::Rc::new(std::cell::Cell::new(0usize));
    let attempted_batches = std::rc::Rc::new(std::cell::Cell::new(0usize));
    let runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .existing_truth_verification(PermissiveExistingTruthVerificationAdapter)
        .write_authority(AtomicBatchCountingWriteAuthority {
            attempted_writes: attempted_writes.clone(),
            attempted_batches: attempted_batches.clone(),
        })
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .support_profile(WorthQueryRuntimeSupportProfile::bridge_backed(
            "test-subscription-activation",
            "test-preview-basis",
            "test-inspector-evidence",
        ))
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("verified update test aspect contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("primary bridge-backed runtime should build");
    let mut workspace = runtime
        .workspace("tasks.batch-update-existing-verified-atomic")
        .expect("task runtime should open a named workspace");

    let binding_one = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                test_entity_identity("Task:1"),
            )
            .expect("first existing entity target should build")
            .in_target_collection("Task")
            .expect("first existing entity target collection should build"),
        )
        .expect("first binding should build");
    let binding_two = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                test_entity_identity("Task:2"),
            )
            .expect("second existing entity target should build")
            .in_target_collection("Task")
            .expect("second existing entity target collection should build"),
        )
        .expect("second binding should build");

    let receipt = workspace
        .batch(|batch| {
            batch
                .update_existing_verified(
                    binding_one,
                    |task| {
                        task.set_aspect(
                            test_aspect_touch("status.value"),
                            test_authored_string_aspect_value("open"),
                        )
                    },
                    |task| {
                        task.set_aspect(
                            test_aspect_touch("status.value"),
                            test_authored_string_aspect_value("closed"),
                        )
                    },
                )
                .update_existing_verified(
                    binding_two,
                    |task| {
                        task.set_aspect(
                            test_aspect_touch("status.value"),
                            test_authored_string_aspect_value("open"),
                        )
                    },
                    |task| {
                        task.set_aspect(
                            test_aspect_touch("status.value"),
                            test_authored_string_aspect_value("closed"),
                        )
                    },
                )
        })
        .expect("verified update batch should execute atomically");

    assert_eq!(receipt.write_receipts().len(), 2);
    assert_eq!(attempted_batches.get(), 1);
    assert_eq!(attempted_writes.get(), 0);
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .backend_verified_update_count(),
        2
    );
}
