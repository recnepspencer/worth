use super::*;

#[test]
fn optional_native_contract_supports_clear() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let contract = optional_string_contract("note");
    let mut workspace =
        WorthQueryMemoryWorkspace::collection_with_native_contracts_for_initial_seed(
            "Task",
            [aspect("note.value", "note.value")],
            [contract.clone()],
            InvariantCatalog::default(),
            [],
            0,
        )
        .unwrap();
    let inserted = workspace
        .insert_aspects(
            request_execution,
            vec![
                WorthQueryAuthoredAspectMutation::new_set(touch("note.value"), text("remember"))
                    .unwrap(),
            ],
        )
        .unwrap();

    workspace
        .update_aspects(
            request_execution,
            inserted.deltas[0].entity_identity.clone(),
            vec![WorthQueryAuthoredAspectMutation::new_clear(touch("note.value")).unwrap()],
        )
        .unwrap();

    assert!(workspace.entities(request_execution).unwrap()[0]
        .aspect_value(contract.key())
        .is_none());
    assert!(workspace.entities(request_execution).unwrap()[0]
        .scalar_value_at(&field_path("note.value"))
        .is_none());
}

#[test]
fn unrelated_installed_contract_does_not_require_a_local_mapping() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let title = optional_string_contract("title");
    let unrelated = optional_string_contract("note");
    let mut workspace =
        WorthQueryMemoryWorkspace::collection_with_native_contracts_for_initial_seed(
            "Task",
            [aspect("title.value", "title.value")],
            [title.clone(), unrelated],
            InvariantCatalog::default(),
            [],
            0,
        )
        .expect("an unrelated installed contract must not require a local physical mapping");

    workspace
        .insert_aspects(
            request_execution,
            vec![WorthQueryAuthoredAspectMutation::new_set(
                touch("title.value"),
                text("Mapped title"),
            )
            .unwrap()],
        )
        .expect("the locally mapped contract should remain usable");

    assert_eq!(
        workspace.entities(request_execution).unwrap()[0].aspect_value(title.key()),
        Some(&text("Mapped title"))
    );
}

#[test]
fn physical_mapping_without_its_native_contract_is_denied() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        WorthQueryMemoryWorkspace::collection_with_native_contracts_for_initial_seed(
            "Task",
            [aspect("title.value", "title.value")],
            [optional_string_contract("note")],
            InvariantCatalog::default(),
            [],
            0,
        )
        .expect("an unrelated installed contract may coexist with the collection");

    let denial = workspace
        .insert_aspects(
            request_execution,
            vec![WorthQueryAuthoredAspectMutation::new_set(
                touch("title.value"),
                text("Uncontracted title"),
            )
            .unwrap()],
        )
        .expect_err("a physical mapping must not become authority without its contract");

    assert!(denial.message().contains("no Foundational contract"));
}

#[test]
fn native_contract_preserves_struct_field_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let contract = summary_contract();
    let mut workspace =
        WorthQueryMemoryWorkspace::collection_with_native_contracts_for_initial_seed(
            "Task",
            [
                aspect("summary.title", "summary.title"),
                aspect("summary.status", "summary.status"),
            ],
            [contract.clone()],
            InvariantCatalog::default(),
            [],
            0,
        )
        .unwrap();
    let summary = StructAspectValue::new([
        (FieldKey::new("title").unwrap(), text("Native summary")),
        (FieldKey::new("status").unwrap(), text("open")),
    ])
    .unwrap();
    let inserted = workspace
        .insert_aspects(
            request_execution,
            vec![WorthQueryAuthoredAspectMutation::new_set(
                touch("summary"),
                ContractValidationInput::Struct(summary),
            )
            .unwrap()],
        )
        .unwrap();

    workspace
        .update_aspects(
            request_execution,
            inserted.deltas[0].entity_identity.clone(),
            vec![WorthQueryAuthoredAspectMutation::new_clear(touch("summary.status")).unwrap()],
        )
        .unwrap();

    let entities = workspace.entities(request_execution).unwrap();
    let entity = &entities[0];
    let summary = entity.struct_aspect_value(contract.key()).unwrap();
    assert!(summary.get(&FieldKey::new("status").unwrap()).is_none());
    assert_eq!(
        summary.get(&FieldKey::new("title").unwrap()),
        Some(&text("Native summary"))
    );
    assert!(entity
        .scalar_value_at(&field_path("summary.status"))
        .is_none());
}
