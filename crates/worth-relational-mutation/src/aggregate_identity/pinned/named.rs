//! Project canonical group reads into the existing aggregate read vocabulary.
use super::super::{AggregateGroupMembers, AggregateLineageEvent, AggregateLineageKind};
use super::*;
use crate::contracts::aggregate_identity::group_lineage::{
    AggregateGroupHistory, AggregateGroupLineageEvent as Event,
};
use crate::contracts::history::data::coordinates::CommitId;
use std::collections::BTreeSet;
use worth_foundational::facade::AggregateIdentity;

/// Complete selected group and its incident canonical lineage.
pub struct NamedAggregateRead {
    pub identity: AggregateIdentity,
    pub members: AggregateGroupMembers,
    pub history: AggregateGroupHistory,
}

pub(super) fn read(
    definition: AggregateDefinitionIdentity,
    selected: Result<Option<NamedAggregateRead>, Cause>,
    work: &mut funding::AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<result::Value> {
    let selected = require(selected, work)?;
    let mut members = BTreeSet::new();
    for (_, member) in selected.members.iter() {
        work.retain(extent::tree_entry::<NativeMemberKey, ()>())?;
        members.insert(member.carrier.clone());
    }
    Ok(result::Value {
        definition,
        identity: selected.identity,
        members,
        lineage: edges(selected.history.events, work)?,
    })
}

pub(super) fn lineage(
    definition: AggregateDefinitionIdentity,
    identity: AggregateIdentity,
    read: impl Fn(AggregateIdentity) -> Result<Option<AggregateGroupHistory>, Cause>,
    work: &mut funding::AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<lineage::Value> {
    let initial = require(read(identity), work)?;
    let incident = edges(initial.events, work)?;
    let mut result = lineage::Value {
        definition,
        identity,
        retired: initial.retired,
        incident,
        steps: Vec::new(),
        latest: None,
    };
    let mut current = identity;
    let mut retired = initial.retired;
    let mut events = &result.incident;
    let mut subsequent;
    let mut seen = BTreeSet::new();
    while let Some(at) = retired {
        work.retain(extent::tree_entry::<AggregateIdentity, ()>())?;
        if !seen.insert(current) {
            return work.invalid(PlanDenial::LineageCycle);
        }
        let Some(edge) = events.iter().find(|edge| {
            edge.kind == AggregateLineageKind::MergedInto
                && edge.from == current
                && edge.occurrence == at
        }) else {
            return Ok(result);
        };
        work.retain(extent::vector_entry::<AggregateLineageEvent>())?;
        result.steps.push(*edge);
        current = edge.to;
        let next = require(read(current), work)?;
        retired = next.retired;
        subsequent = edges(next.events, work)?;
        events = &subsequent;
    }
    result.latest = Some(current);
    Ok(result)
}

fn require<T>(
    value: Result<Option<T>, Cause>,
    work: &funding::AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<T> {
    match value {
        Ok(Some(value)) => Ok(value),
        Ok(None) => work.invalid(PlanDenial::IdentityUnavailable),
        Err(cause) => work.invalid(PlanDenial::Execution(cause)),
    }
}

fn edges(
    events: Vec<(CommitId, Event)>,
    work: &mut funding::AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<BTreeSet<AggregateLineageEvent>> {
    let mut edges = BTreeSet::new();
    for (occurrence, event) in events {
        work.checkpoint(1)?;
        let (kind, from, to) = match event {
            Event::SplitFrom { source, fragment } => {
                (AggregateLineageKind::SplitFrom, source, fragment)
            }
            Event::MergedInto { absorbed, keeper } => {
                (AggregateLineageKind::MergedInto, absorbed, keeper)
            }
            _ => continue,
        };
        work.retain(extent::tree_entry::<AggregateLineageEvent, ()>())?;
        edges.insert(AggregateLineageEvent {
            occurrence,
            kind,
            from,
            to,
        });
    }
    Ok(edges)
}
