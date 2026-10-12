use super::*;

fn seeded_existing_binding(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    workspace_name: &str,
) -> WorthQueryExistingTruthTargetBinding {
    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value(format!("{workspace_name}-task")),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
            .set_aspect(
                test_aspect_touch("status.value"),
                test_authored_string_aspect_value("open"),
            )
        })
        .expect("seed insert should execute");
    workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new(format!("authority:{workspace_name}-task")).expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding should build")
}

#[test]
fn workspace_verify_existing_delegates_to_authoritative_mutation_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    const WORKSPACE_NAME: &str = "intent-admission-verify-existing";

    let mut delegated_workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-verify-existing-delegated")
        .expect("workspace should open");
    let delegated_binding =
        seeded_existing_binding(request_execution, &mut delegated_workspace, WORKSPACE_NAME);
    let delegated = delegated_workspace
        .verify_existing(request_execution, delegated_binding, |task| {
            task.set_aspect(
                test_aspect_touch("status.value"),
                test_authored_string_aspect_value("open"),
            )
        })
        .expect("legacy verify_existing should execute");

    let mut canonical_workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-verify-existing-canonical")
        .expect("workspace should open");
    let canonical_binding =
        seeded_existing_binding(request_execution, &mut canonical_workspace, WORKSPACE_NAME);
    let command = WorthQueryAspectMutationBuilder::new()
        .set_aspect(
            test_aspect_touch("status.value"),
            test_authored_string_aspect_value("open"),
        )
        .build_verify_existing(canonical_binding)
        .expect("verify command should build");
    let canonical = canonical_workspace
        .write_intent(command)
        .execute(request_execution)
        .expect("canonical write intent should execute");
    let delegated_assumptions = delegated
        .verified_assumption_set()
        .expect("delegated verify should retain assumption set");
    let canonical_assumptions = canonical
        .verified_assumption_set()
        .expect("canonical verify should retain assumption set");

    assert_eq!(delegated_assumptions, canonical_assumptions);
    assert_eq!(
        canonical
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteScalarWrite)
    );
}

#[test]
fn workspace_update_existing_verified_delegates_to_authoritative_mutation_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    const WORKSPACE_NAME: &str = "intent-admission-update-existing-verified";

    let mut delegated_workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-update-existing-verified-delegated")
        .expect("workspace should open");
    let delegated_binding =
        seeded_existing_binding(request_execution, &mut delegated_workspace, WORKSPACE_NAME);
    let delegated = delegated_workspace
        .update_existing_verified(
            request_execution,
            delegated_binding.clone(),
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
        .expect("legacy verified update should execute");
    let delegated_probe = delegated_workspace
        .probe_existing(
            request_execution,
            delegated_binding,
            test_aspect_touches(["status.value", "title.value", "identity.id"]),
        )
        .expect("delegated post-update probe should execute");

    let mut canonical_workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-update-existing-verified-canonical")
        .expect("workspace should open");
    let canonical_binding =
        seeded_existing_binding(request_execution, &mut canonical_workspace, WORKSPACE_NAME);
    let asserted_aspects = WorthQueryAspectMutationBuilder::new()
        .set_aspect(
            test_aspect_touch("status.value"),
            test_authored_string_aspect_value("open"),
        )
        .finish_existing_truth_verification_aspects("backend-verified existing-truth update")
        .expect("asserted aspects should build");
    let command = WorthQueryAspectMutationBuilder::new()
        .set_aspect(
            test_aspect_touch("status.value"),
            test_authored_string_aspect_value("closed"),
        )
        .build_update_existing_verified(canonical_binding.clone(), asserted_aspects)
        .expect("verified update command should build");
    let canonical = canonical_workspace
        .write_intent(command)
        .execute(request_execution)
        .expect("canonical write intent should execute");
    let canonical_probe = canonical_workspace
        .probe_existing(
            request_execution,
            canonical_binding,
            test_aspect_touches(["status.value", "title.value", "identity.id"]),
        )
        .expect("canonical post-update probe should execute");
    let delegated_assumptions = delegated
        .verified_assumption_set()
        .expect("delegated update should retain assumption set");
    let canonical_assumptions = canonical
        .verified_assumption_set()
        .expect("canonical update should retain assumption set");

    assert_eq!(delegated_assumptions, canonical_assumptions);
    assert_eq!(
        delegated_probe
            .field_for_touch(&test_aspect_touch("status.value"))
            .map(|field| field.foundational_value()),
        canonical_probe
            .field_for_touch(&test_aspect_touch("status.value"))
            .map(|field| field.foundational_value())
    );
    assert_eq!(
        delegated_probe
            .field_for_touch(&test_aspect_touch("title.value"))
            .map(|field| field.foundational_value()),
        canonical_probe
            .field_for_touch(&test_aspect_touch("title.value"))
            .map(|field| field.foundational_value())
    );
    assert_eq!(
        delegated_probe
            .field_for_touch(&test_aspect_touch("identity.id"))
            .map(|field| field.foundational_value()),
        canonical_probe
            .field_for_touch(&test_aspect_touch("identity.id"))
            .map(|field| field.foundational_value())
    );
}

#[test]
fn workspace_delete_existing_verified_delegates_to_authoritative_mutation_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    const WORKSPACE_NAME: &str = "intent-admission-delete-existing-verified";

    let mut delegated_workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-delete-existing-verified-delegated")
        .expect("workspace should open");
    let delegated_binding =
        seeded_existing_binding(request_execution, &mut delegated_workspace, WORKSPACE_NAME);
    let delegated = delegated_workspace
        .delete_existing_verified(
            request_execution,
            delegated_binding.clone(),
            |task| {
                task.set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Seed title"),
                )
            },
            |delete| delete.touch(test_aspect_touch("title.value")),
        )
        .expect("legacy verified delete should execute");

    let mut canonical_workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-delete-existing-verified-canonical")
        .expect("workspace should open");
    let canonical_binding =
        seeded_existing_binding(request_execution, &mut canonical_workspace, WORKSPACE_NAME);
    let asserted_aspects = WorthQueryAspectMutationBuilder::new()
        .set_aspect(
            test_aspect_touch("title.value"),
            test_authored_string_aspect_value("Seed title"),
        )
        .finish_existing_truth_verification_aspects("backend-verified existing-truth delete")
        .expect("asserted aspects should build");
    let command = WorthQueryDeleteMutationBuilder::new()
        .touch(test_aspect_touch("title.value"))
        .build_delete_existing_verified(canonical_binding, asserted_aspects)
        .expect("verified delete command should build");
    let canonical = canonical_workspace
        .write_intent(command)
        .execute(request_execution)
        .expect("canonical write intent should execute");
    let delegated_assumptions = delegated
        .verified_assumption_set()
        .expect("delegated delete should retain assumption set");
    let canonical_assumptions = canonical
        .verified_assumption_set()
        .expect("canonical delete should retain assumption set");

    assert_eq!(delegated_assumptions, canonical_assumptions);
    assert_eq!(
        canonical
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteScalarWrite)
    );
}
