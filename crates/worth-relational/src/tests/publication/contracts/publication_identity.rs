use crate::capabilities::DurabilityRead;
use crate::tests::support::*;

#[test]
fn publication_rejects_envelope_identity_drift_before_any_effect() {
    let runtime = runtime_with_test_schema();
    let commit = create_entity_outcome(&runtime, "publication-identity");
    let receipt = commit.commit.clone();
    let envelope = commit.publication().envelope.clone();
    let identity = runtime
        .branch_identity(&BranchId("main".to_owned()))
        .expect("main identity");
    let binding = runtime
        .admitted_branch_basis_for_identity(&identity)
        .expect("main binding");
    let before_cells = runtime.history.branch_cells_snapshot();
    let before_catalog = runtime.history.catalog_len();

    let mut mismatched_commit = envelope.as_ref().clone();
    mismatched_commit.commit.commit_id.0 += 100;
    let error = runtime
        .mvcc_publication_authority()
        .validate_versioned_publication(receipt.commit_id, &receipt, &binding, &mismatched_commit)
        .expect_err("envelope commit identity drift must be denied");
    assert!(error.contains("envelope commit identity mismatch"));
    assert_eq!(runtime.history.branch_cells_snapshot(), before_cells);
    assert_eq!(runtime.history.catalog_len(), before_catalog);

    let mut mismatched_branch = envelope.as_ref().clone();
    mismatched_branch.branch_context = BranchId("other".to_owned());
    let error = runtime
        .mvcc_publication_authority()
        .validate_versioned_publication(receipt.commit_id, &receipt, &binding, &mismatched_branch)
        .expect_err("envelope branch context drift must be denied");
    assert!(error.contains("envelope branch context mismatch"));
    assert_eq!(runtime.history.branch_cells_snapshot(), before_cells);
    assert_eq!(runtime.history.catalog_len(), before_catalog);
}

#[test]
fn prepared_publication_rejects_an_existing_envelope_before_any_effect() {
    let runtime = runtime_with_test_schema();
    let committed = create_entity_outcome(&runtime, "duplicate-catalog-envelope");
    let identity = runtime
        .branch_identity(&BranchId("main".to_owned()))
        .expect("main identity");
    let binding = runtime
        .admitted_branch_basis_for_identity(&identity)
        .expect("main binding");
    let selected_branch_state = runtime
        .selected_branch_state(&binding)
        .expect("current binding selects its committed root");
    let schema_registry = runtime.config.schema.registry.clone();
    let before_cells = runtime.history.branch_cells_snapshot();
    let before_catalog = runtime.history.catalog_len();
    let preparation_runtime = runtime.preparation_runtime_snapshot();
    let delta = crate::storage::RelationalPublishedPartitionDelta::from_committed_partitions(
        &std::collections::BTreeMap::new(),
    );

    let error = preparation_runtime
        .mvcc_publication_authority()
        .prepare_commit(
            committed.commit.commit_id,
            committed.commit.clone(),
            &binding,
            &selected_branch_state,
            delta,
            std::sync::Arc::clone(&committed.publication().envelope),
            &schema_registry,
        )
        .err()
        .expect("an existing envelope must fail the real prepared-publication court");

    assert!(error.contains("DuplicateCommit"));
    assert_eq!(runtime.history.branch_cells_snapshot(), before_cells);
    assert_eq!(runtime.history.catalog_len(), before_catalog);
}

#[test]
fn root_capture_sabotage_leaves_storage_index_history_and_reference_unchanged() {
    let runtime = runtime_with_test_schema();
    let committed = create_entity_outcome(&runtime, "root-capture-sabotage");
    let current_receipt = committed.commit.clone();
    let identity = runtime
        .branch_identity(&BranchId("main".to_owned()))
        .expect("main identity");
    let binding = runtime
        .admitted_branch_basis_for_identity(&identity)
        .expect("main binding");
    let commit_id = runtime.history.preview_next_commit_id();
    let version_id = runtime.history.preview_next_version_id();
    let mut future_envelope = committed.publication().envelope.as_ref().clone();
    future_envelope.commit = crate::history::data::RelationalCommitReceipt {
        commit_id,
        version_id,
        branch_id: BranchId("main".to_owned()),
        parents: vec![current_receipt.commit_id],
    };

    let partition_id = *runtime
        .partitions
        .partition_ids()
        .first()
        .expect("the committed entity installs one partition");
    let mut malformed_partition = runtime
        .partitions
        .partition(partition_id)
        .expect("committed partition")
        .as_ref()
        .clone();
    malformed_partition.entity_arena.aspect_versions[0]
        .insert(crate::symbols::data::Symbol(u32::MAX), 1);
    let mut journal = crate::storage::overlay::PartitionMutationJournal::default();
    journal.entity_slots.insert(0);
    let delta = crate::storage::RelationalPublishedPartitionDelta::from_committed_partitions(
        &std::collections::BTreeMap::from([(partition_id, (malformed_partition, journal))]),
    );

    let before_storage = runtime
        .acquire_partition_edition()
        .entries()
        .map(|(id, partition)| {
            (
                id,
                partition
                    .authoritative_content_digest(&runtime.services.symbols.interner_snapshot())
                    .expect("installed storage symbols resolve"),
            )
        })
        .collect::<Vec<_>>();
    let before_index = runtime.indexes.with_unique_index(Clone::clone);
    let before_cells = runtime.history.branch_cells_snapshot();
    let before_catalog = runtime.history.catalog_len();
    let before_envelopes = runtime.history.recorded_commit_envelope_count();
    let before_patch_index = runtime.history.recorded_patch_position_count();
    let before_sequences = runtime.history.reserved_sequence_floors();

    let selected_branch_state = runtime
        .selected_branch_state(&binding)
        .expect("current binding must select a root");
    let schema_registry = runtime.config.schema.registry.clone();
    let preparation_runtime = runtime.preparation_runtime_snapshot();
    let error = preparation_runtime
        .mvcc_publication_authority()
        .prepare_commit(
            commit_id,
            future_envelope.commit.clone(),
            &binding,
            &selected_branch_state,
            delta,
            std::sync::Arc::new(future_envelope),
            &schema_registry,
        )
        .err()
        .expect("the unresolved owner symbol sabotages root capture");
    assert!(error.contains("UnresolvedContentSymbol"));
    let after_storage = runtime
        .acquire_partition_edition()
        .entries()
        .map(|(id, partition)| {
            (
                id,
                partition
                    .authoritative_content_digest(&runtime.services.symbols.interner_snapshot())
                    .expect("failed preparation cannot corrupt installed storage"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(after_storage, before_storage);
    assert_eq!(
        runtime.indexes.with_unique_index(Clone::clone),
        before_index
    );
    assert_eq!(runtime.history.branch_cells_snapshot(), before_cells);
    assert_eq!(runtime.history.catalog_len(), before_catalog);
    assert_eq!(
        runtime.history.recorded_commit_envelope_count(),
        before_envelopes
    );
    assert_eq!(
        runtime.history.recorded_patch_position_count(),
        before_patch_index
    );
    assert_eq!(runtime.history.reserved_sequence_floors(), before_sequences);
}

#[test]
fn production_commit_root_capture_sabotage_precedes_durable_append_and_all_effects() {
    let runtime = runtime_with_test_schema();
    create_entity_outcome(&runtime, "root-capture-production-anchor");
    let before_storage = runtime
        .acquire_partition_edition()
        .entries()
        .map(|(id, partition)| {
            (
                id,
                partition
                    .authoritative_content_digest(&runtime.services.symbols.interner_snapshot())
                    .expect("installed storage symbols resolve"),
            )
        })
        .collect::<Vec<_>>();
    let before_index = runtime.indexes.with_unique_index(Clone::clone);
    let before_cells = runtime.history.branch_cells_snapshot();
    let before_catalog = runtime.history.catalog_len();
    let before_durable = runtime.durable_log().len();
    let before_envelopes = runtime.history.recorded_commit_envelope_count();
    let before_patch_index = runtime.history.recorded_patch_position_count();
    let before_sequences = runtime.history.reserved_sequence_floors();

    runtime.history.sabotage_next_root_capture();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("root-capture-production-sabotage"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let error = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect_err("the test-only root court must reject the real commit path");
    assert!(format!("{error:?}").contains("UnresolvedContentSymbol"));

    let after_storage = runtime
        .acquire_partition_edition()
        .entries()
        .map(|(id, partition)| {
            (
                id,
                partition
                    .authoritative_content_digest(&runtime.services.symbols.interner_snapshot())
                    .expect("failed publication cannot corrupt installed storage"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(after_storage, before_storage);
    assert_eq!(
        runtime.indexes.with_unique_index(Clone::clone),
        before_index
    );
    assert_eq!(runtime.history.branch_cells_snapshot(), before_cells);
    assert_eq!(runtime.history.catalog_len(), before_catalog);
    assert_eq!(runtime.durable_log().len(), before_durable);
    assert_eq!(
        runtime.history.recorded_commit_envelope_count(),
        before_envelopes
    );
    assert_eq!(
        runtime.history.recorded_patch_position_count(),
        before_patch_index
    );
    assert_eq!(runtime.history.reserved_sequence_floors(), before_sequences);
}

#[test]
fn new_partition_publication_reuses_every_prior_region() {
    let runtime = runtime_with_test_schema();
    create_entity_outcome(&runtime, "prior-main-region");
    let identity = runtime
        .branch_identity(&BranchId("main".to_owned()))
        .expect("main branch identity exists");
    let scope =
        crate::facade::inspection::RelationalMvccCostScope::capture(&runtime, vec![identity]);

    create_entity_in_partition(&runtime, "new-partition-region", PartitionId(29));
    let costs = runtime
        .observe_mvcc_cost(&scope)
        .expect("main sharing remains inspectable")
        .sharing_cost_delta();

    assert_eq!(costs.publication_touched_region_count, 1);
    assert_eq!(costs.publication_reused_region_count, 1);
}
