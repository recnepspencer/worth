//! Abandoned client keys burn only D7 identity frontiers in the Embedded image.
//! Fixture adaptation (C2.3): the original captured and restored on runtimes
//! bound to the retired in-crate test medium. Every capture here is Embedded
//! and the restore reads only those bytes, so the claim has no physical-medium
//! content and runs on the owner's Memory runtime (C2 placement rule).
use super::*;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointFieldsFixturePort as _;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointOwnerPort as _;

#[test]
fn unpublished_client_keys_leave_checkpoint_dictionary_equal_and_published_keys_restore() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "published-before-abandonment");
    let capture = || {
        runtime
            .durability_authority()
            .native_checkpoint_with_policy(
                execution,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &[],
                    source: None,
                },
            )
            .unwrap()
    };
    let before = capture();
    let mut candidate = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    candidate
        .push_batch(
            batch_create("unpublished-client-key"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, candidate)
        .unwrap();
    // Preparation retains the key in candidate custody until durable publication.
    assert!(!runtime
        .services
        .symbols
        .with_read(|symbols| symbols.contains("unpublished-client-key")));
    drop(candidate);
    let after = capture();
    let decode = |checkpoint: &crate::durability::data::RelationalNativeCheckpoint| {
        crate::durability::log::native_checkpoint_codec::decode_expanded_checkpoint(
            execution,
            checkpoint.bytes(),
            crate::tests::support::durability::captured_members_source(checkpoint),
        )
        .unwrap()
        .checkpoint
    };
    let before = decode(&before);
    let mut after = decode(&after);
    // D7 retains abandoned identity frontiers. Decoding authenticates both
    // dictionary indices; their expanded meaning and commits stay unchanged.
    assert!(after.symbol_identity_frontier_for_test() > before.symbol_identity_frontier_for_test());
    assert_eq!(before.symbol_table, after.symbol_table);
    assert_eq!(before.envelopes(), after.envelopes());
    assert_eq!(before.partition_images, after.partition_images);
    assert_eq!(before.aspect_contracts, after.aspect_contracts);
    assert_eq!(
        before.branch_root_schema_images_for_test(),
        after.branch_root_schema_images_for_test()
    );
    assert_eq!(before.index_definitions, after.index_definitions);
    assert_eq!(before.lineage, after.lineage);
    // Only D7's burned symbol and record identities may differ. Full equality
    // also protects root images, coverage, schemas, and derived artifacts.
    for checkpoint in [&before, &after] {
        let directory = checkpoint.original_canonical_directory().unwrap();
        assert_eq!(
            directory.manifest.record_identity,
            paged_roots(checkpoint.record_identity_for_test())
        );
        assert_eq!(
            directory.manifest.symbol_identity_frontier,
            checkpoint.symbol_identity_frontier_for_test()
        );
    }
    *after.symbol_identity_frontier_mut_for_test() = before.symbol_identity_frontier_for_test();
    assert_eq!(
        before.record_identity_for_test().schema_version,
        after.record_identity_for_test().schema_version
    );
    // Fixture adaptation: the image constructor now takes its optional retained
    // memory charge; a decoded comparison image carries none, and image
    // equality never compares that charge.
    *after.record_identity_mut_for_test() =
        crate::durability::data::DurableRecordIdentityState::current_with_memory(
            before
                .record_identity_for_test()
                .generation_high_water
                .clone(),
            before.record_identity_for_test().reusable_slots.clone(),
            before.record_identity_for_test().append_frontiers.clone(),
            after
                .record_identity_for_test()
                .pending_reservations
                .clone(),
            None,
        );
    let mut original_directory = after
        .canonical_directory_mut_for_test()
        .take()
        .unwrap()
        .into_original();
    let changed_directory = std::sync::Arc::get_mut(&mut original_directory)
        .expect("decoded checkpoint owns its original directory uniquely");
    let mut changed_manifest = changed_directory.manifest.ordinary_owned_draft();
    changed_manifest.record_identity = paged_roots(after.record_identity_for_test());
    changed_manifest.symbol_identity_frontier = after.symbol_identity_frontier_for_test();
    changed_directory.manifest = changed_manifest.seal_ordinary();
    *after.canonical_directory_mut_for_test() = Some(
        crate::durability::data::NativeCheckpointDirectoryEdge::new(original_directory),
    );
    assert_eq!(before, after);

    let committed = create_entity_outcome(execution, &runtime, "published-after-abandonment");
    let published = capture();
    let mut recovered = runtime_with_test_schema();
    let restored = recovered
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            &published,
            crate::tests::support::durability::captured_members_source(&published),
        )
        .unwrap();
    assert_eq!(restored.latest_commit, Some(committed.commit.clone()));
    recovered
        .services
        .symbols
        .demand_text(execution, "published-after-abandonment")
        .unwrap();
    recovered
        .services
        .symbols
        .demand_text(execution, "unpublished-client-key")
        .unwrap();
    assert!(recovered
        .services
        .symbols
        .with_read(|symbols| symbols.contains("published-after-abandonment")));
    assert!(!recovered
        .services
        .symbols
        .with_read(|symbols| symbols.contains("unpublished-client-key")));
    create_entity_outcome(execution, &recovered, "editable-after-dictionary-recovery");
}

// Fixture adaptation (C2.4): the original used `checkpoint_and_recover_with`
// over the retired in-crate test medium. Symbol-frontier preservation across
// checkpoint recovery has no physical-medium content, so it runs over an
// Embedded native checkpoint and restore on the Memory runtime.
#[test]
fn checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers() {
    use crate::tests::support::*;
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema();
    create_entity(&execution, &runtime, "symbol-frontier-seed");
    let mut abandoned = runtime.services.symbols.begin_pending_meanings();
    let symbol = abandoned.reserve("abandoned-before-checkpoint").unwrap();
    drop(abandoned);
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(
            &execution,
            &native,
            crate::tests::support::durability::captured_members_source(&native),
        )
        .unwrap();
    let mut candidate = recovered.services.symbols.begin_pending_meanings();
    let fresh = loop {
        match candidate.reserve("abandoned-before-checkpoint") {
            Ok(symbol) => break symbol,
            Err(crate::runtime::SymbolReservationDenial::ColdBucket(text)) => {
                drop(candidate);
                recovered
                    .services
                    .symbols
                    .demand_text(&execution, text.as_str())
                    .unwrap();
                candidate = recovered.services.symbols.begin_pending_meanings();
            }
            other => panic!("recovered symbol reservation failed: {other:?}"),
        }
    };
    assert!(fresh.0 > symbol.0);
    assert_eq!(
        recovered
            .services
            .symbols
            .with_read(|symbols| symbols.resolve(symbol).map(str::to_owned)),
        None
    );
    let meanings = candidate.interner().snapshot();
    {
        let binding = candidate.binding();
        candidate.seal_for_publication(&meanings, &binding, [])
    }
    .expect("fixture meanings agree with the candidate")
    .publish()
    .unwrap();
}

/// CHK24 manifests retain the identity tables' content-addressed roots and roster,
/// which are a pure function of the resident image.
fn paged_roots(
    image: &crate::durability::data::DurableRecordIdentityState,
) -> worth_relational_storage::internal_port::storage::data::record_identity_pages::PagedRecordIdentity{
    let mut sink = worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::CollectedMembers::default();
    worth_relational_storage::internal_port::storage::data::record_identity_pages::encode_record_identity(image, &mut sink)
        .unwrap()
}
