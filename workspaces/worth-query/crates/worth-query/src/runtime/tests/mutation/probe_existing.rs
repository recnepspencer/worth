use super::super::support::*;

#[test]
fn probe_existing_returns_backend_verified_values_for_entity_targets() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.probe-existing-entity")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.probe-existing-entity-table",
            |q| {
                q.from("Task")
                    .select([identity_id_field_key(), title_value_field_key()])
                    .order_by(title_value_field_key())
                    .schema_basis("tasks-probe-existing-entity-table")
            },
        )
        .expect("live view should declare");
    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
        })
        .expect("seed insert should execute");
    let binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding should build");

    let probe = workspace
        .probe_existing(
            request_execution,
            binding,
            test_aspect_touches(["identity.id", "title.value"]),
        )
        .expect("probe should execute");

    assert_eq!(
        probe.mode(),
        WorthQueryExistingTruthProbeMode::BackendVerifiedProbe
    );
    assert_eq!(probe.fields().len(), 2);
    assert_eq!(
        probe
            .field_for_touch(&test_aspect_touch("identity.id"))
            .expect("identity field should exist")
            .foundational_value(),
        &test_string_aspect_value("task-1")
    );
    assert_eq!(
        probe
            .field_for_touch(&test_aspect_touch("title.value"))
            .expect("title field should exist")
            .foundational_value(),
        &test_string_aspect_value("Seed title")
    );
    assert!(!probe.probe_digest().is_empty());
}

#[test]
fn probe_existing_returns_backend_verified_values_for_relation_targets() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = stateful_bridge_task_relation_runtime(request_execution);
    let mut workspace = runtime
        .workspace("tasks.probe-existing-relation")
        .expect("workspace should open");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.probe-existing-relation-table",
            |q| {
                q.from("TaskRelation")
                    .select([identity_id_field_key(), kind_value_field_key()])
                    .order_by(kind_value_field_key())
                    .schema_basis("tasks-probe-existing-relation-table")
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

    let probe = workspace
        .probe_existing(
            request_execution,
            binding,
            test_aspect_touches(["kind.value"]),
        )
        .expect("relation probe should execute");

    assert_eq!(
        probe.binding().family(),
        WorthQueryExistingTruthBindingFamily::DirectRelationIdentity
    );
    assert_eq!(
        probe
            .field_for_touch(&test_aspect_touch("kind.value"))
            .expect("kind field should exist")
            .foundational_value(),
        &test_string_aspect_value("depends_on")
    );
}

#[test]
fn probe_existing_denies_missing_aspect_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.probe-existing-missing-aspect")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.probe-existing-missing-aspect-table",
            |q| {
                q.from("Task")
                    .select([identity_id_field_key(), title_value_field_key()])
                    .order_by(title_value_field_key())
                    .schema_basis("tasks-probe-existing-missing-aspect-table")
            },
        )
        .expect("live view should declare");
    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
        })
        .expect("seed insert should execute");
    let binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding should build");

    let error = workspace
        .probe_existing(
            request_execution,
            binding,
            test_aspect_touches(["status.value"]),
        )
        .expect_err("missing probed aspect should deny");

    match error {
        WorthQueryRuntimeError::ExistingTruthProbeDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthProbeDenialKind::MissingProbedAspect
            );
            assert_eq!(
                denial.probed_aspect_touch(),
                Some(&test_aspect_touch("status.value"))
            );
        }
        other => panic!("expected typed probe denial, got {other:?}"),
    }
}

#[test]
fn probe_existing_reports_the_actual_missing_aspect_in_multi_aspect_requests() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.probe-existing-multi-missing-aspect")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.probe-existing-multi-missing-aspect-table",
            |q| {
                q.from("Task")
                    .select([identity_id_field_key(), title_value_field_key()])
                    .order_by(title_value_field_key())
                    .schema_basis("tasks-probe-existing-multi-missing-aspect-table")
            },
        )
        .expect("live view should declare");
    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
        })
        .expect("seed insert should execute");
    let binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding should build");

    let error = workspace
        .probe_existing(
            request_execution,
            binding,
            test_aspect_touches(["identity.id", "status.value", "title.value"]),
        )
        .expect_err("missing probed aspect should deny");

    match error {
        WorthQueryRuntimeError::ExistingTruthProbeDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthProbeDenialKind::MissingProbedAspect
            );
            assert_eq!(
                denial.probed_aspect_touch(),
                Some(&test_aspect_touch("status.value"))
            );
        }
        other => panic!("expected typed probe denial, got {other:?}"),
    }
}

#[test]
fn probe_existing_denies_unsupported_backend_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime_with_support(
        request_execution,
        WorthQueryRuntimeSupportProfile::bridge_backed(
            "test-subscription-activation",
            "test-preview-basis",
            "test-inspector-evidence",
        ),
    );
    let workspace = runtime
        .workspace("tasks.probe-existing-unsupported")
        .expect("workspace should open");
    let binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"), test_entity_identity("Task:1"))
                .expect("existing entity target should build")
                .in_target_collection("Task")
                .expect("existing entity target collection should build"),
        )
        .expect("binding should build");

    let error = workspace
        .probe_existing(
            request_execution,
            binding,
            test_aspect_touches(["title.value"]),
        )
        .expect_err("unsupported backend probe should deny");

    match error {
        WorthQueryRuntimeError::ExistingTruthProbeDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthProbeDenialKind::BackendProbeUnsupported
            );
        }
        other => panic!("expected typed probe denial, got {other:?}"),
    }
}
