use super::*;

#[test]
fn continuity_update_denies_missing_binding_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.continuity-denial")
        .expect("task runtime should open a named workspace");

    let error = workspace
        .update(request_execution, test_entity_identity("entity:0:1:0"), |task| {
            task.continuity_rebind_existing_target(crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:task-1").expect("continuity prior authority label")).expect("continuity prior authority identity"), crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new("authority:task-1-successor").expect("continuity successor authority label")).expect("continuity successor authority identity"))
                .set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("No binding"))
        })
        .expect_err("continuity-aware update should deny without existing binding");

    match error {
        WorthQueryRuntimeError::MutationContinuityDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryContinuityMutationDenialKind::RequiresExistingTruthBinding
            );
            assert!(!denial.denial_digest().is_empty());
        }
        other => panic!("expected typed continuity denial, got {other:?}"),
    }
}

#[test]
fn continuity_insert_denies_non_update_family_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.continuity-insert-denial")
        .expect("task runtime should open a named workspace");

    let error = workspace
        .insert(request_execution, "Task", |task| {
            task.continuity_rebind_existing_target(crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:task-1").expect("continuity prior authority label")).expect("continuity prior authority identity"), crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new("authority:task-1-successor").expect("continuity successor authority label")).expect("continuity successor authority identity"))
                .set_aspect(test_aspect_touch("identity.id"), test_authored_string_aspect_value("task-2"))
        })
        .expect_err("continuity-aware insert should deny on non-update family");

    match error {
        WorthQueryRuntimeError::MutationContinuityDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryContinuityMutationDenialKind::RequiresUpdateFamily
            );
            assert!(!denial.denial_digest().is_empty());
        }
        other => panic!("expected typed continuity denial, got {other:?}"),
    }
}

#[test]
fn preview_update_existing_denies_continuity_without_authoritative_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.preview-continuity-denial")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.preview-continuity-denial-table",
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
                    .schema_basis("tasks-preview-continuity-denial-table")
            },
        )
        .expect("live view should declare");
    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-preview"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Preview continuity"),
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
    let mut preview = workspace
        .preview(request_execution, test_session_label("continuity denial"))
        .expect("preview should open");

    let error = preview
        .update_existing(request_execution, binding, |task| {
            task.continuity_rebind_existing_target(crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:task-preview").expect("continuity prior authority label")).expect("continuity prior authority identity"), crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new("authority:task-preview-successor").expect("continuity successor authority label")).expect("continuity successor authority identity"),
            )
            .set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Preview continuity denied"))
        })
        .expect_err("preview continuity should deny outside authoritative lane");

    match error {
        WorthQueryRuntimeError::MutationContinuityDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryContinuityMutationDenialKind::RequiresAuthoritativeLane
            );
            assert_eq!(denial.basis_binding_digest(), Some(binding_digest.as_str()));
            assert!(!denial.denial_digest().is_empty());
        }
        other => panic!("expected typed continuity denial, got {other:?}"),
    }
}

#[test]
fn preview_batch_denies_continuity_without_authoritative_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.preview-batch-continuity-denial")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.preview-batch-continuity-denial-table",
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
                    .schema_basis("tasks-preview-batch-continuity-denial-table")
            },
        )
        .expect("live view should declare");
    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-preview-batch"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Preview continuity batch"),
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
    let mut preview = workspace
        .preview(
            request_execution,
            test_session_label("continuity batch denial"),
        )
        .expect("preview should open");

    let error = preview
        .batch(request_execution, |batch| {
            batch.update_existing(binding, |task| {
                task.continuity_rebind_existing_target(crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:task-preview-batch").expect("continuity prior authority label")).expect("continuity prior authority identity"), crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new("authority:task-preview-batch-successor").expect("continuity successor authority label")).expect("continuity successor authority identity"),
                )
                .set_aspect(test_aspect_touch("title.value"), test_authored_string_aspect_value("Preview continuity batch denied"))
            })
        })
        .expect_err("preview batch continuity should deny outside authoritative lane");

    match error {
        WorthQueryRuntimeError::MutationContinuityDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryContinuityMutationDenialKind::RequiresAuthoritativeLane
            );
            assert_eq!(denial.basis_binding_digest(), Some(binding_digest.as_str()));
            assert!(!denial.denial_digest().is_empty());
        }
        other => panic!("expected typed continuity denial, got {other:?}"),
    }
}
