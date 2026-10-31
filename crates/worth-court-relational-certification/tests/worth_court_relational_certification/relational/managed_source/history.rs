//! Real canonical impact candidates, including no-ops and transient membership.
use super::*;
use std::num::NonZeroUsize;
use worth_relational::facade::{branch::AdmittedRelationalBranchBasis, history::*};

pub(super) fn basis(runtime: &RelationalRuntime) -> AdmittedRelationalBranchBasis {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
}
pub(super) fn selected(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
) -> worth_relational::facade::change_source::RelationalSelectedCommit {
    runtime
        .select_exact_commit(
            &basis.observation(),
            basis.observation().commit_id().unwrap(),
        )
        .into_outcome()
        .unwrap()
}
pub(super) fn limits() -> RelationalAuthoredWritePageLimits {
    RelationalAuthoredWritePageLimits {
        maximum_entries: NonZeroUsize::new(1).unwrap(),
        maximum_source_visits: NonZeroUsize::new(1).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(usize::MAX).unwrap(),
    }
}
pub(super) fn collect(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    mut cursor: RelationalAuthoredWriteCursor,
) -> Vec<CommitId> {
    let mut result = Vec::new();
    loop {
        let page = cursor.next_page(runtime, execution, limits()).unwrap();
        assert_eq!(page.source_visits(), page.entries().len());
        result.extend(page.entries().map(|entry| entry.commit_id()));
        if page.is_complete() {
            return result;
        }
    }
}
enum HistoryProfile {
    Membership,
    EndpointProjection,
}
fn run_court(
    profile: HistoryProfile,
    run: impl FnOnce(&RelationalRuntime, &ManagedIndexSourceProgram, ExecutionRequest<'_, '_>),
) {
    let (mut registry, mut program) = registry();
    if matches!(profile, HistoryProfile::EndpointProjection) {
        let root = DerivedRelationIdentity {
            schema: SchemaId("managed-source-court".into()),
            version: SchemaVersionId(1),
            name: "directory-items".into(),
        };
        let reverse = DerivedRelationIdentity {
            name: "item-owner".into(),
            ..root.clone()
        };
        registry = registry
            .register_derived_relations(vec![DerivedRelationDefinitionParts {
                identity: reverse.clone(),
                from: KindId(4).into(),
                to: KindId(1).into(),
                nodes: vec![
                    DerivedRelationNode::Stored {
                        relation: KindId(2),
                        from: KindId(1),
                        to: KindId(4),
                    },
                    DerivedRelationNode::Reverse { input: 0 },
                ],
                laws: None,
            }])
            .unwrap();
        let route = ManagedIndexSourceProjectionRoute::new(
            "owner".into(),
            registry.derived_relation(&reverse).unwrap(),
            vec![],
        )
        .unwrap();
        drop(program);
        program = registry
            .bind_managed_index_source(
                &registry.derived_relation(&root).unwrap(),
                vec![],
                vec![route],
            )
            .unwrap();
    }
    let court = Arc::new(owner::Court::default());
    let rule = CustomInvariantRegistration::new(owner::Reader {
        maximum_work: None,
        program: program.clone(),
        court: court.clone(),
    })
    .unwrap()
    .with_managed_index_sources(vec![program.clone()])
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
        .activate(Arc::new(owner::SourceOwner(court)))
        .unwrap();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    run(&runtime, &program, ExecutionRequest::serial(&host));
}
fn edge(scope: EntityId, row: EntityId, label: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: ClientKey::raw(label),
        source: EntityReference::Existing(scope),
        target: EntityReference::Existing(row),
        fields: Default::default(),
    }))
}
fn edge_id(outcome: &CommitResult, scope: EntityId, row: EntityId, label: &str) -> RelationId {
    outcome
        .created_relation(&CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw(label),
            source: EntityReference::Existing(scope),
            target: EntityReference::Existing(row),
        })
        .unwrap()
}
fn endpoints(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    relation: RelationId,
    scope: EntityId,
    row: EntityId,
) -> CommitResult {
    commit(
        runtime,
        &execution,
        WorkerIntentBatch::new("authored-source-endpoints").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: KindId(2),
                source: EntityReference::Existing(scope),
                target: EntityReference::Existing(row),
            }),
        )),
    )
}

#[test]
fn managed_membership_history_keeps_noops_transients_shared_scope_candidates_and_disjoint_omission()
{
    run_court(HistoryProfile::Membership, |runtime, program, execution| {
        let mut birth = WorkerIntentBatch::new("history-source-birth");
        for (kind, label) in [
            (1, "A"),
            (1, "B"),
            (1, "C"),
            (1, "D"),
            (4, "x"),
            (4, "y"),
            (4, "z"),
            (4, "w"),
        ] {
            birth = birth.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(kind),
                client_key: ClientKey::raw(label),
                fields: Default::default(),
            })));
        }
        let born = commit(runtime, &execution, birth);
        let [a, b, c, d] =
            ["A", "B", "C", "D"].map(|name| born.created_entity(&created(1, name)).unwrap());
        let [x, y, z, w] =
            ["x", "y", "z", "w"].map(|name| born.created_entity(&created(4, name)).unwrap());
        let links = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("history-source-links")
                .push(edge(a, x, "A-x"))
                .push(edge(b, x, "B-x"))
                .push(edge(d, z, "D-z")),
        );
        let a_x = edge_id(&links, a, x, "A-x");
        let d_z = edge_id(&links, d, z, "D-z");
        let original = basis(runtime);
        let selectors = [a, b, c, d].map(|scope| {
            runtime
                .authored_managed_source_selector(execution, program, scope)
                .unwrap()
        });
        assert!(collect(
            runtime,
            execution,
            runtime
                .authored_managed_source_history_cursor(
                    execution,
                    &original,
                    selected(runtime, &original),
                    &selectors[..1]
                )
                .unwrap()
        )
        .is_empty());
        let noop = endpoints(runtime, execution, a_x, a, x);
        let entrant = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("transient-C-y").push(edge(c, y, "C-y")),
        );
        let c_y = edge_id(&entrant, c, y, "C-y");
        let moved = endpoints(runtime, execution, a_x, b, w);
        let restored = endpoints(runtime, execution, a_x, a, x);
        let departed = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("remove-transient-C-y").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: c_y }),
            )),
        );
        let disjoint = endpoints(runtime, execution, d_z, d, z);
        let target = basis(runtime);
        // Independent authored roster: A/B share x; C's y enters then leaves;
        // D's z is disjoint. B is an honest conservative candidate after A-x.
        let expected = [
            vec![
                noop.commit.commit_id,
                moved.commit.commit_id,
                restored.commit.commit_id,
            ],
            vec![
                noop.commit.commit_id,
                moved.commit.commit_id,
                restored.commit.commit_id,
            ],
            vec![entrant.commit.commit_id, departed.commit.commit_id],
            vec![disjoint.commit.commit_id],
        ];
        for (key, expected) in selectors.iter().zip(&expected) {
            let cursor = runtime
                .authored_managed_source_history_cursor(
                    execution,
                    &original,
                    selected(runtime, &target),
                    std::slice::from_ref(key),
                )
                .unwrap();
            assert_eq!(collect(runtime, execution, cursor), *expected);
        }
        // Unaccepted staged work and an interrupted preparation cannot move the
        // same immutable continuation; a fresh request gets its first real entry.
        let cursor = runtime
            .authored_managed_source_history_cursor(
                execution,
                &original,
                selected(runtime, &target),
                &selectors[..1],
            )
            .unwrap();
        drop(
            cursor
                .prepare_next_page(runtime, execution, limits())
                .unwrap(),
        );
        let cancel = CancellationSource::new();
        cancel.cancel();
        let stopped =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
        assert!(matches!(
            cursor.prepare_next_page(runtime, ExecutionRequest::serial(&stopped), limits()),
            Err(RelationalAuthoredWriteDenial::Request(
                worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::Cancelled
                )
            ))
        ));
        assert_eq!(collect(runtime, execution, cursor), expected[0]);
        runtime
            .discard_authored_write_generation(execution, selected(runtime, &target))
            .unwrap();
        assert!(matches!(
            runtime.authored_managed_source_history_cursor(
                execution,
                &original,
                selected(runtime, &target),
                &selectors[..1]
            ),
            Err(RelationalAuthoredWriteDenial::GenerationUnavailable)
        ));
        runtime
            .rebuild_authored_write_generation(execution, selected(runtime, &target))
            .unwrap();
        assert_eq!(
            collect(
                runtime,
                execution,
                runtime
                    .authored_managed_source_history_cursor(
                        execution,
                        &original,
                        selected(runtime, &target),
                        &selectors[..1]
                    )
                    .unwrap()
            ),
            expected[0]
        );
    });
}

#[test]
fn managed_history_selector_requires_actual_program_and_full_generation_scope() {
    run_court(HistoryProfile::Membership, |runtime, program, execution| {
        let (_, foreign) = registry();
        let scope = EntityId::new(PartitionId::main(), 7, 1);
        assert!(matches!(
            runtime.authored_managed_source_selector(execution, &foreign, scope),
            Err(RelationalAuthoredWriteDenial::ForeignManagedSourceProgram)
        ));
        assert!(matches!(
            runtime.authored_managed_source_selector(
                execution,
                program,
                EntityId::new(PartitionId::main(), 7, 0)
            ),
            Err(RelationalAuthoredWriteDenial::UnsupportedManagedSourceSelector)
        ));
        let cancel = CancellationSource::new();
        cancel.cancel();
        let stopped =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
        assert!(matches!(
            runtime.authored_managed_source_selector(
                ExecutionRequest::serial(&stopped),
                program,
                scope
            ),
            Err(RelationalAuthoredWriteDenial::Request(
                worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::Cancelled
                )
            ))
        ));
    });
}

#[test]
fn managed_history_explicitly_refuses_registered_endpoint_projection_profiles() {
    run_court(
        HistoryProfile::EndpointProjection,
        |runtime, program, execution| {
            assert!(!program.projection_routes().is_empty());
            assert!(matches!(
                runtime.authored_managed_source_selector(
                    execution,
                    program,
                    EntityId::new(PartitionId::main(), 7, 1)
                ),
                Err(RelationalAuthoredWriteDenial::UnsupportedManagedSourceSelector)
            ));
        },
    );
}
