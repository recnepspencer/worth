//! Open the actual old writer's store through current Native storage admission.
use super::*;
#[test]
fn chk18_store_written_before_historical_selection_still_opens() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&host);
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-testkit-relational/assets/fixtures/history_pre_historical_selection.chk18"
    ));
    assert!(bytes.starts_with(b"WORTH-REL-CHK18\n"));
    let recorded: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-testkit-relational/assets/fixtures/history_pre_historical_selection.json"
    )))
    .unwrap();
    let source: EntityId = serde_json::from_value(recorded["source"].clone()).unwrap();
    let output: EntityId = serde_json::from_value(recorded["output"].clone()).unwrap();
    let meaning: RequiredOutputPublicationMeaning =
        serde_json::from_value(recorded["meaning"].clone()).unwrap();
    let mut runtime = registration::runtime();
    let expected = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes(bytes.to_vec());
    runtime
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .unwrap();
    let receipt = runtime
        .readmit_recovered_initial_schema_installation(expected)
        .unwrap();
    let (_, installed) = receipt.issue_application_publication_owner();
    let installed = installed.with_source_scope_owned_producers([("recorder".into(), KindId(1))]);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let publication = runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "recorder",
            &[1; 32],
            &meaning,
            &[RequiredOutputRoleTarget {
                role: "output".into(),
                entity_name: "node".into(),
                kind: KindId(1),
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Existing(output),
            }],
        )
        .unwrap();
    assert_eq!(
        publication
            .members()
            .map(|(_, _, _, id)| id)
            .collect::<Vec<_>>(),
        [output]
    );
    assert_eq!(publication.meaning(), &meaning);
    let mut read = begin_main_transaction(&runtime);
    assert!(read
        .read_entity(execution, source)
        .unwrap()
        .base()
        .is_some());
    assert!(read
        .read_entity(execution, output)
        .unwrap()
        .base()
        .is_some());
}
use worth_relational::facade::durability::RelationalNativeCheckpoint;
