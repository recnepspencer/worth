//! Native relevance on real registered Sources: an ordered-events Source
//! behind one derived relation admits only a born scope's predecessor, and
//! only for a shape where nothing else reads the scope or the order relation.
use super::*;
use crate::authority::mutation::{AdjacencyDelta, AdjacencyDeltaKind, MutationEffect};
use crate::derived_relation::DerivedRelationNode as Node;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::publication::patch::data::RecordStructuralChange as Change;
use crate::config::data::CrossContextPolicy;
use crate::schema::data::{
    EndpointKindContractDeclaration, EntityKindRegistration, RelationIntegrityDeclarations,
    RelationalSchemaRegistry,
};
mod footprint;

const SCOPE: KindId = KindId(1);
const MEMBER: KindId = KindId(3);
const PREDECESSOR: KindId = KindId(4);
const INTRODUCES: KindId = KindId(5);
const ROW: KindId = KindId(6);

/// The registry with a row kind and the relations an event chain needs.
fn event_registry() -> RelationalSchemaRegistry {
    let (registry, _) = crate::tests::support::managed_cache_schema();
    let template = registry.relation_registration(MEMBER).unwrap().clone();
    let mut order = template.clone();
    order.kind_id = PREDECESSOR;
    order.kind_name = "event-predecessor".into();
    order.aspect_contract_declarations = Default::default();
    let mut introduces = order.clone();
    introduces.kind_id = INTRODUCES;
    introduces.kind_name = "event-introduces".into();
    introduces.relation_integrity = RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: "event-introduces-endpoints".into(),
            allowed_source_kinds: vec![SCOPE],
            allowed_target_kinds: vec![ROW],
            self_edges_allowed: false,
            cross_context_policy: CrossContextPolicy::Forbid,
        }],
        vec![],
        vec![],
        vec![],
        vec![],
    );
    registry
        .register_entity_kind(EntityKindRegistration {
            kind_id: ROW,
            kind_name: "event-row".into(),
            schema_id: template.schema_id.clone(),
            schema_version_id: template.schema_version_id,
            aspect_contract_declarations: Default::default(),
        })
        .unwrap()
        .register_relation_kind(order)
        .unwrap()
        .register_relation_kind(introduces)
        .unwrap()
}

/// A Source whose root path is `[Derived(source)]` over `nodes`, from the
/// scope to `row`; its compiled ordered recipe is absent.
fn behind_derived(row: KindId, nodes: Vec<Node>) -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    let registry = event_registry();
    let (_, template) = crate::tests::support::managed_cache_schema();
    let mut membership = template.0.path.parts().clone();
    membership.identity.name = "event-membership".into();
    membership.to = row.into();
    membership.nodes = nodes;
    let mut source = membership.clone();
    source.identity.name = "event-source".into();
    source.nodes = vec![Node::Derived(membership.identity.clone())];
    let identity = source.identity.clone();
    let registry = registry.register_derived_relations(vec![membership, source]).unwrap();
    let program = registry
        .bind_managed_index_source(&registry.derived_relation(&identity).unwrap(), vec![], vec![])
        .unwrap();
    (registry, program)
}

fn events(row: KindId, introduced: KindId) -> Vec<Node> {
    vec![
        Node::Stored { relation: PREDECESSOR, from: SCOPE, to: SCOPE },
        Node::Stored { relation: introduced, from: SCOPE, to: row },
        Node::Stored { relation: introduced, from: SCOPE, to: row },
        Node::OrderedEvents { predecessor: 0, introduced: 1, retired: 2 },
    ]
}

pub(super) fn entity(slot: u64) -> EntityId {
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

pub(super) fn lifecycle(entity: EntityId, change: Change) -> CanonicalRecordAspectDelta {
    CanonicalRecordAspectDelta {
        target: RecordRef::Entity(entity),
        kind_id: SCOPE,
        plan_revision: crate::schema::data::AspectContractPlanRevision(1),
        structural_change: change,
        retained_relation_endpoints: None,
        changed_aspects: Vec::new(),
        evaluated_bindings: smallvec::SmallVec::new(),
        contains_opaque_aspect: false,
    }
}

/// The candidate's effect with these entity deltas.
pub(super) fn effect(deltas: Vec<CanonicalRecordAspectDelta>) -> MutationEffect {
    let mut effect = MutationEffect::default();
    effect.publication.canonical_deltas = deltas;
    effect
}

pub(super) fn unread(_: EntityId) -> Option<worth_foundational::facade::AspectValue> {
    None
}

#[test]
fn derived_ordered_source_admits_only_a_born_scope_predecessor() {
    let (registry, program) = behind_derived(ROW, events(ROW, INTRODUCES));
    assert!(program.0.ordered.is_none(), "the same-scope Source has no compiled recipe");
    let (probed, peer, born, older) = (entity(1), entity(2), entity(3), entity(4));
    let footprint = ManagedIndexParentReadFootprint::default();
    // The candidate created `born`: its delta, not the court, marks it.
    let effect = effect(vec![lifecycle(born, Change::Created)]);
    let relevance = SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &unread);
    assert!(
        relevance.ordered.is_some() && relevance.born.contains(&born),
        "the derived wrapper resolves to its ordered-events membership"
    );
    assert!(!relevance.adjacency(&edge(PREDECESSOR, true, born, peer)));
    assert!(relevance.other_root(born, SCOPE));
    assert!(!relevance.delta(&lifecycle(born, Change::Created)));
    // An older scope's new predecessor, a removed predecessor, or a new
    // membership event can all reach the probed scope's chain.
    assert!(relevance.adjacency(&edge(PREDECESSOR, true, older, peer)));
    assert!(relevance.adjacency(&edge(PREDECESSOR, false, born, peer)));
    assert!(relevance.adjacency(&edge(INTRODUCES, true, born, peer)));
    assert!(!relevance.other_root(older, SCOPE));
    assert!(!relevance.other_root(probed, SCOPE));
}

#[test]
fn ordered_rule_is_off_when_a_scope_is_a_row_or_the_order_is_an_event() {
    let (probed, peer, born) = (entity(1), entity(2), entity(3));
    let footprint = ManagedIndexParentReadFootprint::default();
    let effect = effect(vec![lifecycle(born, Change::Created)]);
    // Scope rows, and an order relation reused as the membership event.
    for (row, introduced) in [(SCOPE, MEMBER), (SCOPE, PREDECESSOR)] {
        let (registry, program) = behind_derived(row, events(row, introduced));
        let relevance =
            SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &unread);
        assert!(relevance.ordered.is_none(), "{row:?}/{introduced:?}");
        assert!(relevance.adjacency(&edge(PREDECESSOR, true, born, peer)));
        assert!(relevance.delta(&lifecycle(born, Change::Created)));
    }
}
