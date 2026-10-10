use super::*;

#[test]
fn naming_existing_target_denies_missing_binding_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.naming-denial")
        .expect("task runtime should open a named workspace");

    let error = workspace
        .update(
            request_execution,
            test_entity_identity("entity:0:1:0"),
            |task| {
                task.naming_attach_existing_target(
                    crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(
                        crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new(
                            "persistent-name:task-1",
                        )
                        .expect("naming attachment authority label"),
                    )
                    .expect("naming attachment identity"),
                    crate::runtime::WorthQueryMutationAuthorityIdentity::naming_target_authority(
                        crate::runtime::WorthQueryNamingTargetAuthorityLabel::new(
                            "persistent-name:task-1",
                        )
                        .expect("naming target authority label"),
                    )
                    .expect("naming target authority identity"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("No binding"),
                )
            },
        )
        .expect_err("naming attach-to-existing should deny without binding");

    match error {
        WorthQueryRuntimeError::MutationNamingDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryNamingMutationDenialKind::RequiresExistingTruthBinding
            );
            assert!(!denial.denial_digest().is_empty());
        }
        other => panic!("expected typed naming denial, got {other:?}"),
    }
}

#[test]
fn naming_remove_denies_non_delete_family_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.naming-remove-denial")
        .expect("task runtime should open a named workspace");

    let command = WorthQueryWriteCommand::UpdateAspects {
        entity_identity: crate::memory_workspace::admit_authored_entity_label("entity:0:1:0"),
        aspects: Vec::new(),
        metadata: WorthQueryMutationMetadata::default(),
        naming_intent: Some(WorthQueryNamingMutationIntent::remove(
            crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(
                crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new(
                    "persistent-name:task-1",
                )
                .expect("naming attachment authority label"),
            )
            .expect("naming attachment identity"),
            crate::runtime::WorthQueryMutationAuthorityIdentity::naming_prior_authority(
                crate::runtime::WorthQueryNamingPriorAuthorityLabel::new("persistent-name:task-1")
                    .expect("naming prior authority label"),
            )
            .expect("naming prior authority identity"),
        )),
        continuity_intent: None,
    };
    let error = workspace
        .write(request_execution, command)
        .expect_err("naming remove should deny on non-delete mutation family");

    match error {
        WorthQueryRuntimeError::MutationNamingDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryNamingMutationDenialKind::RequiresDeleteFamily
            );
            assert!(!denial.denial_digest().is_empty());
        }
        other => panic!("expected typed naming denial, got {other:?}"),
    }
}
