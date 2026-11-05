//! Pure registered-recipe and owned-source mechanics; no runtime or session.
use super::super::{Adjacency, DerivedRelationReads};
use super::*;
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::derived_relation::{
    DerivedRelationDefinitionParts, DerivedRelationIdentity, DerivedRelationNode as Node,
};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::schema::data::*;
use crate::validation::data::StructuralRelationRecord;
use std::collections::BTreeMap;
pub(super) fn id(slot: u64) -> EntityId {
    EntityId::new(PartitionId(0), slot, 1)
}
pub(super) fn identity(name: &str) -> DerivedRelationIdentity {
    DerivedRelationIdentity {
        schema: SchemaId("memo-owned-source".into()),
        version: SchemaVersionId(1),
        name: name.into(),
    }
}
pub(super) fn registry() -> RelationalSchemaRegistry {
    let mut registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "node".into(),
            schema_id: SchemaId("memo-owned-source".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    for kind in [2, 3] {
        registry = registry
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(kind),
                kind_name: format!("edge-{kind}"),
                schema_id: SchemaId("memo-owned-source".into()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::Forbid,
                cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
                aspect_contract_declarations: Default::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: format!("edge-{kind}").into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: true,
                        cross_context_policy: CrossContextPolicy::Forbid,
                    }],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                ),
            })
            .unwrap();
    }
    let parts = |name: &str, nodes| DerivedRelationDefinitionParts {
        identity: identity(name),
        from: KindId(1).into(),
        to: KindId(1).into(),
        nodes,
        laws: None,
    };
    registry
        .register_derived_relations(vec![
            parts(
                "a",
                vec![Node::Stored {
                    relation: KindId(2),
                    from: KindId(1),
                    to: KindId(1),
                }],
            ),
            parts(
                "b",
                vec![Node::Stored {
                    relation: KindId(3),
                    from: KindId(1),
                    to: KindId(1),
                }],
            ),
            parts(
                "union",
                vec![
                    Node::Derived(identity("a")),
                    Node::Derived(identity("b")),
                    Node::Union { left: 0, right: 1 },
                ],
            ),
            parts(
                "intersection",
                vec![
                    Node::Derived(identity("a")),
                    Node::Derived(identity("b")),
                    Node::Intersection { left: 0, right: 1 },
                ],
            ),
        ])
        .unwrap()
}
#[derive(Default)]
struct Source {
    stopped: bool,
    stop_after_read: bool,
}
impl SourceRead for Source {
    fn checkpoint(&mut self, _: usize) -> Result<(), Denial> {
        if self.stopped {
            Err(Denial::Request(
                crate::execution::RelationalExecutionDenialCause::Cancelled,
            ))
        } else {
            Ok(())
        }
    }
    fn retain(&mut self, _: usize) -> Result<(), Denial> {
        self.checkpoint(0)
    }
    fn check_scratch_peak(&mut self, _: usize) -> Result<(), Denial> {
        self.checkpoint(0)
    }
    fn entity_kind(&mut self, _: EntityId) -> Result<KindId, Denial> {
        self.checkpoint(0)?;
        Ok(KindId(1))
    }
    fn adjacency(&mut self, probe: Adjacency) -> Result<Vec<StructuralRelationRecord>, Denial> {
        self.checkpoint(0)?;
        let records = [(2, 1, 2), (2, 1, 4), (3, 1, 3), (3, 1, 4)];
        let rows = records
            .into_iter()
            .enumerate()
            .filter(|&(_, (kind, a, b))| {
                probe.relation == KindId(kind)
                    && probe.anchor == if probe.outgoing { id(a) } else { id(b) }
            })
            .map(|(ordinal, (kind, a, b))| StructuralRelationRecord {
                relation_id: RelationId::new(PartitionId(0), ordinal as u64, 1),
                kind_id: KindId(kind),
                source: id(a),
                target: id(b),
            })
            .collect();
        self.stopped = self.stop_after_read;
        Ok(rows)
    }
    fn reads(&self) -> DerivedRelationReads {
        DerivedRelationReads::default()
    }
}
fn evaluation<'a>(
    registry: &'a RelationalSchemaRegistry,
    source: &'a mut Source,
) -> Evaluation<'a, Source> {
    Evaluation {
        registry,
        root_definition: registry.derived_relation(&identity("union")).unwrap(),
        read: source,
        pending: Vec::new(),
        values: MemoValues::default(),
        entities: BTreeMap::new(),
        relations: BTreeMap::new(),
        adjacency: BTreeSet::new(),
    }
}
#[test]
fn local_keys_preserve_named_identity_coordinates_and_total_order() {
    let registry = registry();
    let root = registry.derived_relation(&identity("union")).unwrap();
    let mut source = Source::default();
    let mut evaluation = evaluation(&registry, &mut source);
    let key = Key::root(root.clone(), 0, id(1).into(), false);
    let normalized = evaluation.target_key(&key, root).unwrap();
    assert_eq!(normalized.definition_scope, DefinitionScope::Root);
    let a = evaluation
        .target_key(&key, registry.derived_relation(&identity("a")).unwrap())
        .unwrap();
    let b = evaluation
        .target_key(&key, registry.derived_relation(&identity("b")).unwrap())
        .unwrap();
    assert_ne!(a, b);
    assert!(key.comparison_bytes().unwrap() < a.comparison_bytes().unwrap());
    let keys = [
        key.clone(),
        Key {
            node: 1,
            ..key.clone()
        },
        Key {
            anchor: id(2).into(),
            ..key.clone()
        },
        Key {
            reverse: true,
            ..key.clone()
        },
        a,
        b,
    ];
    for (i, left) in keys.iter().enumerate() {
        for (j, right) in keys.iter().enumerate() {
            assert_eq!(left == right, i == j);
            assert_eq!(left == right, left.cmp(right).is_eq());
            assert_eq!(left.cmp(right), right.cmp(left).reverse());
            for third in &keys {
                if left < right && right < third {
                    assert!(left < third);
                }
            }
        }
    }
}
#[test]
fn named_union_and_intersection_match_independent_edge_sets() {
    let registry = registry();
    for (name, expected) in [("union", vec![2, 3, 4]), ("intersection", vec![4])] {
        let definition = registry.derived_relation(&identity(name)).unwrap();
        let result = super::super::evaluate_position(
            &registry,
            &definition,
            2,
            false,
            id(1).into(),
            &mut Source::default(),
        )
        .unwrap();
        assert_eq!(
            result.endpoints(),
            expected
                .into_iter()
                .map(|slot| id(slot).into())
                .collect::<Vec<Member>>()
        );
        assert_eq!(
            result.adjacency_dependencies().len(),
            2,
            "both Named source reads remain required"
        );
    }
}
#[test]
fn named_evaluation_preserves_the_original_owned_source_stop() {
    let registry = registry();
    let definition = registry.derived_relation(&identity("union")).unwrap();
    let mut source = Source {
        stop_after_read: true,
        ..Source::default()
    };
    let result = super::super::evaluate_position(
        &registry,
        &definition,
        2,
        false,
        id(1).into(),
        &mut source,
    );
    assert!(matches!(
        result,
        Err(Denial::Request(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        ))
    ));
}

#[test]
fn root_partition_and_named_storage_match_independent_full_key_operations() {
    use crate::graph_member::ProjectedMemberCoordinate;
    use crate::projected_member_domain::ProjectedMemberDomainIdentity;
    let registry = registry();
    let projected = |case: &str| {
        Member::Projected(
            std::sync::Arc::new(ProjectedMemberCoordinate {
                domain: ProjectedMemberDomainIdentity {
                    schema: SchemaId("memo-coordinate".into()),
                    version: SchemaVersionId(1),
                    name: "choice".into(),
                },
                carrier: id(1),
                case: case.into(),
            })
            .into(),
        )
    };
    let anchors = [
        id(1).into(),
        id(2).into(),
        projected("left"),
        projected("right"),
    ];
    let mut keys = Vec::new();
    for name in ["union", "a", "b"] {
        let definition = registry.derived_relation(&identity(name)).unwrap();
        for node in 0..definition.parts().nodes.len() {
            for reverse in [false, true] {
                for anchor in &anchors {
                    keys.push(Key {
                        definition: definition.clone(),
                        definition_scope: if name == "union" {
                            DefinitionScope::Root
                        } else {
                            DefinitionScope::Named
                        },
                        node,
                        anchor: anchor.clone(),
                        reverse,
                    });
                }
            }
        }
    }
    let mut actual = MemoValues::default();
    #[expect(
        clippy::mutable_key_type,
        reason = "Key orders immutable declared identity, node, member and direction; its retained binding cache is excluded"
    )]
    let mut reference = BTreeMap::new();
    let mut source = Source::default();
    let mut seed = 919u32;
    for step in 0..512 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let key = &keys[seed as usize % keys.len()];
        if step % 3 == 0 {
            assert_eq!(
                actual.remove(key, &mut source).unwrap(),
                reference.remove(key)
            );
        } else {
            let value = BTreeSet::from([id(4 + step % 7).into()]);
            assert_eq!(
                actual
                    .insert(key.clone(), value.clone(), &mut source)
                    .unwrap(),
                reference.insert(key.clone(), value)
            );
        }
        for candidate in &keys {
            assert_eq!(
                actual.get(candidate, &mut source).unwrap(),
                reference.get(candidate)
            );
        }
    }
    let mut stopped = Source {
        stopped: true,
        ..Default::default()
    };
    let old = reference.get(&keys[0]).cloned();
    assert!(matches!(
        actual.insert(keys[0].clone(), BTreeSet::new(), &mut stopped),
        Err(Denial::Request(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        ))
    ));
    assert_eq!(actual.get(&keys[0], &mut source).unwrap().cloned(), old);
}
