//! Publication membership must survive out-of-order preparation reservations.
use super::*;
use crate::durability::data::RelationalCheckpointManifest;

fn publish_checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    current: &[RelationalDurableMemberLocator],
) -> crate::durability::data::RelationalNativeCheckpoint {
    let medium = attached_test_medium(runtime);
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: current,
                source: Some(durable_recovery_source(runtime).into_parts().cold_source),
            },
        )
        .unwrap();
    medium.file_checkpoint(native.clone());
    native.acknowledge_publication();
    native
}

#[test]
fn a_reserved_candidate_published_after_another_branch_checkpoint_keeps_its_pages() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "reservation-anchor");
    let other = create_branch_from_main(&runtime, "reservation-other");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("reserved-lower-page-id"),
            execution.allocation_policy(),
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    create_entity_in_partition_on_branch(
        execution,
        &runtime,
        "published-higher-page-id",
        PartitionId(2),
        other,
    );
    let first = publish_checkpoint(execution, &runtime, &[]);
    let current = first
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect::<Vec<_>>();
    let committed = runtime
        .publish_prepared_candidate(execution, candidate)
        .unwrap();
    let entity = changed_entities(&committed)[0];
    release_test_commit_snapshot(&runtime, &committed);
    let expected_node = runtime
        .lineage_access()
        .for_record(execution, entity)
        .unwrap()
        .unwrap();
    let expected_symbols = runtime.services.symbols.snapshot();
    let second = publish_checkpoint(execution, &runtime, &current);
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: RelationalCheckpointManifest,
    }
    let decoded: Input = rmp_serde::from_slice(
        second
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap();
    let mut reopened = durable_runtime_with_test_schema();
    let plan = medium_recovery_plan(
        execution,
        &runtime,
        &mut reopened,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    reopened
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    for (symbol, text) in expected_symbols.entries {
        reopened
            .services
            .symbols
            .demand_symbol_ids(execution, [symbol])
            .unwrap();
        assert_eq!(
            reopened
                .services
                .symbols
                .snapshot()
                .entries
                .iter()
                .find(|(id, _)| *id == symbol)
                .map(|(_, value)| value),
            Some(&text)
        );
    }
    let mut reader = test_owner_begin_transaction_for_main(&reopened);
    let read = reader.read_entity(execution, entity).unwrap();
    assert!(
        read.effective().is_some(),
        "the published record survives the checkpoint"
    );
    assert_eq!(
        reopened
            .lineage_access()
            .for_record(execution, entity)
            .unwrap(),
        Some(expected_node)
    );
    let (_, basis) = reopened
        .observe_branch(&reopened.main_branch_identity())
        .unwrap();
    basis
        .inner
        .root
        .demand_regions_full(execution, &reopened.services.symbols, [entity.partition_id])
        .unwrap();
    assert!(!decoded.manifest.lineage.is_empty());
}
