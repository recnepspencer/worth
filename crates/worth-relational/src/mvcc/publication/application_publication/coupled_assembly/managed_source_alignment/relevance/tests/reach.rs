//! A member's change of a relevant kind is the probed scope's change only on
//! what that scope's Source read before the member: the lists membership
//! read, the records it depended on and the rows it produced.
use super::footprint::{key, root_only, text, updated};
use super::*;
use crate::mvcc::publication::application_publication::coupled_assembly::managed_source_alignment::reach::Reach;

fn row(entity: EntityId, change: Change) -> CanonicalRecordAspectDelta {
    let mut delta = lifecycle(entity, change);
    delta.kind_id = ROW;
    delta
}

/// The probed scope read its own `INTRODUCES` list and produced `rows`.
fn read_by_probed(probed: EntityId, rows: &[EntityId]) -> Option<Reach> {
    Some(Reach {
        entities: rows.iter().copied().collect(),
        anchors: [(probed, INTRODUCES, true)].into(),
    })
}

#[test]
fn a_row_written_under_another_scope_is_not_the_probed_scopes_change() {
    let (registry, program) = root_only();
    let (probed, other, mine, theirs) = (entity(1), entity(2), entity(5), entity(6));
    let footprint = ManagedIndexParentReadFootprint::default();
    let linked = edge(INTRODUCES, true, other, theirs);
    let mut effect = effect(vec![row(theirs, Change::Created)]);
    effect.adjacency.deltas = vec![linked];
    let judge = |reach| {
        SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &|_| None)
            .reached_by(reach)
    };
    // By kind alone the other scope's new row and link are both relevant.
    let by_kind = judge(None);
    assert!(by_kind.adjacency(&linked));
    assert!(by_kind.delta(&row(theirs, Change::Created)));
    assert!(by_kind.undescribed(&RecordRef::Entity(theirs), Some(ROW)));
    // The probed scope's Source read neither that list nor that row.
    let exact = judge(read_by_probed(probed, &[mine]));
    assert!(!exact.adjacency(&linked));
    assert!(!exact.delta(&row(theirs, Change::Created)));
    assert!(!exact.undescribed(&RecordRef::Entity(theirs), Some(ROW)));
}

#[test]
fn a_link_on_a_list_the_source_read_and_a_row_it_produced_stay_relevant() {
    let (registry, program) = root_only();
    let (probed, mine, fresh) = (entity(1), entity(5), entity(7));
    let footprint = ManagedIndexParentReadFootprint::default();
    let linked = edge(INTRODUCES, true, probed, fresh);
    let mut effect = effect(Vec::new());
    effect.adjacency.deltas = vec![linked];
    let exact = SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &|_| None)
        .reached_by(read_by_probed(probed, &[mine]));
    // A new member of the probed scope, and a change to a row it had.
    assert!(exact.adjacency(&linked));
    assert!(exact.delta(&row(mine, Change::Deleted)));
    assert!(exact.undescribed(&RecordRef::Entity(mine), Some(ROW)));
    // The row the link introduces is judged by that link, not by itself.
    assert!(!exact.delta(&row(fresh, Change::Created)));
}

#[test]
fn a_nested_parent_relation_keeps_every_record_of_a_relevant_kind() {
    let (registry, program) = root_only();
    let (probed, mine, theirs) = (entity(1), entity(5), entity(6));
    // The Parent reads nested results the Source's reach does not hold.
    let footprint = ManagedIndexParentReadFootprint {
        relations: vec![MEMBER],
        entities: vec![ROW],
        ..Default::default()
    };
    let effect = effect(Vec::new());
    let exact = SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &|_| None)
        .reached_by(read_by_probed(probed, &[mine]));
    assert!(exact.delta(&row(theirs, Change::Deleted)));
}

/// The probed scope answers to `peer`; every other scope to `sibling`.
fn keyed(
    probed: EntityId,
    sibling: &'static str,
) -> impl Fn(EntityId) -> Option<worth_foundational::facade::AspectValue> {
    move |id| Some(text(if id == probed { "peer" } else { sibling }))
}

fn keyed_footprint() -> ManagedIndexParentReadFootprint {
    ManagedIndexParentReadFootprint {
        fields: vec![key()],
        scope_key: Some(key()),
        ..Default::default()
    }
}

#[test]
fn a_sibling_scope_taking_the_probed_key_is_relevant_outside_the_reach() {
    let (registry, program) = root_only();
    let (probed, sibling, mine) = (entity(1), entity(2), entity(5));
    let (footprint, effect) = (keyed_footprint(), effect(Vec::new()));
    // The sibling's key was rewritten to the probed key: the key no longer
    // names the probed scope alone, though its Source never read the sibling.
    let taken = keyed(probed, "peer");
    let exact = SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &taken)
        .reached_by(read_by_probed(probed, &[mine]));
    assert!(exact.delta(&updated(sibling, "other", "peer", "")));
    // A changed sibling Native did not describe, now holding that key.
    assert!(exact.undescribed(&RecordRef::Entity(sibling), Some(SCOPE)));
    // A sibling whose key stays distinct is not the probed scope's change.
    let apart = keyed(probed, "elsewhere");
    let exact = SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &apart)
        .reached_by(read_by_probed(probed, &[mine]));
    assert!(!exact.delta(&updated(sibling, "other", "elsewhere", "")));
    assert!(!exact.undescribed(&RecordRef::Entity(sibling), Some(SCOPE)));
}

#[test]
fn a_restored_sibling_with_the_probed_key_is_relevant_where_scopes_are_not_roots_only() {
    // An ordered-events Source: an existing sibling is no root-only scope
    // and was not born here, so no scope exemption applies to it.
    let (registry, program) = behind_derived(ROW, events(ROW, INTRODUCES));
    let (probed, sibling, mine) = (entity(1), entity(2), entity(5));
    let (footprint, effect) = (keyed_footprint(), effect(Vec::new()));
    let taken = keyed(probed, "peer");
    let exact = SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &taken)
        .reached_by(read_by_probed(probed, &[mine]));
    assert!(exact.delta(&lifecycle(sibling, Change::Rematerialized)));
    assert!(exact.delta(&lifecycle(sibling, Change::Created)));
}

const TAGGED: KindId = KindId(7);

/// Membership `SCOPE -INTRODUCES-> ROW` with one projection route
/// `ROW -TAGGED-> SCOPE`: the route reads lists membership never reads.
fn routed() -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    let registry = event_registry();
    let mut tagged = registry.relation_registration(MEMBER).unwrap().clone();
    tagged.kind_id = TAGGED;
    tagged.kind_name = "row-tagged".into();
    tagged.aspect_contract_declarations = Default::default();
    tagged.relation_integrity = RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: "row-tagged-endpoints".into(),
            allowed_source_kinds: vec![ROW],
            allowed_target_kinds: vec![SCOPE],
            self_edges_allowed: false,
            cross_context_policy: CrossContextPolicy::Forbid,
        }],
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let registry = registry.register_relation_kind(tagged).unwrap();
    let (_, template) = crate::tests::support::managed_cache_schema();
    let mut source = template.0.path.parts().clone();
    source.identity.name = "routed-source".into();
    source.to = ROW.into();
    source.nodes = vec![Node::Stored { relation: INTRODUCES, from: SCOPE, to: ROW }];
    let mut route = source.clone();
    route.identity.name = "row-tag".into();
    route.from = ROW.into();
    route.to = SCOPE.into();
    route.nodes = vec![Node::Stored { relation: TAGGED, from: ROW, to: SCOPE }];
    let (source_identity, route_identity) = (source.identity.clone(), route.identity.clone());
    let registry = registry.register_derived_relations(vec![source, route]).unwrap();
    let route = crate::validation::candidate_source::ManagedIndexSourceProjectionRoute::new(
        "tag".into(),
        registry.derived_relation(&route_identity).unwrap(),
        vec![],
    )
    .unwrap();
    let program = registry
        .bind_managed_index_source(
            &registry.derived_relation(&source_identity).unwrap(),
            vec![],
            vec![route],
        )
        .unwrap();
    (registry, program)
}

#[test]
fn a_projection_route_edge_stays_relevant_under_a_membership_reach() {
    let (registry, program) = routed();
    let (probed, mine, tag) = (entity(1), entity(5), entity(8));
    let footprint = ManagedIndexParentReadFootprint::default();
    // A row of the probed scope gains a tag: its projected content changes,
    // on a list only the route reads.
    let tagged = edge(TAGGED, true, mine, tag);
    let mut effect = effect(Vec::new());
    effect.adjacency.deltas = vec![tagged];
    let exact = SourceRelevance::of(&registry, &program, probed, &footprint, &effect, &|_| None)
        .reached_by(read_by_probed(probed, &[mine]));
    assert!(exact.adjacency(&tagged));
    assert!(exact.undescribed(&RecordRef::Relation(tagged.relation_id), Some(TAGGED)));
}
