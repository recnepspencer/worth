//! One recorded-fact closure owner for authored and producer-triggered source removal.
use super::{
    charge, require_linear_history, CodecRetirementClosureStop as Stop,
    PublicationHistoryFailure as Failure,
};
use crate::{
    history::data::{CanonicalCommitEnvelope, CommitId, SelectedOutputPublication},
    identity::data::EntityId,
    mvcc::{
        RelationalInstalledApplicationPublicationAuthority, RequiredOutputRolePosture as Posture,
    },
};
use std::sync::Arc;
use worth_execution::MapKernelContext;

/// O(H² + P²M), catalog-bounded and charged under the original request.
pub(crate) fn current(
    mut next: Option<CommitId>,
    bound: usize,
    installed: Option<&RelationalInstalledApplicationPublicationAuthority>,
    load: &mut dyn FnMut(CommitId) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
    validate: &mut dyn FnMut(
        &CanonicalCommitEnvelope,
        &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Failure>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<SelectedOutputPublication>, Failure> {
    let mut latest: Vec<SelectedOutputPublication> = Vec::with_capacity(bound);
    for _ in 0..bound {
        let Some(commit) = next else { break };
        charge(context, bound.saturating_add(1))?;
        let envelope = load(commit)?.ok_or("source-scope closure lost canonical history")?;
        require_linear_history(&envelope)?;
        for (ordinal, origin) in envelope.output_origins().enumerate() {
            charge(context, latest.len() + 1)?;
            let work = latest.iter().fold(origin.members.len(), |sum, prior| {
                sum.saturating_add(origin.meaning.family.len())
                    .saturating_add(prior.origin.meaning.family.len() + 64)
            });
            charge(context, work)?;
            envelope
                .output_origin_for_route(&origin.meaning, context)
                .map_err(|stop| Failure::at_source(origin.meaning.source_scope, stop))?;
            if !latest
                .iter()
                .any(|prior| prior.origin.meaning.same_route(&origin.meaning))
            {
                if latest.len() >= bound {
                    return Err("source-scope catalog bound exceeded".into());
                }
                if let Some(installed) = installed {
                    charge(
                        context,
                        installed
                            .source_scope_ownership
                            .lookup_work(&origin.producer),
                    )?;
                    if installed
                        .source_scope_ownership
                        .scope_kind(&origin.producer)
                        .is_some()
                        != origin.source_scope_owned
                    {
                        return Err(Failure::at_source(
                            origin.meaning.source_scope,
                            "source-scope publication differs from installed ownership".into(),
                        ));
                    }
                }
                latest.push(SelectedOutputPublication {
                    envelope: Arc::clone(&envelope),
                    origin: Arc::clone(origin),
                    ordinal,
                });
            }
        }
        validate(&envelope, context)?;
        next = envelope
            .required_output_materialization
            .as_ref()
            .map(|event| event.source_commit)
            .or_else(|| envelope.commit.parents.first().copied());
    }
    if next.is_some() {
        return Err("source-scope ancestry incomplete or cyclic".into());
    }
    Ok(latest)
}

pub(crate) fn select(
    roots: &[EntityId],
    latest: &[SelectedOutputPublication],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<SelectedOutputPublication>, Stop> {
    let mut selected: Vec<SelectedOutputPublication> = Vec::with_capacity(latest.len());
    for _ in 0..=latest.len() {
        let mut advanced = false;
        for publication in latest {
            let origin = &publication.origin;
            charge(
                context,
                roots.len().saturating_sub(1)
                    + selected.len()
                    + origin.members.len()
                    + origin.producer.len()
                    + 1,
            )?;
            if selected
                .iter()
                .any(|prior| Arc::ptr_eq(&prior.origin, origin))
                || !origin.source_scope_owned
                || origin.members.iter().all(|m| m.posture == Posture::Retire)
            {
                continue;
            }
            let source = origin.meaning.source_scope;
            let mut in_closure = roots.contains(&source);
            for prior in &selected {
                charge(context, prior.origin.members.len())?;
                in_closure |= prior
                    .origin
                    .members
                    .iter()
                    .any(|m| m.id == source && m.posture != Posture::Retire);
            }
            if in_closure {
                selected.push(publication.clone());
                advanced = true;
            }
        }
        if !advanced {
            break;
        }
    }
    Ok(selected)
}

pub(crate) fn validate_preparation_history(
    envelope: &CanonicalCommitEnvelope,
    load: &mut impl FnMut(CommitId) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
    bound: usize,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    if envelope.required_output_materialization.is_some() {
        super::materialization::validate_header(envelope, load, bound, context)?;
    }
    Ok(())
}
