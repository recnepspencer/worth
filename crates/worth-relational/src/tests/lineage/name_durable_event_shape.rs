//! Arch 8, 10, 21: a persisted lineage event whose arity, receipt branch, or
//! naming payload is not canonical for its kind refuses typed and yields no
//! index, so a tampered `Retire` can never rebind a retired name.

use crate::facade::history::{BranchId, CommitId};
use crate::facade::lineage::{
    LineageEventKind, LineageEventRecord, PersistentNameCommittedBatch,
    PersistentNameDurabilityError,
};
use worth_relational_contracts::internal_port::lineage::data::events::LineageEventRecordFieldsPort as _;

use super::name_durable_world::{durable_naming_history, DurableNamingHistory};

type Commits = Vec<(PersistentNameCommittedBatch, Option<CommitId>)>;

/// Mutates the first event matching `select`, rebuilds, and requires the
/// typed shape refusal naming that event.
fn refuses_shape(
    history: &DurableNamingHistory,
    select: impl Fn(&LineageEventRecord) -> bool,
    mutate: impl FnOnce(&mut LineageEventRecord, &Commits),
    detail: &'static str,
) {
    let mut event_id = None;
    let refused = history.rebuild_mutated(|commits| {
        let snapshot = commits.clone();
        let event = commits
            .iter_mut()
            .flat_map(|(batch, _)| batch.lineage_events.iter_mut())
            .find(|event| select(event))
            .expect("the history carries the selected event");
        mutate(event, &snapshot);
        event_id = Some(event.event_id());
    });
    assert_eq!(
        refused,
        Err(PersistentNameDurabilityError::InvalidEventShape {
            event_id: event_id.unwrap(),
            detail,
        })
    );
}

fn kind(kind: LineageEventKind) -> impl Fn(&LineageEventRecord) -> bool {
    move |event| event.kind() == kind
}

/// Some lineage the log allocated, for corrupt arity.
fn any_created_lineage(commits: &Commits) -> crate::facade::identity::LineageId {
    commits[1]
        .0
        .lineage_events
        .iter()
        .find(|event| event.kind() == LineageEventKind::Create)
        .unwrap()
        .targets()[0]
}

#[test]
fn non_canonical_arity_for_each_kind_refuses_typed() {
    let history = durable_naming_history();
    refuses_shape(
        &history,
        kind(LineageEventKind::Retire),
        |event, commits| (*event.owner_targets_mut()) = vec![any_created_lineage(commits)],
        "a Retire has exactly one source and no targets",
    );
    refuses_shape(
        &history,
        kind(LineageEventKind::Create),
        |event, commits| (*event.owner_sources_mut()) = vec![any_created_lineage(commits)],
        "a Create has no sources and exactly one target",
    );
    refuses_shape(
        &history,
        kind(LineageEventKind::Split),
        |event, _| (*event.owner_targets_mut()).truncate(1),
        "a Split has one source and at least two distinct other targets",
    );
    refuses_shape(
        &history,
        kind(LineageEventKind::Split),
        |event, _| (*event.owner_targets_mut())[1] = (*event.owner_sources())[0],
        "a Split has one source and at least two distinct other targets",
    );
    refuses_shape(
        &history,
        kind(LineageEventKind::Merge),
        |event, _| (*event.owner_sources_mut()).truncate(1),
        "a Merge has at least two distinct sources and exactly one target",
    );
    refuses_shape(
        &history,
        kind(LineageEventKind::Merge),
        |event, _| (*event.owner_sources_mut())[1] = (*event.owner_sources())[0],
        "a Merge has at least two distinct sources and exactly one target",
    );
    refuses_shape(
        &history,
        kind(LineageEventKind::Restore),
        |event, _| (*event.owner_targets_mut()) = (*event.owner_sources()).clone(),
        "a Restore or Replace has one source and one different target",
    );
}

#[test]
fn receipt_branch_mismatch_refuses_typed() {
    let history = durable_naming_history();
    refuses_shape(
        &history,
        kind(LineageEventKind::Split),
        |event, _| (*event.owner_commit_mut()).branch_id = BranchId("forged".to_owned()),
        "the receipt names a different branch than the event",
    );
}

#[test]
fn births_and_kept_names_on_kinds_that_cannot_carry_them_refuse_typed() {
    let history = durable_naming_history();
    let birth = history.commits[1]
        .0
        .lineage_events
        .iter()
        .find_map(|event| event.name_birth().cloned())
        .unwrap();
    let kept = history
        .commits
        .iter()
        .flat_map(|(batch, _)| batch.lineage_events.iter())
        .find_map(LineageEventRecord::kept_name)
        .unwrap();
    refuses_shape(
        &history,
        kind(LineageEventKind::Retire),
        |event, _| (*event.owner_name_birth_mut()) = Some(birth),
        "only a Create can carry a name birth",
    );
    for carrier in [LineageEventKind::Retire, LineageEventKind::Split] {
        refuses_shape(
            &history,
            kind(carrier),
            |event, _| (*event.owner_kept_name_mut()) = Some(kept),
            "only Create, Replace, Merge, or Restore can keep a name",
        );
    }
}
