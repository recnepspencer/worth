use super::branch_history_publications::create_entity;
use super::checkpoint_source_capture::capture_durably;
use std::sync::Arc;
use worth_execution::ExecutionImmutableBytes;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_foundational::facade::InternedString;
use worth_relational::facade::durability::RecoveryVerificationMode;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, VersionId};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CommitResult;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::TransactionCommitError;
use worth_testkit_relational::aspect_storage::definition::aspect_key;
use worth_testkit_relational::aspect_storage::definition::field_key;
use worth_testkit_relational::aspect_storage::publications::changed_entities;
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::durable_publication::batch_create;
use worth_testkit_relational::durable_publication::create_entity_outcome;
use worth_testkit_relational::durable_publication::durable_runtime;

const JSON: &str = r#"{"actor":"native","request":{"input":"exact logical content"}}"#;
fn commit_text(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
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
        worth_relational::facade::transactions::planned_single_field_locator(
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
    runtime: &RelationalRuntime,
    entity: EntityId,
    version: VersionId,
) -> worth_relational::facade::runtime::RelationalPinnedRead {
    let record = runtime
        .read_truth()
        .visible_entity_at_version(execution, entity, version)
        .unwrap()
        .unwrap();
    let state = record.authoritative_aspect_state.as_ref().unwrap();
    let value = match state.get(&aspect_key("name")).unwrap().view() {
        ContractValidatedAspectValueView::Scalar(value) => value,
        ContractValidatedAspectValueView::Struct(value) => value.get(&field_key("name")).unwrap(),
    };
    let AspectValue::String(text) = value else {
        panic!("the admitted field must contain text")
    };
    let snapshot = super::checkpoint_recovery_observation::snapshot_for_branch(
        runtime,
        &BranchId("main".into()),
    );
    let pinned = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[], execution)
        .unwrap();
    let actual = match text {
        InternedString::Raw(text) => Some(text.as_str()),
        InternedString::Symbol(symbol) => pinned
            .symbols()
            .resolve(worth_relational::facade::symbols::Symbol(symbol.0)),
    };
    assert_eq!(actual, Some(JSON));
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
    pinned
}

fn test_execution_lease() -> worth_execution::ExecutionResourceLease<'static> {
    use std::{num::NonZeroUsize, sync::OnceLock};
    use worth_execution::{
        CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, LeaseRequest,
    };
    use worth_foundational::facade::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY
        .get_or_init(|| {
            ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::new(8).unwrap(),
                charged_memory_bytes: Some(512 * 1024 * 1024),
            })
            .unwrap()
        })
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), None, Some(1_000_000)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

#[test]
fn native_field_text_checkpoint_then_tail_recovers_exact_json_after_original_owners_drop() {
    let original = test_execution_lease();
    let execution = &ExecutionRequest::leased(&original);
    let medium = worth_testkit_relational::durable_medium::TestDurableMedium::new();
    let source = durable_runtime(&medium);
    let baseline = create_entity_outcome(execution, &source, "checkpoint-before-managed-text");
    release_test_commit_snapshot(&source, &baseline);
    let recovery_schema = source.config().schema.registry; // no managed text exists here
    capture_durably(execution, &source, &medium).unwrap();
    let committed = commit_text(execution, &source).unwrap();
    let entity = changed_entities(&committed)[0];
    let version = committed.version_id;
    release_test_commit_snapshot(&source, &committed);
    drop(committed);
    drop(baseline);
    drop(source);
    drop(original);

    let recovery_lease = test_execution_lease();
    let execution = &ExecutionRequest::leased(&recovery_lease);
    let mut restored = RelationalRuntimeApi::builder()
        .profile(worth_relational::facade::config::RelationalRuntimeProfile::CertificationCore)
        .schema_registry(recovery_schema)
        .build_durable(medium.attachment())
        .unwrap();
    let plan = restored
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let outcome = restored
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(outcome.coverage.replayed_tail_commits, 1);
    assert_eq!(outcome.coverage.checkpoint_commits, 1);
    let pinned = read_exact(execution, &restored, entity, version);
    let key = pinned.symbols().symbol("native-managed-text").unwrap();
    assert_eq!(pinned.symbols().resolve(key), Some("native-managed-text"));
    create_entity(execution, &restored, "after-restored-managed-text");
    assert_eq!(pinned.symbols().resolve(key), Some("native-managed-text"));
    drop(restored);
    assert_eq!(
        pinned
            .symbols()
            .symbol(JSON)
            .and_then(|symbol| pinned.symbols().resolve(symbol)),
        Some(JSON)
    );
}
