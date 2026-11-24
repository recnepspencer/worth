//! Destruction of the final opaque value must not run under its directory lock.
use super::*;
use crate::validation::candidate_source::program::schema_fixture::managed_cache_schema;
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
    let (_, program) = managed_cache_schema();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let scope =
        crate::identity::data::EntityId::new(crate::identity::data::PartitionId::main(), 0, 1);
    let weak = Arc::new(Mutex::new(Weak::new()));
    let drops = Arc::new(AtomicUsize::new(0));
    let payload: Payload = Arc::new(Probe {
        image: weak.clone(),
        drops: drops.clone(),
    });
    let revision = crate::history::data::CommitId(1);
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

#[test]
fn ordered_birth_edges_into_old_reverse_projection_anchors_prepare_the_prefix() {
    use crate::authority::mutation::{AdjacencyDelta, AdjacencyDeltaKind};
    use crate::config::data::CascadeDeletePolicy;
    use crate::config::data::CrossContextPolicy;
    use crate::config::data::RelationalRuntimeConfig;
    use crate::derived_relation::definition::DerivedRelationDefinitionParts;
    use crate::derived_relation::definition::DerivedRelationIdentity;
    use crate::derived_relation::definition::DerivedRelationNode as Node;
    use crate::identity::data::{KindId, PartitionId, VersionId};
    use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
    use crate::schema::data::{
        CardinalityContractDeclaration, EndpointKindContractDeclaration, EntityKindRegistration,
        MinimumCardinalityEnforcement, PairMinimumSemantics, RelationIntegrityDeclarations,
        RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    };
    use crate::storage::overlay::small_relation_partition::typed_relation_partition;
    use crate::transactions::data::primitives::RecordRef;
    use crate::validation::candidate_source::{scope_closure, ManagedIndexSourceProjectionRoute};
    use crate::validation::custom_rule::{CustomInvariantWorkMeter, StructuralRelationView};
    use crate::validation::data::descriptor::CustomInvariantAccessContract;
    use crate::validation::engine::test_view::invariant_view;
    use crate::validation::engine::{state_view::InvariantStateView, InvariantObservation};
    use std::collections::BTreeMap;

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
        let host = crate::execution::test_policy::serial_request();
        let execution = ExecutionRequest::serial(&host);
        let id = PartitionId::main();
        let mut edges = vec![(0, 1, KindId(4))];
        if nested {
            edges.push((1, 2, KindId(6)));
        }
        let kinds = [KindId(1), KindId(2), KindId(2), KindId(1), KindId(2)];
        let (before, _, _) = typed_relation_partition(id, kinds[..3].iter().copied(), &edges);
        let anchor = if nested { 2 } else { 1 };
        edges.extend([(3, 0, KindId(3)), (4, anchor, KindId(6))]);
        let (after, entities, relations) = typed_relation_partition(id, kinds.into_iter(), &edges);
        let before = BTreeMap::from([(id, before)]);
        let after = BTreeMap::from([(id, after)]);
        let (parent, row, successor, endpoint) =
            (entities[0], entities[1], entities[3], entities[4]);
        let changed = [
            RecordRef::Entity(successor),
            RecordRef::Entity(endpoint),
            RecordRef::Relation(relations[relations.len() - 2]),
            RecordRef::Relation(relations[relations.len() - 1]),
        ];
        let before_observation = InvariantObservation::committed_branch(&before);
        let after_observation = InvariantObservation::committed_branch(&after);
        let mut config = RelationalRuntimeConfig::default();
        config.schema.registry = schema.clone();
        let instrumentation = RuntimeInstrumentation::default();
        let runtime_view = invariant_view(&instrumentation, config, VersionId(1));
        let access = Arc::new(CustomInvariantAccessContract {
            read_entity_kinds: vec![KindId(1), KindId(2)],
            read_relation_kinds: vec![KindId(3), KindId(4), KindId(5), KindId(6)],
            ..Default::default()
        });
        let before = StructuralRelationView::new(
            InvariantStateView::new(
                before_observation.committed_partition_access(),
                VersionId(1),
            ),
            execution,
            CustomInvariantWorkMeter::new(None),
            access.clone(),
        )
        .with_structural_source(&runtime_view, &before_observation, true);
        let after = StructuralRelationView::new(
            InvariantStateView::new(after_observation.committed_partition_access(), VersionId(1)),
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
        let closure =
            scope_closure::discover(&schema, &program, &changed, &adjacency, &before, &after)
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
    }
}
