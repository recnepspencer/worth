//! A member's change of a relevant kind is the probed scope's change only on
//! what that scope's Source read before the member: the lists membership
//! read, the records it depended on and the rows it produced.
use super::footprint::root_only;
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
