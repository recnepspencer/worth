//! Real native payload custody through current checkpoint and post-checkpoint tail.
use super::*;
use crate::capabilities::DurabilityRead;
use crate::durability::data::{RecoveryVerificationMode, RelationalRecoveryStop};
use crate::identity::data::{EntityId, VersionId};
use crate::mvcc::RelationalTransactionIntent;
use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{AspectValue, InternedString};
const JSON: &str = r#"{"actor":"native","request":{"input":"exact logical content"}}"#;

fn commit_text(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
) -> Result<CommitResult, TransactionCommitError> {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let external = ExecutionImmutableBytes::from_external_bytes(Arc::new(JSON.as_bytes().into()));
    let text = runtime
        .admit_field_text(execution, &basis, &external)
        .unwrap();
    let mut batch = batch_create("native-managed-text");
    let MutationIntent::Create(CreateIntent::Entity(entity)) = &mut batch.intents[0] else {
        panic!("real entity input")
    };
    entity.fields = AspectFieldPatch::from_locator(
        crate::transactions::data::planned_single_field_locator(
            aspect_key("name"),
            field_key("name"),
        ),
        text.field_value_for(&basis).unwrap(),
    );
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    transaction.commit(execution, runtime)
}
fn read_exact(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    entity: EntityId,
    version: VersionId,
) -> crate::runtime::RelationalPinnedRead {
    let record = runtime
        .read_truth()
        .visible_entity_at_version(execution, entity, version)
        .unwrap()
        .unwrap();
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let value = crate::visibility::materialization::read_records::entity_query_locus_value(
        &record, &locator,
    )
    .unwrap();
    let AspectValue::String(InternedString::Symbol(symbol)) = value else {
        panic!("managed text must remain a symbol, never Raw")
    };
    let snapshot = runtime.visibility_authority().snapshot();
    let pinned = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[], execution)
        .unwrap();
    assert_eq!(
        pinned
            .symbols()
            .resolve(crate::symbols::data::Symbol(symbol.0)),
        Some(JSON)
    );
    assert!(pinned
        .symbols()
        .retained_text(crate::symbols::data::Symbol(symbol.0))
        .unwrap()
        .charged_payload_bytes()
        .is_some());
    pinned
}

#[test]
fn native_field_text_checkpoint_then_tail_recovers_exact_json_after_original_owners_drop() {
    let original = test_execution_lease();
    let execution = &ExecutionRequest::leased(&original);
    let source = persisted_runtime_with_test_schema();
    let baseline = create_entity_outcome(execution, &source, "checkpoint-before-managed-text");
    release_test_commit_snapshot(&source, &baseline);
    let recovery_config = source.config(); // no managed text exists here
    source.durability_authority().checkpoint(execution).unwrap();
    let committed = commit_text(execution, &source).unwrap();
    let entity = changed_entities(&committed)[0];
    let version = committed.version_id;
    release_test_commit_snapshot(&source, &committed);
    assert_eq!(
        source.durable_store().unwrap().segments.len(),
        1,
        "same current segment accepts tail; no per-commit rotation"
    );
    drop(committed);
    drop(baseline);
    drop(source);
    drop(original);

    let recovery_lease = test_execution_lease();
    let execution = &ExecutionRequest::leased(&recovery_lease);
    let mut restored = crate::runtime::RelationalRuntime::new(recovery_config);
    let plan = restored
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    assert_eq!(plan.checkpoint.as_ref().unwrap().envelopes.len(), 1);
    assert_eq!(plan.tail_log.len(), 1);
    let outcome = restored
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(outcome.coverage.replayed_tail_commits, 1);
    let pinned = read_exact(execution, &restored, entity, version);
    let key = pinned.symbols().symbol("native-managed-text").unwrap();
    assert_eq!(pinned.symbols().resolve(key), Some("native-managed-text"));
    assert_eq!(
        pinned
            .symbols()
            .retained_text(key)
            .unwrap()
            .charged_payload_bytes(),
        None,
        "ordinary client-key custody stays System"
    );
    create_entity(execution, &restored, "after-restored-managed-text");
    restored
        .services
        .symbols
        .with_read(|symbols| assert_eq!(symbols.resolve(key), Some("native-managed-text")));
    drop(restored);
    assert_eq!(
        pinned
            .symbols()
            .symbol(JSON)
            .and_then(|symbol| pinned.symbols().resolve(symbol)),
        Some(JSON)
    );
}

#[test]
fn native_field_text_checkpoint_readmission_denial_keeps_bytes_and_target_root_intact() {
    let lease = test_execution_lease();
    let execution = &ExecutionRequest::leased(&lease);
    let source = persisted_runtime_with_test_schema();
    let committed = commit_text(execution, &source).unwrap();
    release_test_commit_snapshot(&source, &committed);
    let native = source
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let before = native.bytes().to_vec(); // independent test observation of external evidence
    let mut target = runtime_with_test_schema();
    let identity = target.main_branch_identity();
    let original = target.observe_branch(&identity).unwrap().0;
    let limited = lease
        .child(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(1).unwrap(),
                    1,
                    Some(1_000_000),
                ),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let error = target
        .durability_recovery()
        .restore_native_checkpoint(&ExecutionRequest::leased(&limited), &native)
        .unwrap_err();
    assert!(
        matches!(error,RelationalRecoveryStop::Request(crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { requested, admitted:1, .. }) if requested == JSON.len() as u64)
    );
    assert_eq!(native.bytes(), before);
    assert_eq!(target.observe_branch(&identity).unwrap().0, original);
    target
        .durability_recovery()
        .restore_native_checkpoint(execution, &native)
        .unwrap();
    read_exact(
        execution,
        &target,
        changed_entities(&committed)[0],
        committed.version_id,
    );
}

#[test]
fn native_field_text_configuration_requires_current_explicit_readmission() {
    let lease = test_execution_lease();
    let execution = &ExecutionRequest::leased(&lease);
    let source = runtime_with_test_schema();
    let (_, basis) = source
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let input = ExecutionImmutableBytes::from_external_bytes(Arc::new(JSON.as_bytes().into()));
    let field = source.admit_field_text(execution, &basis, &input).unwrap();
    let snapshot = source.config().identity.symbol_table;
    let wire = rmp_serde::to_vec_named(&snapshot).unwrap();
    assert!(rmp_serde::from_slice::<crate::symbols::data::SymbolTableSnapshot>(&wire).is_err());
    drop((snapshot, field, source));
    let admitted =
        crate::symbols::data::SymbolTableSnapshot::readmit_current(execution, &wire).unwrap();
    let managed = admitted
        .entries
        .iter()
        .find(|(_, text)| text.as_str() == JSON)
        .unwrap();
    assert_eq!(managed.1.charged_payload_bytes(), Some(JSON.len() as u64));
    let mut config = runtime_with_test_schema().config();
    config.identity.symbol_table = admitted;
    let restored = crate::runtime::RelationalRuntime::new(config);
    restored.services.symbols.with_read(|symbols| {
        assert_eq!(symbols.resolve(symbols.symbol(JSON).unwrap()), Some(JSON))
    });
}

#[test]
fn native_field_text_prior_checkpoint_format_refuses_before_managed_text() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let source = persisted_runtime_with_test_schema();
    let committed = commit_text(execution, &source).unwrap();
    release_test_commit_snapshot(&source, &committed);
    let native = source
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let unframed = native
        .bytes()
        .strip_prefix(crate::durability::log::native_file_codec::CHECKPOINT_MAGIC)
        .unwrap();
    let error = crate::durability::log::native_file_codec::decode_checkpoint(execution, unframed)
        .unwrap_err();
    assert!(
        matches!(error,RelationalRecoveryStop::Durability(ref error) if error.class == RecoveryFailureClass::UnsupportedLegacySemantics)
    );
    assert!(
        crate::durability::log::native_file_codec::decode_checkpoint(execution, native.bytes())
            .is_ok()
    );
}

#[test]
fn native_field_text_deferred_settlement_keeps_prepared_bytes_after_request_drop() {
    let lease = test_execution_lease();
    let execution = &ExecutionRequest::leased(&lease);
    let runtime = persisted_runtime_with_test_schema();
    let baseline = create_entity_outcome(execution, &runtime, "before-manifest-failure");
    release_test_commit_snapshot(&runtime, &baseline);
    let store = runtime.durable_store().unwrap();
    let root = std::fs::canonicalize(&store.layout.root_path).unwrap();
    let obstruction =
        crate::durability::log::local_store::manifest_path(&store.layout).with_extension("tmp");
    assert_eq!(obstruction.parent().unwrap().canonicalize().unwrap(), root);
    assert!(!obstruction.exists());
    std::fs::create_dir(&obstruction).unwrap();
    let failure = commit_text(execution, &runtime).unwrap_err();
    let settlement = failure.deferred_settlement().unwrap().clone();
    let entity = changed_entities(settlement.performed_result())[0];
    let version = settlement.performed_result().version_id;
    let commit = settlement.commit().commit_id;
    let segment = &store.segments[0].path;
    assert_eq!(
        runtime.durable_log().len(),
        1,
        "failed manifest did not publish logical append"
    );
    let manifest_path = crate::durability::log::local_store::manifest_path(&store.layout);
    assert_eq!(
        crate::durability::log::native_file_codec::read_store_manifest_file(&manifest_path)
            .unwrap()
            .store
            .segments[0]
            .commit_count,
        1
    );
    let inventory =
        crate::durability::log::native_file_codec::read_segment_inventory(segment).unwrap();
    assert_eq!(inventory, vec![baseline.commit.commit_id, commit]);
    let frames =
        crate::durability::log::local_store::read_segment_entries(execution, segment).unwrap();
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[1].envelope().commit.commit_id, commit);
    let position = frames[1].position();
    assert!(position > frames[0].position());
    let published_bytes = std::fs::read(segment).unwrap();
    drop((frames, failure, baseline));
    drop(lease);
    // Remove only the empty directory this fixture created, beneath its verified root.
    assert_eq!(obstruction.canonicalize().unwrap().parent().unwrap(), root);
    std::fs::remove_dir(&obstruction).unwrap();
    let receipt = runtime
        .repair_deferred_publication_settlement(&settlement)
        .unwrap();
    assert_eq!(receipt.commit_id, commit);
    assert_eq!(runtime.durable_log().len(), 2);
    let repaired_store = runtime.durable_store().unwrap();
    assert_eq!(repaired_store.segments.len(), 1);
    assert_eq!(repaired_store.segments[0].commit_count, inventory.len());
    assert_eq!(
        crate::durability::log::native_file_codec::read_store_manifest_file(&manifest_path)
            .unwrap()
            .store
            .segments[0]
            .commit_count,
        inventory.len()
    );
    assert_eq!(
        runtime
            .repair_deferred_publication_settlement(&settlement)
            .unwrap(),
        receipt
    );
    assert_eq!(runtime.durable_log().len(), 2);
    assert_eq!(std::fs::read(segment).unwrap(), published_bytes);
    assert_eq!(
        crate::durability::log::native_file_codec::read_segment_inventory(segment).unwrap(),
        inventory
    );
    let read_lease = test_execution_lease();
    let read_execution = ExecutionRequest::leased(&read_lease);
    let repaired =
        crate::durability::log::local_store::read_segment_entries(&read_execution, segment)
            .unwrap();
    assert_eq!(repaired.len(), 2);
    assert_eq!(repaired[1].position(), position);
    assert_eq!(repaired[1].envelope().commit.commit_id, commit);
    read_exact(&read_execution, &runtime, entity, version);
}

mod missing_dictionary;
