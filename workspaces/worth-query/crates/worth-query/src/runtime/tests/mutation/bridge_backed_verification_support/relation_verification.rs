use super::*;

#[test]
fn bridge_backed_relation_verification_rows_match_runtime_behavior() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_relation_runtime(request_execution)
        .workspace("tasks.bridge-backed-relation-verification-support")
        .expect("workspace should open");
    let support = workspace.public_authoritative_mutation_evidence_support();
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.bridge-backed-relation-verification-support-table",
            |q| {
                q.from("TaskRelation")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("status", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                    )
                    .schema_basis("tasks-bridge-backed-relation-verification-support-table")
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
                    test_authored_string_aspect_value("active"),
                )
        })
        .expect("seed insert should execute");
    let binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing relation target should build")
            .in_target_collection("TaskRelation")
            .expect("existing relation target collection should build"),
        )
        .expect("relation binding should build");

    for operation_family in [
        "verify_existing",
        "probe_existing",
        "update_existing_verified",
        "delete_existing_verified",
    ] {
        let row = verification_row(&support, operation_family, "direct_relation_identity");
        assert_eq!(
            row.current_posture_status(),
            WorthQueryBridgeBackedVerificationSupportStatus::Admitted
        );
    }

    workspace
        .verify_existing(request_execution, binding.clone(), |relation| {
            relation.set_aspect(
                test_aspect_touch("status.value"),
                test_authored_string_aspect_value("active"),
            )
        })
        .expect("relation verify_existing should execute");
    workspace
        .probe_existing(
            request_execution,
            binding.clone(),
            test_aspect_touches(["status.value"]),
        )
        .expect("relation probe_existing should execute");
    workspace
        .update_existing_verified(
            request_execution,
            binding.clone(),
            |verify| {
                verify.set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("active"),
                )
            },
            |update| {
                update.set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("retired"),
                )
            },
        )
        .expect("relation update_existing_verified should execute");
    workspace
        .delete_existing_verified(
            request_execution,
            binding,
            |verify| {
                verify.set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("retired"),
                )
            },
            |delete| delete.touch(test_aspect_touch("status.value")),
        )
        .expect("relation delete_existing_verified should execute");
}

#[test]
fn primary_relation_bridge_backed_verification_rows_match_runtime_denials() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = bridge_runtime_with_support(
        request_execution,
        WorthQueryRuntimeSupportProfile::bridge_backed(
            "test-subscription-activation",
            "test-preview-basis",
            "test-inspector-evidence",
        ),
    )
    .workspace("tasks.primary-relation-verification-support")
    .expect("workspace should open");
    let support = workspace.public_authoritative_mutation_evidence_support();
    let binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                test_entity_identity("TaskRelation:1"),
            )
            .expect("existing relation target should build")
            .in_target_collection("TaskRelation")
            .expect("existing relation target collection should build"),
        )
        .expect("relation binding should build");

    assert_eq!(
        verification_row(&support, "verify_existing", "direct_relation_identity")
            .current_posture_status(),
        WorthQueryBridgeBackedVerificationSupportStatus::Denied
    );
    assert_eq!(
        verification_row(&support, "probe_existing", "direct_relation_identity")
            .denial_class_when_unsupported(),
        Some("backend_probe_unsupported")
    );

    assert!(matches!(
        workspace.verify_existing(request_execution, binding.clone(), |relation| relation.set_aspect(test_aspect_touch("kind.value"), test_authored_string_aspect_value("depends_on"))),
        Err(WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial))
            if denial.kind() == WorthQueryExistingTruthAssertionDenialKind::BackendVerificationUnsupported
    ));
    assert!(matches!(
        workspace.probe_existing(request_execution, binding.clone(), test_aspect_touches(["kind.value"])),
        Err(WorthQueryRuntimeError::ExistingTruthProbeDenied(denial))
            if denial.kind() == WorthQueryExistingTruthProbeDenialKind::BackendProbeUnsupported
    ));
    assert!(matches!(
        workspace.update_existing_verified(
            request_execution, binding.clone(),
            |verify| verify.set_aspect(test_aspect_touch("kind.value"), test_authored_string_aspect_value("depends_on")),
            |update| update.set_aspect(test_aspect_touch("kind.value"), test_authored_string_aspect_value("blocked_by")),
        ),
        Err(WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial))
            if denial.kind() == WorthQueryExistingTruthAssertionDenialKind::BackendVerificationUnsupported
    ));
    assert!(matches!(
        workspace.delete_existing_verified(
            request_execution, binding,
            |verify| verify.set_aspect(test_aspect_touch("kind.value"), test_authored_string_aspect_value("depends_on")),
            |delete| delete.touch(test_aspect_touch("kind.value")),
        ),
        Err(WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial))
            if denial.kind() == WorthQueryExistingTruthAssertionDenialKind::BackendVerificationUnsupported
    ));
}
