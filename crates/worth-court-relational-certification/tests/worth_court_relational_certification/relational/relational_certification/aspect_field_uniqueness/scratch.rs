//! Uniqueness scratch follows the admitted field, not borrowed record payloads.
use super::*;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    runtime::{
        InvariantCatalog, InvariantExecutionPoint, InvariantExecutionResult, InvariantRegistration,
        InvariantRule, RelationalRuntimeApi,
    },
    schema::{EntityKindRegistration, KindAspectContractDeclarations, SchemaId, SchemaVersionId},
};
use worth_testkit_relational::aspect_storage::{
    entity_field_aspect, entity_summary_struct_aspect, publications::string_aspect_field_patch,
    AspectSchemaFixture,
};

const REQUEST_MEMORY: u64 = 256 * 1024;
const LARGE_TEXT: usize = 1024 * 1024;

#[test]
fn uniqueness_ignores_large_sibling_and_unrelated_kind_payloads() {
    let host = serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let runtime = fixture();
    let large = "x".repeat(LARGE_TEXT);
    commit_entity_with_summary(&execution, &runtime, "matching", "unique-title", &large)
        .expect("the matching record retains its independently owned sibling");
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("unrelated-payload").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(3),
                    client_key: ClientKey::raw("unrelated"),
                    fields: string_aspect_field_patch([(
                        aspect_key("payload"),
                        field_key("payload"),
                        large.as_str(),
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
    let host = serial_request();
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
    // The request refuses the matching field's clone at its charge, before
    // the clone exists: the refused increment is at least the field itself.
    let Err(TransactionCommitError::Execution {
        denial:
            CommitExecutionDenial {
                kind:
                    CommitExecutionDenialKind::Cause(
                        RelationalExecutionDenialCause::PolicyMemoryExhausted {
                            requested,
                            admitted,
                            ..
                        },
                    ),
                partition_identity: Some(0),
            },
        ..
    }) = evaluate(&runtime)
    else {
        panic!("the request's memory policy refuses the large matching field");
    };
    assert!(requested >= LARGE_TEXT as u64, "refused charge {requested}");
    assert!(admitted <= REQUEST_MEMORY, "admitted {admitted}");
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
    let mut registration = InvariantRegistration::commit_boundary_blocking(
        InvariantRule::unique_entity_aspect_field(aspect_key("summary"), field_key("title")),
    );
    registration.execution_point = InvariantExecutionPoint::MutationSensitive;
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .invariant_catalog(InvariantCatalog {
            registrations: vec![registration],
        })
        .build()
        .expect("valid native runtime configuration")
}

fn evaluate(
    runtime: &RelationalRuntime,
) -> Result<InvariantExecutionResult, TransactionCommitError> {
    let lease = super::super::acyclicity_resources::request_authority()
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
    runtime
        .validation()
        .mutation_sensitive_state(&worth_execution::ExecutionRequest::leased(&lease))
}
