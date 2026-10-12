use super::*;

#[test]
fn update_existing_preserves_continuity_evidence_on_receipt_and_inspection() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.continuity-existing")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.continuity-existing-table", |q| {
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
                .schema_basis("tasks-continuity-existing-table")
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
                test_authored_string_aspect_value("Before continuity rebind"),
            )
        })
        .expect("seed insert should execute");
    let binding_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1")
                .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity");
    let prior_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(
            crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:task-1")
                .expect("continuity prior authority label"),
        )
        .expect("continuity prior authority identity");
    let successor_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
            crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(
                "authority:task-1-successor",
            )
            .expect("continuity successor authority label"),
        )
        .expect("continuity successor authority identity");
    let binding = WorthQueryExistingTruthTargetBinding::direct_entity(
        binding_authority.clone(),
        seed.deltas()[0].entity_identity.clone(),
    )
    .expect("binding should build")
    .in_target_collection("Task")
    .expect("binding collection should build");
    let expected_basis_binding_digest = WorthQueryMutationEvidenceDigest::source_identity(
        "continuity-basis-binding",
        binding.binding_evidence_identity(),
    );
    let receipt = workspace
        .update_existing(request_execution, binding, |task| {
            task.continuity_rebind_existing_target(
                prior_authority.clone(),
                successor_authority.clone(),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("After continuity rebind"),
            )
        })
        .expect("continuity-aware existing-target update should execute");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("continuity-aware receipt should inspect");

    let continuity = receipt
        .continuity_mutation_evidence()
        .expect("receipt should retain continuity evidence");
    assert_eq!(
        continuity.family(),
        WorthQueryContinuityMutationFamily::RebindExistingTarget
    );
    assert_eq!(
        continuity.outcome_class(),
        WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor
    );
    assert_eq!(
        continuity.prior_authoritative_identity().as_str(),
        expected_bridge_continuity_authority_label(&prior_authority).as_str()
    );
    assert_eq!(
        continuity
            .successor_authoritative_identity()
            .map(|identity| identity.as_str()),
        Some(expected_bridge_continuity_authority_label(&successor_authority).as_str(),)
    );
    let basis_binding_digest = continuity
        .basis_binding_digest()
        .expect("continuity evidence should retain typed query binding basis digest");
    assert_eq!(
        basis_binding_digest.as_str(),
        expected_basis_binding_digest.as_str()
    );
    assert_eq!(
        continuity.resolved_target_entity_identity(),
        Some(&seed.deltas()[0].entity_identity)
    );
    assert_eq!(
        continuity
            .target_collection()
            .map(|collection| collection.as_str()),
        Some("Task")
    );
    assert!(!continuity.lineage_digest().is_empty());
    assert!(!continuity.continuity_resolution_digest().is_empty());

    match inspection {
        WorthQueryInspection::WriteReceipt(inspection) => {
            let continuity = inspection
                .continuity_mutation_evidence()
                .expect("inspection should retain continuity evidence");
            assert_eq!(
                continuity.prior_authoritative_identity().as_str(),
                expected_bridge_continuity_authority_label(&prior_authority).as_str()
            );
            assert_eq!(
                continuity
                    .successor_authoritative_identity()
                    .map(|identity| identity.as_str()),
                Some(expected_bridge_continuity_authority_label(&successor_authority).as_str(),)
            );
            assert_eq!(
                continuity
                    .basis_binding_digest()
                    .map(|digest| digest.as_str()),
                Some(basis_binding_digest.as_str())
            );
            assert_eq!(
                continuity.lineage_digest(),
                receipt
                    .continuity_mutation_evidence()
                    .expect("receipt should retain continuity evidence")
                    .lineage_digest()
            );
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
}

#[test]
fn mixed_batch_preserves_continuity_and_naming_session_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.continuity-batch")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.continuity-batch-table", |q| {
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
                .schema_basis("tasks-continuity-batch-table")
        })
        .expect("live view should declare");

    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-existing"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Existing"),
            )
        })
        .expect("seed insert should execute");
    let binding = WorthQueryExistingTruthTargetBinding::direct_entity(
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1")
                .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity"),
        seed.deltas()[0].entity_identity.clone(),
    )
    .expect("binding should build")
    .in_target_collection("Task")
    .expect("binding collection should build");
    let binding_digest = binding.binding_digest();

    let receipt = workspace
        .batch(request_execution, |batch| {
            batch
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
                        task.naming_attach_new_target(
                            crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new(
                                "persistent-name:draft",
                            )
                            .expect("naming attachment authority label")).expect("naming attachment identity"),
                        )
                        .set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Draft named"))
                    },
                )
                .update_existing(binding, |task| {
                    task.continuity_rebind_merge_successor(crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:task-existing").expect("continuity prior authority label")).expect("continuity prior authority identity"), crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new("authority:task-existing-merged").expect("continuity successor authority label")).expect("continuity successor authority identity"),
                    )
                    .set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Existing continuity merged"))
                })
        })
        .expect("mixed continuity batch should execute");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("batch should inspect");

    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .continuity_mutation_count(),
        1
    );
    assert_eq!(receipt.batch_mutation_evidence().naming_mutation_count(), 1);
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .existing_truth_binding_count(),
        1
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_target_reference_count(),
        1
    );
    assert!(receipt
        .batch_mutation_evidence()
        .aggregate_continuity_mutation_digest()
        .is_some());

    match inspection {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .continuity_mutation_count(),
                1
            );
            assert!(inspection
                .batch_mutation_evidence()
                .aggregate_continuity_mutation_digest()
                .is_some());
            assert_eq!(
                inspection.component_operations()[2]
                    .continuity_mutation_evidence()
                    .expect("existing continuity component should retain evidence")
                    .outcome_class(),
                WorthQueryContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor
            );
            assert_eq!(
                inspection.component_operations()[2]
                    .continuity_mutation_evidence()
                    .expect("existing continuity component should retain evidence")
                    .basis_binding_digest()
                    .map(|digest| digest.as_str()),
                Some(binding_digest.as_str())
            );
        }
        other => panic!("expected batch write inspection, got {other:?}"),
    }
}
