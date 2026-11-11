//! Destruction of the final opaque value must not run under its directory lock.
use super::*;
use crate::tests::support::{create_entity, managed_cache_schema, snapshot_for_owner_identity};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Weak,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
struct Probe {
    image: Arc<Mutex<Weak<ManagedIndexBranchImage>>>,
    drops: Arc<AtomicUsize>,
}
impl Drop for Probe {
    fn drop(&mut self) {
        let image = self.image.lock().unwrap().upgrade().unwrap();
        assert!(
            image.roots.try_lock().is_ok(),
            "opaque Drop is outside directory mutex"
        );
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
#[test]
fn managed_cache_retirement_defers_last_opaque_drop_beyond_directory_guard() {
    let (schema, program) = managed_cache_schema();
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
        .unwrap();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let scope = create_entity(&execution, &runtime, "scope");
    let weak = Arc::new(Mutex::new(Weak::new()));
    let drops = Arc::new(AtomicUsize::new(0));
    let payload: Payload = Arc::new(Probe {
        image: weak.clone(),
        drops: drops.clone(),
    });
    let snapshot = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    let revision = runtime
        .read_truth()
        .positioned_snapshot(&snapshot)
        .unwrap()
        .commit_id()
        .unwrap();
    let mut image = ManagedIndexBranchImage::default();
    image
        .prepare_source(
            &execution,
            &program,
            scope,
            Availability::Available,
            revision,
            true,
            Some(&payload),
        )
        .unwrap();
    let image = ManagedIndexBranchImage::retain_header(&execution, image).unwrap();
    *weak.lock().unwrap() = Arc::downgrade(&image);
    drop(payload);
    let (_, foreign) = managed_cache_schema();
    assert!(matches!(
        image.retained(&foreign, scope),
        Err(
            crate::validation::candidate_source::ManagedIndexSourcePreparationDenial::ForeignSource
        )
    ));
    assert!(matches!(
        image.evict_source(&execution, &foreign, scope),
        Err(Stop::ForeignCell)
    ));
    let retired = image.evict_source(&execution, &program, scope).unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert!(image
        .retained(&program, scope)
        .unwrap()
        .unwrap()
        .payload()
        .is_none());
    drop(retired);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

// Keep real runtime construction in this existing managed-Source court owner.
#[test]
fn ordered_birth_edges_into_old_reverse_projection_anchors_prepare_the_prefix() {
    use crate::authority::mutation::{AdjacencyDelta, AdjacencyDeltaKind};
    use crate::derived_relation::{
        DerivedRelationDefinitionParts, DerivedRelationIdentity, DerivedRelationNode as Node,
    };
    use crate::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
    use crate::facade::identity::{KindId, PartitionId};
    use crate::facade::schema::*;
    use crate::facade::transactions::*;
    use crate::symbols::data::ClientKey;
    use crate::tests::support::{
        release_test_commit_snapshot, test_owner_begin_transaction_for_main,
    };
    use crate::validation::candidate_source::{scope_closure, ManagedIndexSourceProjectionRoute};
    use crate::validation::custom_rule::{CustomInvariantWorkMeter, StructuralRelationView};
    use crate::validation::data::CustomInvariantAccessContract;
    use crate::validation::engine::{
        state_view::InvariantStateView, InvariantObservation, InvariantRuntimeView,
    };
    use worth_execution::ExecutionAllocationPolicy;

    // The nested case uses the same physical relation in both directions:
    // row -> old endpoint, then new child -> old endpoint (read in reverse).
    for nested in [false, true] {
        let identity = |name: &str| DerivedRelationIdentity {
            schema: SchemaId("ordered-projection-court".into()),
            version: SchemaVersionId(1),
            name: name.into(),
        };
        let mut schema = RelationalSchemaRegistry::new();
        for kind in [1, 2] {
            schema = schema
                .register_entity_kind(EntityKindRegistration {
                    kind_id: KindId(kind),
                    kind_name: format!("node-{kind}"),
                    schema_id: identity("").schema,
                    schema_version_id: SchemaVersionId(1),
                    aspect_contract_declarations: Default::default(),
                })
                .unwrap();
        }
        for kind in [3, 4, 5, 6] {
            let mut relation_integrity = RelationIntegrityDeclarations::default();
            let (from, to) = match kind {
                3 => (1, 1),
                4 | 5 => (1, 2),
                _ => (2, 2),
            };
            relation_integrity
                .endpoint_kind_contracts
                .push(EndpointKindContractDeclaration {
                    contract_id: format!("endpoints-{kind}").into(),
                    allowed_source_kinds: vec![KindId(from)],
                    allowed_target_kinds: vec![KindId(to)],
                    self_edges_allowed: false,
                    cross_context_policy: CrossContextPolicy::Forbid,
                });
            if kind == 3 {
                relation_integrity
                    .cardinality_contracts
                    .push(CardinalityContractDeclaration {
                        contract_id: "one-predecessor".into(),
                        source_max: Some(1),
                        target_max: None,
                        pair_max: None,
                        source_min: None,
                        target_min: None,
                        pair_min: None,
                        pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
                        minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
                    });
            }
            schema = schema
                .register_relation_kind(RelationKindRegistration {
                    kind_id: KindId(kind),
                    kind_name: format!("edge-{kind}"),
                    schema_id: identity("").schema,
                    schema_version_id: SchemaVersionId(1),
                    cross_context_policy: CrossContextPolicy::Forbid,
                    cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
                    aspect_contract_declarations: Default::default(),
                    relation_integrity,
                })
                .unwrap();
        }
        let physical = |kind, from, to| Node::Stored {
            relation: KindId(kind),
            from: KindId(from),
            to: KindId(to),
        };
        schema = schema
            .register_derived_relations(vec![
                DerivedRelationDefinitionParts {
                    identity: identity("live"),
                    from: KindId(1).into(),
                    to: KindId(2).into(),
                    nodes: vec![
                        physical(3, 1, 1),
                        physical(4, 1, 2),
                        physical(5, 1, 2),
                        Node::OrderedEvents {
                            predecessor: 0,
                            introduced: 1,
                            retired: 2,
                        },
                    ],
                    laws: None,
                },
                DerivedRelationDefinitionParts {
                    identity: identity("forward"),
                    from: KindId(2).into(),
                    to: KindId(2).into(),
                    nodes: vec![physical(6, 2, 2)],
                    laws: None,
                },
                DerivedRelationDefinitionParts {
                    identity: identity("reverse"),
                    from: KindId(2).into(),
                    to: KindId(2).into(),
                    nodes: vec![physical(6, 2, 2), Node::Reverse { input: 0 }],
                    laws: None,
                },
            ])
            .unwrap();
        let reverse = schema.derived_relation(&identity("reverse")).unwrap();
        let routes = if nested {
            vec![
                ManagedIndexSourceProjectionRoute::new(
                    "parent".into(),
                    schema.derived_relation(&identity("forward")).unwrap(),
                    vec![],
                )
                .unwrap(),
                ManagedIndexSourceProjectionRoute::nested("child".into(), 0, reverse, vec![])
                    .unwrap(),
            ]
        } else {
            vec![
                ManagedIndexSourceProjectionRoute::new("endpoint".into(), reverse, vec![]).unwrap(),
            ]
        };
        let program = schema
            .bind_ordered_predecessor_managed_index_source(
                &schema.derived_relation(&identity("live")).unwrap(),
                vec![],
                routes,
            )
            .unwrap();
        let runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
            .schema_registry(schema.clone())
            .build()
            .unwrap();
        let host = crate::test_execution_policy::serial_request();
        let execution = ExecutionRequest::serial(&host);
        let created = |kind, key: &str| CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(kind),
            client_key: ClientKey::raw(key),
        };
        let entity = |kind, key: &str| {
            MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(kind),
                client_key: ClientKey::raw(key),
                fields: AspectFieldPatch::default(),
            }))
        };
        let edge = |kind, key: &str, source, target| {
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(kind),
                client_key: ClientKey::raw(key),
                source,
                target,
                fields: AspectFieldPatch::default(),
            }))
        };
        let mut seed = WorkerIntentBatch::new("seed-prefix");
        seed.intents.extend([
            entity(1, "P"),
            entity(2, "R"),
            entity(2, "Q"),
            edge(
                4,
                "introduced",
                EntityReference::Created(created(1, "P")),
                EntityReference::Created(created(2, "R")),
            ),
        ]);
        if nested {
            seed.intents.push(edge(
                6,
                "parent",
                EntityReference::Created(created(2, "R")),
                EntityReference::Created(created(2, "Q")),
            ));
        }
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(seed, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let seed = transaction.commit(&execution, &runtime).unwrap();
        let parent = seed.created_entity(&created(1, "P")).unwrap();
        let row = seed.created_entity(&created(2, "R")).unwrap();
        let anchor = if nested {
            seed.created_entity(&created(2, "Q")).unwrap()
        } else {
            row
        };
        release_test_commit_snapshot(&runtime, &seed);
        let before_observation =
            InvariantObservation::committed(runtime.storage_access().current_edition());
        let before_version = runtime.current_version_id();

        let mut candidate = WorkerIntentBatch::new("birth-and-reverse-edge");
        candidate.intents.extend([
            entity(1, "S"),
            entity(2, "X"),
            edge(
                3,
                "previous",
                EntityReference::Created(created(1, "S")),
                EntityReference::Existing(parent),
            ),
            edge(
                6,
                "new-endpoint",
                EntityReference::Created(created(2, "X")),
                EntityReference::Existing(anchor),
            ),
        ]);
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(candidate, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let candidate = transaction.commit(&execution, &runtime).unwrap();
        let successor = candidate.created_entity(&created(1, "S")).unwrap();
        let endpoint = candidate.created_entity(&created(2, "X")).unwrap();
        let after_observation =
            InvariantObservation::committed(runtime.storage_access().current_edition());
        let runtime_view = InvariantRuntimeView::from_runtime(&runtime);
        let access = Arc::new(CustomInvariantAccessContract {
            read_entity_kinds: vec![KindId(1), KindId(2)],
            read_relation_kinds: vec![KindId(3), KindId(4), KindId(5), KindId(6)],
            ..Default::default()
        });
        let before = StructuralRelationView::new(
            InvariantStateView::new(
                before_observation.committed_partition_access(),
                before_version,
            ),
            execution,
            CustomInvariantWorkMeter::new(None),
            access.clone(),
        )
        .with_structural_source(&runtime_view, &before_observation, true);
        let after = StructuralRelationView::new(
            InvariantStateView::new(
                after_observation.committed_partition_access(),
                runtime.current_version_id(),
            ),
            execution,
            CustomInvariantWorkMeter::new(None),
            access,
        )
        .with_structural_source(&runtime_view, &after_observation, false);
        let adjacency = [successor, endpoint]
            .into_iter()
            .flat_map(|entity| after.outgoing_relations_for_entity(entity).unwrap())
            .map(|relation| after.relation(relation).unwrap())
            .map(|record| AdjacencyDelta {
                relation_id: record.relation_id,
                kind_id: record.kind_id,
                kind: AdjacencyDeltaKind::Created {
                    source: record.source,
                    target: record.target,
                },
            })
            .collect::<Vec<_>>();
        let closure = scope_closure::discover(
            &schema,
            &program,
            &candidate.changed_records,
            &adjacency,
            &before,
            &after,
        )
        .unwrap();
        assert!(
            closure
                .scopes
                .get(&parent)
                .is_some_and(|rows| rows.contains(&row)),
            "the old row's projection changed (nested={nested})"
        );
        assert!(closure.scopes.contains_key(&successor));
        assert!(
            !closure.support.is_empty(),
            "general pullback must retain its real reads"
        );
        release_test_commit_snapshot(&runtime, &candidate);
    }
}
