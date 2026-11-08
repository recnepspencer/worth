//! Record a real owned-publication store with the pre-HistoricalSelection writer.
use super::*;
/// Run from the pre-change checkout; the compatibility court only reads these assets.
#[test]
#[ignore = "fixture recorder: run explicitly in the pre-HistoricalSelection checkout"]
fn record_pre_historical_selection_chk20_store() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&host);
    let mut runtime = registration::runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let installed = installed.with_source_scope_owned_producers([("recorder".into(), KindId(1))]);
    let authored = targets(&["legacy-source"]);
    let mut tx = begin_main_transaction(&runtime);
    tx.push_batch(creates("author", &authored), execution.allocation_policy())
        .unwrap();
    let born = tx.commit(execution, &runtime).unwrap();
    let source = born.created_entity(&authored[0]).unwrap();
    release_test_commit_snapshot(&runtime, &born);
    let meaning = RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap(),
        family: "legacy-history".into(),
        source_scope: source,
        source_partition: [0; 32],
        source_epoch: [1; 32],
    };
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let absence = runtime
        .readmit_required_output_route_absence(execution, &installed, &basis, &meaning)
        .unwrap();
    let generated = targets(&["legacy-output"]);
    let publication = publish(
        &runtime,
        execution,
        &installed,
        creates("recorder", &generated),
        RequiredOutputPublication::Birth {
            admission: owner.admit_required_output("recorder", [1; 32]),
            absence,
            meaning: meaning.clone(),
            roles: vec![RequiredOutputRoleTarget {
                role: "output".into(),
                entity_name: "node".into(),
                kind: KindId(1),
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Created(generated[0].clone()),
            }],
        },
    );
    let output = publication.created_entity(&generated[0]).unwrap();
    release_test_commit_snapshot(&runtime, &publication);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../worth-testkit-relational/assets/fixtures/history_pre_historical_selection_chk20");
    std::fs::write(base.with_extension("chk20"), checkpoint.bytes()).unwrap();
    let metadata = serde_json::json!({"source":source,"output":output,"meaning":meaning});
    std::fs::write(
        base.with_extension("json"),
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();
}
