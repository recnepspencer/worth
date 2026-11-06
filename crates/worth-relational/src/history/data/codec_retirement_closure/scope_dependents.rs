//! Current owned publications whose exact source would be removed by a candidate.
use super::{charge, CodecRetirementClosureStop as Stop};
use crate::{
    history::data::{CanonicalCommitEnvelope, CanonicalRequiredOutputOrigin, CommitId},
    identity::data::EntityId,
    mvcc::RequiredOutputRolePosture,
};
use std::sync::Arc;
use worth_execution::MapKernelContext;

pub(crate) fn select_owned_scope_dependent(
    removed: &[EntityId],
    mut next: Option<CommitId>,
    bound: usize,
    mut load: impl FnMut(CommitId) -> Option<Arc<CanonicalCommitEnvelope>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<Arc<CanonicalRequiredOutputOrigin>>, Stop> {
    let mut seen: Vec<Arc<CanonicalRequiredOutputOrigin>> = Vec::new();
    for _ in 0..bound {
        let Some(commit) = next else { return Ok(None) };
        charge(context, bound.saturating_add(1))?;
        let envelope = load(commit).ok_or("owned source retirement lost history")?;
        super::require_linear_history(&envelope)?;
        for origin in envelope.output_origins() {
            for old in &seen {
                charge(
                    context,
                    old.meaning.family.len() + origin.meaning.family.len() + 65,
                )?;
            }
            if seen
                .iter()
                .any(|old| old.meaning.same_route(&origin.meaning))
            {
                continue;
            }
            envelope.output_origin_for_route(&origin.meaning, context)?;
            charge(context, removed.len() + origin.members.len() + 1)?;
            if origin.source_scope_owned
                && origin
                    .members
                    .iter()
                    .any(|member| member.posture != RequiredOutputRolePosture::Retire)
                && removed.contains(&origin.meaning.source_scope)
            {
                return Ok(Some(Arc::clone(origin)));
            }
            seen.push(Arc::clone(origin));
        }
        next = envelope
            .required_output_materialization
            .as_ref()
            .map(|event| event.source_commit)
            .or_else(|| envelope.commit.parents.first().copied());
    }
    if next.is_some() {
        return Err("owned source retirement ancestry incomplete or cyclic".into());
    }
    Ok(None)
}
