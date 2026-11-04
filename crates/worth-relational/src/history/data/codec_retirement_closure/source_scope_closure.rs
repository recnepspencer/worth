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
mod coupled;

/// O(H² + P²M), catalog-bounded and charged under the original request.
pub(crate) fn historical_positions(
    mut next: Option<CommitId>,
    bound: usize,
    installed: Option<&RelationalInstalledApplicationPublicationAuthority>,
    load: &mut dyn FnMut(CommitId) -> Option<Arc<CanonicalCommitEnvelope>>,
    validate: &mut dyn FnMut(
        &CanonicalCommitEnvelope,
        &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Failure>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<SelectedOutputPublication>, Failure> {
    let mut latest: Vec<SelectedOutputPublication> = Vec::with_capacity(bound);
    let mut coupled_ancestry = Vec::new();
    for _ in 0..bound {
        let Some(commit) = next else { break };
        charge(context, bound.saturating_add(1))?;
        let envelope = load(commit).ok_or("source-scope closure lost canonical history")?;
        require_linear_history(&envelope)?;
        if envelope.coupled_authored_publication.is_some() || !coupled_ancestry.is_empty() {
            coupled_ancestry.push(Arc::clone(&envelope));
        }
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
                    evidence: crate::publication::ownership::PublicationEvidence::Canonical(
                        Arc::clone(&envelope),
                    ),
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
    coupled::validate_ancestry(&coupled_ancestry, bound, context)?;
    Ok(latest)
}

/// Current ownership is selected solely from authenticated current region facts.
pub(crate) fn current(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
    installed: Option<&RelationalInstalledApplicationPublicationAuthority>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<SelectedOutputPublication>, crate::transactions::data::TransactionCommitError> {
    let roster = Arc::new(crate::publication::ownership::read(
        execution, basis, context,
    )?);
    from_current(roster, installed, context)
}

pub(crate) fn from_current(
    roster: Arc<crate::publication::ownership::CurrentPublicationRoster>,
    installed: Option<&RelationalInstalledApplicationPublicationAuthority>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<SelectedOutputPublication>, crate::transactions::data::TransactionCommitError> {
    let mut selected = Vec::with_capacity(roster.origins().len());
    for (origin, ordinal) in roster.coordinates() {
        crate::execution::request_stage::checkpoint(context, origin.producer.len() as u64 + 1)
            .map_err(crate::publication::ownership::request)?;
        if installed.is_some_and(|installed| {
            installed
                .source_scope_ownership
                .scope_kind(&origin.producer)
                .is_none()
        }) {
            return Err(
                crate::transactions::data::TransactionCommitError::publication(
                    crate::publication::data::PublicationError::authored_retirement(
                        crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
                        crate::transactions::data::RecordRef::Entity(origin.meaning.source_scope),
                    ),
                ),
            );
        }
        selected.push(SelectedOutputPublication {
            evidence: crate::publication::ownership::PublicationEvidence::Current(Arc::clone(
                &roster,
            )),
            origin: Arc::clone(origin),
            ordinal,
        });
    }
    Ok(selected)
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

/// A genuinely historical source birth or inverse selects its recorded prefix.
/// This is never called to enforce current ownership or to rebuild its roster.
pub(crate) fn validate_historical_prefix(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
    envelope: &CanonicalCommitEnvelope,
    source: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), crate::transactions::data::TransactionCommitError> {
    execution
        .consult()
        .map_err(crate::publication::ownership::request)?;
    historical_positions(
        Some(envelope.commit.commit_id),
        runtime.history.catalog_len(),
        None,
        &mut |commit| runtime.history.canonical_envelope(commit),
        &mut |envelope, context| {
            if envelope.required_output_materialization.is_some() {
                super::materialization::validate_semantics(
                    runtime,
                    envelope,
                    context,
                    None,
                    basis.inner.root.schema_authority(),
                )?;
            }
            Ok(())
        },
        context,
    )
    .map(|_| ())
    .map_err(|failure| match failure.stop {
        Stop::Request(cause) => crate::publication::ownership::request(cause),
        Stop::Payload(_) => crate::transactions::data::TransactionCommitError::publication(
            crate::publication::data::PublicationError::authored_retirement(
                crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
                crate::transactions::data::RecordRef::Entity(failure.source.unwrap_or(source)),
            ),
        ),
    })
}
