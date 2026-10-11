//! The shipped outgoing-relation rule is exercised at the native commit door.
use super::*;
use std::num::{NonZeroU64, NonZeroUsize};
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::PartitionId,
    mvcc::RelationalTransactionIntent,
    runtime::{InvariantReportedRule, InvariantVerdict, RelationalRuntime, RelationalRuntimeApi},
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
        RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::*,
};

const VERTEX: KindId = KindId::new(1);
const EDGE: KindId = KindId::new(2);
const ANCHOR: KindId = KindId::new(3);
const MEMORY_BYTES: u64 = 64 * 1024 * 1024;
const UNBOUNDED_WORK: u64 = 8_000_000;

fn installed_rule_world() -> (RelationalRuntime, EntityId) {
    let mut schema = RelationalSchemaRegistry::new();
    for (kind_id, name) in [(VERTEX, "Vertex"), (ANCHOR, "Anchor")] {
        schema = schema
            .register_entity_kind(EntityKindRegistration {
                kind_id,
                kind_name: name.into(),
                schema_id: SchemaId("installed-relations-test".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
            .unwrap();
    }
    schema = schema
        .register_relation_kind(RelationKindRegistration {
            kind_id: EDGE,
            kind_name: "RequiredEdge".into(),
            schema_id: SchemaId("installed-relations-test".into()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
        .unwrap();
    let rule = InstalledRequiresOutgoingRelationsRule {
        rule_id: "installed-required-outgoing".into(),
        display_name: Arc::from("Required outgoing edge"),
        semantic_version: CustomInvariantSemanticVersion::new(1, 0),
        relevant_entity_kinds: vec![VERTEX],
        required_relation_kinds: vec![EDGE],
        traversal_depth: 2,
        maximum_work_units: NonZeroU64::new(4096).unwrap(),
    };
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .custom_invariant(CustomInvariantRegistration::new(rule).unwrap())
        .build();
    let entity = |kind_id, key| EntitySpec {
        partition_id: PartitionId::main(),
        kind_id,
        client_key: ClientKey::raw(key),
        fields: AspectFieldPatch::default(),
    };
    let source = entity(VERTEX, "vertex");
    let target = entity(ANCHOR, "anchor");
    let reference = |spec: &EntitySpec| {
        EntityReference::Created(CreatedEntityRef {
            partition_id: spec.partition_id,
            kind_id: spec.kind_id,
            client_key: spec.client_key.clone(),
        })
    };
    let source_reference = CreatedEntityRef {
        partition_id: source.partition_id,
        kind_id: source.kind_id,
        client_key: source.client_key.clone(),
    };
    let relation = RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: EDGE,
        client_key: ClientKey::raw("edge"),
        source: reference(&source),
        target: reference(&target),
        fields: AspectFieldPatch::default(),
    };
    let mut transaction = transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("seed")
                .push(MutationIntent::Create(CreateIntent::Entity(source)))
                .push(MutationIntent::Create(CreateIntent::Entity(target)))
                .push(MutationIntent::Create(CreateIntent::Relation(relation))),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(MEMORY_BYTES),
        CancellationSource::new().token(),
        None,
    );
    let seeded = transaction
        .commit(&ExecutionRequest::serial(&serial), &runtime)
        .unwrap();
    let source = seeded.created_entity(&source_reference).unwrap();
    (runtime, source)
}

fn transaction(
    runtime: &RelationalRuntime,
) -> worth_relational::facade::mvcc::BranchBoundRelationalTransaction {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap()
}

fn commit(
    leased: bool,
    ceiling: u64,
    cancel: bool,
) -> (Result<CommitResult, TransactionCommitError>, u64) {
    let (runtime, entity_id) = installed_rule_world();
    let mut candidate = transaction(&runtime);
    candidate
        .push_batch(
            WorkerIntentBatch::new("check-existing-vertex").push(MutationIntent::Entity(
                EntityMutationIntent::Revalidate(RevalidateEntityIntent { entity_id }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let cancellation = CancellationSource::new();
    let authority = worth_query_execution::facade::primary_graph::test_execution_authority();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, MEMORY_BYTES, UNBOUNDED_WORK),
            ),
            deadline: None,
            cancellation: cancellation.token(),
        })
        .unwrap();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(MEMORY_BYTES),
        cancellation.token(),
        None,
    );
    let request = if leased {
        ExecutionRequest::leased(&lease)
    } else {
        ExecutionRequest::serial(&serial)
    };
    let (result, report) = request
        .run(ExecutionWorkCeiling::new(ceiling), |_| {
            if cancel {
                cancellation.cancel();
            }
            candidate.commit(&request, &runtime)
        })
        .unwrap();
    (result, report.charged_work())
}

fn verdict(result: &CommitResult) -> &InvariantVerdict {
    result
        .invariant_executions()
        .iter()
        .flat_map(|execution| execution.results())
        .find_map(|result| match &result.rule {
            InvariantReportedRule::Custom(identity)
                if identity.rule_id.as_str() == "installed-required-outgoing" =>
            {
                Some(&result.verdict)
            }
            _ => None,
        })
        .expect("the actual installed rule ran at commit")
}

#[test]
fn installed_relations_lease_exact_budget_keeps_the_serial_verdict() {
    let serial = commit(false, UNBOUNDED_WORK, false).0.unwrap();
    let (paid, exact_work) = commit(true, UNBOUNDED_WORK, false);
    let paid = paid.unwrap();
    assert!(matches!(verdict(&serial), InvariantVerdict::Pass));
    assert_eq!(verdict(&paid), verdict(&serial));
    let (exact, work) = commit(true, exact_work, false);
    assert_eq!(verdict(&exact.unwrap()), verdict(&serial));
    assert_eq!(work, exact_work);
}

#[test]
fn installed_relations_lease_one_unit_short_is_the_work_stop() {
    let (paid, exact_work) = commit(true, UNBOUNDED_WORK, false);
    assert!(matches!(verdict(&paid.unwrap()), InvariantVerdict::Pass));
    let (short, _) = commit(true, exact_work.checked_sub(1).unwrap(), false);
    let Err(TransactionCommitError::Execution { denial, .. }) = short else {
        panic!("{short:?}")
    };
    assert_eq!(
        denial.kind,
        CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::WorkExhausted)
    );
}

#[test]
fn installed_relations_lease_cancelled_commit_keeps_the_cancellation() {
    let (cancelled, _) = commit(true, UNBOUNDED_WORK, true);
    let Err(TransactionCommitError::Execution { denial, .. }) = cancelled else {
        panic!("{cancelled:?}")
    };
    assert_eq!(
        denial.kind,
        CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::Cancelled)
    );
}
