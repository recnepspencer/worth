//! Current owned publications whose exact source would be removed by a candidate.
use super::{charge, CodecRetirementClosureStop as Stop, PublicationHistoryFailure as Failure};
use crate::{
    history::data::{CanonicalCommitEnvelope, CanonicalRequiredOutputOrigin, CommitId},
    identity::data::EntityId,
    mvcc::RequiredOutputRolePosture,
};
use std::sync::Arc;
use worth_execution::MapKernelContext;

pub(crate) enum OwnedScopeDependentSelection<'a> {
    RemovedSources(&'a [EntityId]),
    LiveRecord(
        &'a mut dyn FnMut(
            &CanonicalRequiredOutputOrigin,
            &mut MapKernelContext<'_, '_>,
        ) -> Result<bool, Stop>,
    ),
}

/// One catalog walker owns both source removal and live-record custody selection.
pub(crate) fn select_owned_scope_dependent(
    selection: OwnedScopeDependentSelection<'_>,
    next: Option<CommitId>,
    bound: usize,
    load: impl FnMut(CommitId) -> Option<Arc<CanonicalCommitEnvelope>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<Arc<CanonicalRequiredOutputOrigin>>, Failure> {
    match selection {
        OwnedScopeDependentSelection::RemovedSources(removed) => {
            select(next, bound, load, context, true, |origin, context| {
                charge(context, removed.len() + origin.members.len() + 1)?;
                Ok(removed.contains(&origin.meaning.source_scope))
            })
        }
        OwnedScopeDependentSelection::LiveRecord(matches) => {
            select(next, bound, load, context, false, matches)
        }
    }
}

fn select(
    next: Option<CommitId>,
    bound: usize,
    mut load: impl FnMut(CommitId) -> Option<Arc<CanonicalCommitEnvelope>>,
    context: &mut MapKernelContext<'_, '_>,
    linear: bool,
    mut matches: impl FnMut(
        &CanonicalRequiredOutputOrigin,
        &mut MapKernelContext<'_, '_>,
    ) -> Result<bool, Stop>,
) -> Result<Option<Arc<CanonicalRequiredOutputOrigin>>, Failure> {
    let mut seen: Vec<Arc<CanonicalRequiredOutputOrigin>> = Vec::new();
    let mut pending = next.into_iter().collect::<Vec<_>>();
    let mut visited = Vec::new();
    while let Some(commit) = pending.pop() {
        charge(context, visited.len() + 1)?;
        if visited.contains(&commit) {
            if linear {
                return Err("owned source retirement ancestry is cyclic".into());
            }
            continue;
        }
        if visited.len() >= bound {
            return Err("owned publication ancestry exceeds its catalog".into());
        }
        visited.push(commit);
        charge(context, bound.saturating_add(1))?;
        let envelope = load(commit).ok_or("owned source retirement lost history")?;
        if linear {
            super::require_linear_history(&envelope)?;
        }
        for origin in envelope.output_origins() {
            if !linear && !origin.source_scope_owned {
                continue;
            }
            for old in &seen {
                charge(
                    context,
                    old.meaning.family.len() + origin.meaning.family.len() + 65,
                )?;
            }
            if linear
                && seen
                    .iter()
                    .any(|old| old.meaning.same_route(&origin.meaning))
            {
                continue;
            }
            envelope
                .output_origin_for_route(&origin.meaning, context)
                .map_err(|stop| Failure::at_source(origin.meaning.source_scope, stop))?;
            let matched = matches(origin, context)?;
            if (!linear || origin.source_scope_owned)
                && origin
                    .members
                    .iter()
                    .any(|member| member.posture != RequiredOutputRolePosture::Retire)
                && matched
            {
                return Ok(Some(Arc::clone(origin)));
            }
            if linear {
                seen.push(Arc::clone(origin));
            }
        }
        if let Some(event) = &envelope.required_output_materialization {
            pending.push(event.source_commit);
        } else if linear {
            pending.extend(envelope.commit.parents.first().copied());
        } else {
            charge(context, envelope.commit.parents.len())?;
            pending.extend(envelope.commit.parents.iter().copied());
        }
    }
    Ok(None)
}
