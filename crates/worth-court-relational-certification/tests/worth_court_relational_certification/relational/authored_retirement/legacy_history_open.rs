//! Current owned publication restoration and the explicit retired-frame refusal.
use super::*;
use worth_relational::facade::durability::{
    RecoveryFailureClass, RelationalNativeCheckpoint, RelationalRecoveryStop,
};

#[test]
fn retired_checkpoint_refuses_before_reading_legacy_publication_meaning() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-testkit-relational/assets/fixtures/history_pre_historical_selection_chk20.chk20"
    ));
    assert!(bytes.starts_with(b"WORTH-REL-CHK20\n"));
    let mut runtime = registration::runtime();
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes(bytes.to_vec());
    let stop = runtime
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap_err();
    assert!(matches!(stop, RelationalRecoveryStop::Durability(failure)
        if failure.class == RecoveryFailureClass::UnsupportedLegacySemantics));
    assert_eq!(
        before.descriptor(),
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor()
    );
}

#[test]
fn current_checkpoint_readmits_original_owned_publication_and_live_members() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&host);
    let (checkpoint, source, output, meaning) = capture_publication(execution);
    let mut runtime = registration::runtime();
    let expected = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
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

fn capture_publication(
    execution: &ExecutionRequest<'_, '_>,
) -> (
    RelationalNativeCheckpoint,
    EntityId,
    EntityId,
    RequiredOutputPublicationMeaning,
) {
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
        source_partition: worth_foundational::facade::CanonicalDigestId::new([0; 32]),
        source_epoch: worth_foundational::facade::CanonicalDigestId::new([1; 32]),
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
    (checkpoint, source, output, meaning)
}
