//! Native scope scheduling and observation custody in the existing Native court.
//! This is not public managed-root/World publication evidence.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    history::BranchId,
    identity::*,
    merge::{
        LoweredRecordDecisionKind, MergeExecutionPreparationError, MergeExecutionRequest,
        MergeIntent,
    },
    runtime::*,
    schema::*,
    symbols::ClientKey,
    transactions::*,
};

#[path = "managed_source/owner.rs"]
mod owner;

fn registry() -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    let mut registry = RelationalSchemaRegistry::new();
    let schema = SchemaId("managed-source-court".into());
    for (kind, name) in [(1, "directory"), (4, "item")] {
        registry = registry
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(kind),
                kind_name: name.into(),
                schema_id: schema.clone(),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: Default::default(),
            })
            .unwrap();
    }
    registry = registry
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "contains".into(),
            schema_id: schema.clone(),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::Forbid,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: "contains-endpoints".into(),
                    allowed_source_kinds: vec![KindId(1)],
                    allowed_target_kinds: vec![KindId(4)],
                    self_edges_allowed: false,
                    cross_context_policy: CrossContextPolicy::Forbid,
                }],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
        })
        .unwrap();
    let identity = DerivedRelationIdentity {
        schema,
        version: SchemaVersionId(1),
        name: "directory-items".into(),
    };
    registry = registry
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: identity.clone(),
            from: KindId(1).into(),
            to: KindId(4).into(),
            nodes: vec![DerivedRelationNode::Stored {
                relation: KindId(2),
                from: KindId(1),
                to: KindId(4),
            }],
            laws: None,
        }])
        .unwrap();
    let program = registry
        .bind_managed_index_source(&registry.derived_relation(&identity).unwrap(), vec![])
        .unwrap();
    (registry, program)
}

fn created(kind: u32, label: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: ClientKey::raw(label),
    }
}

fn commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    batch: WorkerIntentBatch,
) -> CommitResult {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}

#[test]
fn native_managed_source_stages_all_scopes_before_validation_and_reborrows_each_actual_observation()
{
    let (registry, program) = registry();
    let court = Arc::new(owner::Court::default());
    let rule = CustomInvariantRegistration::new(owner::Reader {
        program: program.clone(),
        court: court.clone(),
    })
    .unwrap()
    .with_managed_index_sources(vec![program])
    .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .custom_invariant(rule)
        .build()
        .unwrap();
    let _registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court.clone())))
        .unwrap();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let mut birth = WorkerIntentBatch::new("source-scope-birth");
    for (kind, label) in [(1, "A"), (1, "B"), (4, "x"), (4, "y")] {
        birth = birth.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(kind),
            client_key: ClientKey::raw(label),
            fields: Default::default(),
        })));
    }
    birth = birth.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw("A-x"),
            source: EntityReference::Created(created(1, "A")),
            target: EntityReference::Created(created(4, "x")),
            fields: Default::default(),
        },
    )));
    let born = commit(&runtime, &execution, birth);
    let a = born.created_entity(&created(1, "A")).unwrap();
    let b = born.created_entity(&created(1, "B")).unwrap();
    let x = born.created_entity(&created(4, "x")).unwrap();
    let y = born.created_entity(&created(4, "y")).unwrap();
    // Independent authored edge roster: A->x, empty B, then add B->x.
    // No public session or retained public root is opened for either scope.
    *court.expected.lock().unwrap() =
        BTreeMap::from([(a, (vec![x], vec![x])), (b, (vec![], vec![x]))]);
    court.staged.lock().unwrap().clear();
    court.observed.lock().unwrap().clear();
    court
        .checking_previous_proposal
        .store(true, Ordering::SeqCst);
    let shared = commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("share-with-unopened-B").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("B-x"),
                source: EntityReference::Existing(b),
                target: EntityReference::Existing(x),
                fields: Default::default(),
            }),
        )),
    );
    assert_eq!(*court.staged.lock().unwrap(), BTreeSet::from([a, b]));
    assert_eq!(*court.observed.lock().unwrap(), BTreeSet::from([a, b]));
    assert!(court.foreign_observation.load(Ordering::SeqCst) > 0);

    let b_x = shared
        .created_relation(&CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw("B-x"),
            source: EntityReference::Existing(b),
            target: EntityReference::Existing(x),
        })
        .unwrap();
    let source_branch = BranchId("managed-source-merge".into());
    let (_, fork_basis) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(&execution, source_branch.clone(), fork_basis)
        .unwrap();
    let source_identity = runtime.branch_identity(&source_branch).unwrap();
    let (_, source_basis) = runtime.observe_branch(&source_identity).unwrap();
    // Authored source-branch edit and merge expectation: retain A->x,
    // remove B->x. Main still contains both edges until the actual merge.
    *court.expected.lock().unwrap() =
        BTreeMap::from([(a, (vec![x], vec![x])), (b, (vec![x], vec![]))]);
    court.staged.lock().unwrap().clear();
    court.observed.lock().unwrap().clear();
    let mut transaction = runtime
        .begin_branch_transaction(
            &source_basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("remove-source-B-x").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: b_x }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(&execution, &runtime).unwrap();

    let bound = runtime
        .bind_merge_execution_request(MergeExecutionRequest::new(
            BranchId("main".into()),
            source_branch,
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    // Native requires manual resolution for source-deleted/target-live records.
    // The ordinary source-branch deletion above keeps its exact roster oracle;
    // it cannot lawfully be used as an executable merge fixture.
    court.staged.lock().unwrap().clear();
    court.observed.lock().unwrap().clear();
    let refused_before_planning = court.foreign_observation.load(Ordering::SeqCst);
    let error = runtime
        .prepare_merge_execution(&execution, bound)
        .unwrap_err();
    let MergeExecutionPreparationError::NotExecutionReady(report) = error else {
        panic!("expected the installed deletion rule, got {error:?}");
    };
    assert_eq!(
        (
            report.record_count,
            report.blocked_count,
            report.rejected_count
        ),
        (1, 1, 0)
    );
    assert_eq!(report.denied_records.len(), 1);
    assert_eq!(report.denied_records[0].record, RecordRef::Relation(b_x));
    assert_eq!(
        report.denied_records[0].decision,
        LoweredRecordDecisionKind::Block
    );
    assert!(court.staged.lock().unwrap().is_empty());
    assert!(court.observed.lock().unwrap().is_empty());
    assert_eq!(
        court.foreign_observation.load(Ordering::SeqCst),
        refused_before_planning
    );

    // A new fork of unchanged main contributes two genuine source-only edges.
    // No deletion policy is bypassed: main still has A->x and B->x, and the
    // merge must add A->y and B->y through the normal merge execution owner.
    let source_branch = BranchId("managed-source-additions".into());
    let (_, fork_basis) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(&execution, source_branch.clone(), fork_basis)
        .unwrap();
    let source_identity = runtime.branch_identity(&source_branch).unwrap();
    let (_, source_basis) = runtime.observe_branch(&source_identity).unwrap();
    let mut expanded = vec![x, y];
    expanded.sort();
    *court.expected.lock().unwrap() =
        BTreeMap::from([(a, (vec![x], expanded.clone())), (b, (vec![x], expanded))]);
    let mut additions = WorkerIntentBatch::new("shared-source-additions");
    for (scope, label) in [(a, "A-y"), (b, "B-y")] {
        additions = additions.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw(label),
                source: EntityReference::Existing(scope),
                target: EntityReference::Existing(y),
                fields: Default::default(),
            },
        )));
    }
    let mut transaction = runtime
        .begin_branch_transaction(
            &source_basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            additions,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(&execution, &runtime).unwrap();
    let bound = runtime
        .bind_merge_execution_request(MergeExecutionRequest::new(
            BranchId("main".into()),
            source_branch,
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    let prepared = runtime.prepare_merge_execution(&execution, bound).unwrap();
    assert!(!prepared.merged_intents().is_empty());
    court.staged.lock().unwrap().clear();
    court.observed.lock().unwrap().clear();
    let refused_before_merge = court.foreign_observation.load(Ordering::SeqCst);
    runtime
        .execute_prepared_merge(&execution, prepared)
        .unwrap();
    assert_eq!(*court.staged.lock().unwrap(), BTreeSet::from([a, b]));
    assert_eq!(*court.observed.lock().unwrap(), BTreeSet::from([a, b]));
    assert!(court.foreign_observation.load(Ordering::SeqCst) > refused_before_merge);
}
