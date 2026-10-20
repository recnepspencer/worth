//! Uniqueness scratch follows the admitted field, not borrowed record payloads.
use super::*;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

const REQUEST_MEMORY: u64 = 256 * 1024;
const LARGE_TEXT: usize = 1024 * 1024;

#[test]
fn uniqueness_ignores_large_sibling_and_unrelated_kind_payloads() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let runtime = fixture();
    let large = "x".repeat(LARGE_TEXT);
    commit_entity_with_summary(&execution, &runtime, "matching", "unique-title", &large)
        .expect("the matching record retains its independently owned sibling");
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("unrelated-payload").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(3),
                    client_key: ClientKey::raw("unrelated"),
                    fields: aspect_field_patch_from_values([(
                        aspect_key("payload"),
                        field_key("payload"),
                        string_aspect_value(&large),
                    )]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(&execution, &runtime).unwrap();
    let before = runtime_marker(&runtime);
    let result = evaluate(&runtime).expect("borrowed payloads consume no uniqueness scratch");
    assert_eq!(
        result.results().len(),
        1,
        "the uniqueness rule actually ran"
    );
    assert!(!result.summary().has_blocking_violation());
    assert_runtime_marker_unchanged(&runtime, before);
}

#[test]
fn uniqueness_refuses_large_matching_field_before_cloning_it() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let runtime = fixture();
    commit_entity_with_summary(
        &execution,
        &runtime,
        "large-title",
        &"x".repeat(LARGE_TEXT),
        "small-sibling",
    )
    .unwrap();
    let before = runtime_marker(&runtime);
    assert!(matches!(
        evaluate(&runtime),
        Err(TransactionCommitError::Execution {
            denial: crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                    crate::execution::RelationalExecutionDenialCause::ScratchCapacityExceeded
                ),
                partition_identity: Some(0),
            },
            ..
        })
    ));
    assert_runtime_marker_unchanged(&runtime, before);
}

fn fixture() -> RelationalRuntime {
    let registry = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry()
    .register_entity_kind(EntityKindRegistration {
        kind_id: KindId(3),
        kind_name: "test.unrelated-payload".to_owned(),
        schema_id: SchemaId("test".to_owned()),
        schema_version_id: SchemaVersionId(1),
        aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
            entity_field_aspect(aspect_key("payload"), field_key("payload")),
        ]),
    })
    .unwrap();
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::commit_boundary_blocking(
                InvariantRule::unique_entity_aspect_field(
                    aspect_key("summary"),
                    field_key("title"),
                ),
            )],
        })
        .build()
}

fn evaluate(
    runtime: &RelationalRuntime,
) -> Result<crate::validation::engine::InvariantExecutionResult, TransactionCommitError> {
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(REQUEST_MEMORY),
                    Some(10_000),
                ),
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    InvariantEngine::new(runtime)
        .execute(&worth_execution::ExecutionRequest::leased(&lease), request)
}
