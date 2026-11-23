//! An ordered-events Source behind one derived relation (the shape a
//! `forward_derived` root path compiles to) is resolved, not refused: a scope
//! born in the candidate may gain a predecessor; nothing older may.
use super::*;
use crate::authority::mutation::{AdjacencyDelta, AdjacencyDeltaKind, MutationEffect};
use crate::publication::patch::data::RecordStructuralChange as Change;
use crate::derived_relation::DerivedRelationNode as Node;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::schema::data::RelationalSchemaRegistry;

const SCOPE: KindId = KindId(1);
const MEMBER: KindId = KindId(3);
const PREDECESSOR: KindId = KindId(4);

/// A Source whose root path is `[Derived(event-source)]` over an ordered
/// membership; its compiled recipe is absent, as for a same-scope Source.
fn ordered_behind_derived() -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    let (registry, template) = crate::tests::support::managed_cache_schema();
    let mut order = registry.relation_registration(MEMBER).unwrap().clone();
    order.kind_id = PREDECESSOR;
    order.kind_name = "event-predecessor".into();
    order.aspect_contract_declarations = Default::default();
    let mut membership = template.0.path.parts().clone();
    membership.identity.name = "event-membership".into();
    let leaf = membership.nodes[0].clone();
    membership.nodes = vec![
        Node::Stored { relation: PREDECESSOR, from: SCOPE, to: SCOPE },
        leaf.clone(),
        leaf,
        Node::OrderedEvents { predecessor: 0, introduced: 1, retired: 2 },
    ];
    let mut source = template.0.path.parts().clone();
    source.identity.name = "event-source".into();
    source.nodes = vec![Node::Derived(membership.identity.clone())];
    let identity = source.identity.clone();
    let registry = registry
        .register_relation_kind(order)
        .unwrap()
        .register_derived_relations(vec![membership, source])
        .unwrap();
    let program = registry
        .bind_managed_index_source(&registry.derived_relation(&identity).unwrap(), vec![], vec![])
        .unwrap();
    (registry, program)
}

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::new(0), slot, 1)
}

fn edge(kind: KindId, created: bool, source: EntityId, target: EntityId) -> AdjacencyDelta {
    AdjacencyDelta {
        relation_id: RelationId::new(PartitionId::new(0), 9, 1),
        kind_id: kind,
        kind: if created {
            AdjacencyDeltaKind::Created { source, target }
        } else {
            AdjacencyDeltaKind::Deleted { source, target }
        },
    }
}

#[test]
fn derived_ordered_source_admits_only_a_born_scope_predecessor() {
    let (registry, program) = ordered_behind_derived();
    assert!(program.0.ordered.is_none(), "the same-scope Source has no compiled recipe");
    let (probed, peer, born, older) = (entity(1), entity(2), entity(3), entity(4));
    let footprint = ManagedIndexParentReadFootprint::default();
    let effect = MutationEffect::default();
    let mut relevance = SourceRelevance::of(&registry, &program, probed, &footprint, &effect);
    assert!(
        relevance.ordered.is_some() && relevance.structural_paths,
        "the derived wrapper resolves to its ordered-events membership"
    );
    relevance.born.insert(born);
    // The born scope's own chain links to a committed peer: no older chain,
    // the probed one included, can reach it.
    assert!(!relevance.adjacency(&edge(PREDECESSOR, true, born, peer)));
    assert!(relevance.other_root(born, SCOPE));
    // An older scope's new predecessor, a removed predecessor, or a new
    // membership event can all reach the probed scope's chain.
    assert!(relevance.adjacency(&edge(PREDECESSOR, true, older, peer)));
    assert!(relevance.adjacency(&edge(PREDECESSOR, false, born, peer)));
    assert!(relevance.adjacency(&edge(MEMBER, true, born, peer)));
    assert!(!relevance.other_root(older, SCOPE));
    assert!(!relevance.other_root(probed, SCOPE));
}

fn updated(entity: EntityId, aspect: &str) -> CanonicalRecordAspectDelta {
    CanonicalRecordAspectDelta {
        target: RecordRef::Entity(entity),
        kind_id: SCOPE,
        plan_revision: crate::schema::data::AspectContractPlanRevision(1),
        structural_change: Change::Updated,
        retained_relation_endpoints: None,
        changed_aspects: crate::publication::patch::data::ordered_aspect_keys([
            AspectKey::new(aspect).unwrap()
        ]),
        evaluated_bindings: smallvec::SmallVec::new(),
        contains_opaque_aspect: false,
    }
}

#[test]
fn parent_fields_and_the_scope_key_are_relevant_on_any_entity_of_their_kind() {
    // A same-scope stored Source projecting nothing: an aspect update on a
    // Source-kind entity is irrelevant to the Source program alone.
    let (registry, program) = crate::tests::support::managed_cache_schema();
    let (probed, other) = (entity(1), entity(2));
    let effect = MutationEffect::default();
    let none = ManagedIndexParentReadFootprint::default();
    let source_only = SourceRelevance::of(&registry, &program, probed, &none, &effect);
    assert!(!source_only.delta(&updated(other, "key")));
    // The Parent reads `key` (a projected field, or the scope key that
    // resolved the probed scope): that write is relevant on any entity of the
    // kind, including another scope, while an unread aspect stays irrelevant.
    let key = ManagedIndexParentReadFootprint {
        fields: vec![(SCOPE, AspectKey::new("key").unwrap(), FieldKey::new("value").unwrap())],
        ..Default::default()
    };
    let parent = SourceRelevance::of(&registry, &program, probed, &key, &effect);
    assert!(parent.delta(&updated(other, "key")));
    assert!(parent.delta(&updated(probed, "key")));
    assert!(!parent.delta(&updated(other, "note")));
}
