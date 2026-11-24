//! One recorded-fact closure owner for authored and producer-triggered source removal.
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;
use super::{
    charge, require_linear_history, CodecRetirementClosureStop as Stop,
    PublicationHistoryFailure as Failure,
};
use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::CommitId;
use crate::history::data::SelectedOutputPublication;
use crate::identity::data::EntityId;
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture as Posture;
use crate::schema::installed_source_scopes::InstalledSourceScopeOwnership;
use std::sync::Arc;
use worth_execution::MapKernelContext;
pub mod coupled;
pub use coupled::tests::assert_exact_owner_refusals_for_test;

/// O(H^2 + P^2 M), catalog-bounded and charged under the original request.
pub fn historical_positions(
    mut next: Option<CommitId>,
    bound: usize,
    load: &mut dyn FnMut(CommitId) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
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
        let envelope = load(commit)?.ok_or("source-scope closure lost canonical history")?;
        require_linear_history(&envelope)?;
        if envelope.coupled_authored_publication.is_some() || !coupled_ancestry.is_empty() {
            coupled_ancestry.push(Arc::clone(&envelope));
        }
        let validated = envelope
            .validate_output_origins(context)
            .map_err(|stop| match envelope.output_origins().next() {
                Some(origin) => Failure::at_source(origin.meaning.source_scope, stop),
                None => Failure::from(stop),
            })?;
        for (ordinal, origin) in envelope.output_origins().enumerate() {
            charge(context, latest.len() + 1)?;
            let work = latest.iter().fold(origin.members.len(), |sum, prior| {
                sum.saturating_add(origin.meaning.family.len())
                    .saturating_add(prior.origin.meaning.family.len() + 64)
            });
            charge(context, work)?;
            validated
                .output_origin_for_route(&origin.meaning, context)
                .map_err(|stop| Failure::at_source(origin.meaning.source_scope, stop))?;
            if !latest
                .iter()
                .any(|prior| prior.origin.meaning.same_route(&origin.meaning))
            {
                if latest.len() >= bound {
                    return Err("source-scope catalog bound exceeded".into());
                }
                latest.push(SelectedOutputPublication {
                    evidence:
                        crate::publication::ownership::evidence::PublicationEvidence::Canonical(
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

pub fn from_current(
    roster: Arc<crate::publication::ownership::read::roster::CurrentPublicationRoster>,
    ownership: Option<&InstalledSourceScopeOwnership>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<
    Vec<SelectedOutputPublication>,
    crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    let mut selected = Vec::with_capacity(roster.origins().len());
    for (origin, ordinal) in roster.coordinates() {
        crate::execution::request_stage::checkpoint(context, origin.producer.len() as u64 + 1)
            .map_err(crate::publication::ownership::read_request::request)?;
        if ownership.is_some_and(|ownership| ownership.scope_kind(&origin.producer).is_none()) {
            return Err(
                crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                    crate::publication::data::PublicationError::authored_retirement(
                        crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
                        crate::transactions::data::primitives::RecordRef::Entity(origin.meaning.source_scope),
                    ),
                ),
            );
        }
        selected.push(SelectedOutputPublication {
            evidence: crate::publication::ownership::evidence::PublicationEvidence::Current(
                Arc::clone(&roster),
            ),
            origin: Arc::clone(origin),
            ordinal,
        });
    }
    Ok(selected)
}

pub fn select(
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

/// The exact source-owned closure for a corrective envelope's later stage.
/// Current business-neighbour reads supply candidates; history supplies the
/// preceding canonical catalog. Both use this same finite semantic selection.
pub fn later_source_owners(
    source: EntityId,
    facts: &[SelectedOutputPublication],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<SelectedOutputPublication>, Stop> {
    let mut selected = select(&[source], facts, context)?;
    let linked = super::scope_dependents::linked_owners(&[source], &selected, facts, context)?;
    for owner in linked {
        charge(context, selected.len() + 1)?;
        if !selected
            .iter()
            .any(|known| known.origin.same_publication(&owner.origin))
        {
            selected.push(owner);
        }
    }
    Ok(selected)
}

/// A genuinely historical source birth or inverse selects its recorded prefix.
/// This is never called to enforce current ownership or to rebuild its roster.
pub fn validate_historical_prefix(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
    envelope: &CanonicalCommitEnvelope,
    source: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError>
{
    execution
        .consult()
        .map_err(crate::publication::ownership::read_request::request)?;
    execution
        .in_scope(|lease| {
            historical_positions(
                Some(envelope.commit.commit_id),
                runtime.input.projection.history.catalog_len(),
                &mut |commit| runtime.commit(commit),
                &mut |envelope, context| {
                    if envelope.required_output_materialization.is_some() {
                        super::materialization::validate_semantics(
                            runtime,
                            envelope,
                            context,
                            lease,
                            basis.inner.root.schema_authority(),
                        )?;
                    }
                    Ok(())
                },
                context,
            )
            .map(|_| ())
        })
        .map_err(crate::publication::ownership::read_request::request)?
        .map_err(|failure| match failure.stop {
            Stop::Request(cause) => crate::publication::ownership::read_request::request(cause),
            Stop::Payload(detail) => {
                let mut error = crate::publication::data::PublicationError::authored_retirement(
                    crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
                    crate::transactions::data::primitives::RecordRef::Entity(failure.source.unwrap_or(source)),
                );
                error.detail.push_str(": ");
                error.detail.push_str(&detail);
                crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(error)
            }
        })
}
