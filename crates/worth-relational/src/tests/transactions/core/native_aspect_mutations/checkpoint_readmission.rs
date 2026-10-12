use super::*;

#[test]
fn native_patch_state_survives_checkpoint_readmission() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let note = optional_string_contract("note");
    let fixture = AspectSchemaFixture {
        entity_aspects: vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: field_key("note"),
            },
            contract: note.clone(),
        }],
        ..AspectSchemaFixture::default()
    };
    let runtime = fixture.build_runtime();
    let created = commit_entity_create(execution, &runtime, whole_set(&note, text("durable")));
    let entity = changed_entities(&created)[0];
    let recovery_fixture = fixture.clone();
    let (_, recovered) = checkpoint_and_recover_with(execution, &runtime, move || {
        recovery_fixture.build_runtime()
    });
    let read = recovered
        .read_truth()
        .read_version(execution, created.version_id);
    assert!(read
        .get_entity(entity)
        .unwrap()
        .authoritative_aspect_state
        .as_ref()
        .unwrap()
        .get(note.key())
        .is_some());
}
