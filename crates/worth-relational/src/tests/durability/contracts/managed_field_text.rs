//! Real native payload custody through current checkpoint and post-checkpoint tail.
use super::*;
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
    let source = durable_runtime_with_test_schema();
    let baseline = create_entity_outcome(execution, &source, "checkpoint-before-managed-text");
    release_test_commit_snapshot(&source, &baseline);
    let recovery_config = source.config(); // no managed text exists here
    crate::tests::support::checkpoint_durably(execution, &source).unwrap();
    let committed = commit_text(execution, &source).unwrap();
    let entity = changed_entities(&committed)[0];
    let version = committed.version_id;
    release_test_commit_snapshot(&source, &committed);
    let medium = attached_test_medium(&source);
    drop(committed);
    drop(baseline);
    drop(source);
    drop(original);

    let recovery_lease = test_execution_lease();
    let execution = &ExecutionRequest::leased(&recovery_lease);
    let mut restored = crate::runtime::RelationalRuntime::new(recovery_config);
    bind_durable_medium(&mut restored, medium.attachment());
    let plan = restored
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source().unwrap(),
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
    let source = durable_runtime_with_test_schema();
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
                    Some(1),
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
    let source = durable_runtime_with_test_schema();
    let committed = commit_text(execution, &source).unwrap();
    release_test_commit_snapshot(&source, &committed);
    let native = source
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let unframed = native
        .bytes()
        .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
        .unwrap();
    let error =
        crate::durability::log::native_checkpoint_codec::decode_checkpoint(execution, unframed)
            .unwrap_err();
    assert!(
        matches!(error,RelationalRecoveryStop::Durability(ref error) if error.class == RecoveryFailureClass::UnsupportedLegacySemantics)
    );
    assert!(
        crate::durability::log::native_checkpoint_codec::decode_checkpoint(
            execution,
            native.bytes()
        )
        .is_ok()
    );
}

mod missing_dictionary;
