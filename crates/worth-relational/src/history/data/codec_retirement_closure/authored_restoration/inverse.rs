//! Assemble inverse intents from exact retained publication origins.
use super::{creation_patch, failure};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::canonical_commit_envelope::OutputOriginHistory;
use crate::history::data::{CanonicalAuthoredRetirement, CanonicalCommitEnvelope};
use crate::identity::data::{EntityId, RelationId};
use crate::mvcc::publication::application_publication::RelationalRequiredOutputPublication;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::storage::overlay::PartitionAccess;
use crate::symbols::data::ClientKey;
use crate::transactions::data::intents::mutation_intent::{
    CreateIntent, EntityAspectCreateIntent, MutationIntent, RelationAspectCreateIntent,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::{
    CreatedEntityRef, CreatedRelationRef, EntityReference,
};
use worth_execution::{ExecutionResourceLease, MapKernelContext};
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::{
    self as closure, authored_cascade::reject, charge, history_request::RuntimeHistoryRequest,
};
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

pub(super) struct RestorationSelection<'a> {
    pub(super) basis: &'a AdmittedRelationalBranchBasis,
    pub(super) envelope: &'a CanonicalCommitEnvelope,
    pub(super) event: &'a CanonicalAuthoredRetirement,
    pub(super) original: EntityId,
    pub(super) anchor: &'a CreatedEntityRef,
    pub(super) co_anchors: &'a [(EntityId, CreatedEntityRef)],
    pub(super) publications: &'a [RelationalRequiredOutputPublication],
}

pub(super) struct InverseRestoration {
    pub(super) entities: Vec<(EntityId, CreatedEntityRef)>,
    pub(super) relations: Vec<(RelationId, CreatedRelationRef)>,
    pub(super) inverse: Vec<MutationIntent>,
    pub(super) content: crate::content::RelationalContentAttachments,
    pub(super) aggregate_memberships: Vec<crate::aggregate_identity::RestoredMembership>,
}

pub(super) fn prepare(
    selection: RestorationSelection<'_>,
    history: &RuntimeHistoryRequest<'_, '_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<InverseRestoration, TransactionCommitError> {
    let RestorationSelection {
        basis,
        envelope,
        event,
        original,
        anchor,
        co_anchors,
        publications,
    } = selection;
    let execution = &history.execution();
    charge(
        context,
        co_anchors.len().saturating_mul(event.co_anchors.len() + 1),
    )
    .map_err(failure)?;
    if co_anchors.len() != event.co_anchors.len() {
        return Err(reject(Cause::IncompleteRestoration, original));
    }
    for (index, (id, _)) in co_anchors.iter().enumerate() {
        charge(context, index + 1).map_err(failure)?;
        if !event.co_anchors.iter().any(|(old, _)| old == id)
            || co_anchors[..index].iter().any(|(old, _)| old == id)
        {
            return Err(reject(Cause::IncompleteRestoration, *id));
        }
    }
    for (original, anchor) in std::iter::once((original, anchor))
        .chain(co_anchors.iter().map(|(id, target)| (*id, target)))
    {
        (*basis.owner_inner())
            .root
            .demand_full_partition(execution, original.partition_id)
            .map_err(|cause| {
                failure(crate::history::data::codec_retirement_closure::request(
                    cause,
                ))
            })?;
        let partition = (*basis.owner_inner())
            .root
            .partition_state(original.partition_id)
            .map_err(|cause| failure(cause.into()))?
            .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, original))?;
        let row = partition
            .entity_arena
            .get(&original)
            .filter(|row| {
                row.retired_at() == Some(envelope.commit.version_id)
                    && row.kind_id() == Some(anchor.kind_id)
            })
            .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, original))?;
        let _ = row;
    }
    // Fund the fixed-width multi-publication proof comparisons before returning custody.
    charge(
        context,
        publications
            .len()
            .saturating_mul(publications.len().saturating_add(1)),
    )
    .map_err(failure)?;
    let mut entities = vec![(original, (*anchor).clone())];
    entities.extend_from_slice(co_anchors);
    for prior in publications {
        for member in &prior.origin.members {
            charge(context, 1).map_err(failure)?;
            entities.push((
                member.id,
                CreatedEntityRef {
                    partition_id: member.id.partition_id,
                    kind_id: member.kind,
                    client_key: ClientKey::raw(format!(
                        "authored-restore:{}:{}:{}",
                        member.id.partition_id.0,
                        member.id.local_slot_value(),
                        member.id.generation_value()
                    )),
                },
            ));
        }
    }
    let mut origin_frames = OutputOriginHistory::default();
    let validated_history = &mut origin_frames;
    let mut inverse = Vec::new();
    let mut content = crate::content::RelationalContentAttachments::default();
    let mut relations = Vec::new();
    let mut aggregate_memberships = Vec::new();
    for prior in publications {
        let predecessor = prior
            .origin
            .publication
            .predecessor()
            .ok_or_else(|| reject(Cause::CorruptPublication, original))?;
        let live = validated_history
            .by_commit(
                predecessor,
                &mut |commit| history.commit(commit),
                &history.execution(),
                context,
            )
            .map_err(failure)?
            .ok_or_else(|| reject(Cause::CorruptPublication, original))?;
        for member in &prior.origin.members {
            let plan = (*basis.owner_inner())
                .root
                .schema_authority()
                .entity_aspect_plan(member.kind)
                .ok_or_else(|| reject(Cause::CorruptPublication, member.id))?;
            let payload = closure::membership::reconstruct(
                history,
                &live,
                member.id,
                plan,
                validated_history,
                context,
                lease,
                (*basis.owner_inner()).root.schema_authority(),
            )
            .map_err(failure)?;
            (*live.owner_content())
                .retain_restoration_content(
                    execution,
                    payload.as_ref().and_then(|p| p.state.as_ref()),
                    &mut content,
                )
                .map_err(crate::content::publication::commit_denial)?;
            charge(context, entities.len()).map_err(failure)?;
            let target = entities
                .iter()
                .find(|(old, _)| *old == member.id)
                .expect("complete original roster")
                .1
                .clone();
            let aspect_patch = creation_patch(
                payload.as_ref().and_then(|p| p.state.as_ref()),
                plan,
                context,
            )
            .map_err(failure)?;
            inverse.push(MutationIntent::Create(CreateIntent::EntityAspects(
                EntityAspectCreateIntent {
                    partition_id: target.partition_id,
                    kind_id: target.kind_id,
                    client_key: target.client_key,
                    aspect_patch,
                },
            )));
        }
        let live_origin = live
            .output_origin_for_route(&prior.origin.meaning, context)
            .map_err(failure)?;
        for edge in &live_origin.relations {
            let plan = (*basis.owner_inner())
                .root
                .schema_authority()
                .relation_aspect_plan(edge.kind)
                .ok_or_else(|| reject(Cause::CorruptPublication, original))?;
            let payload = closure::relation_payload::reconstruct(
                history,
                &live,
                edge.id,
                plan,
                validated_history,
                context,
                lease,
                (*basis.owner_inner()).root.schema_authority(),
            )
            .map_err(failure)?;
            (*live.owner_content())
                .retain_restoration_content(
                    execution,
                    payload.as_ref().and_then(|p| p.state.as_ref()),
                    &mut content,
                )
                .map_err(crate::content::publication::commit_denial)?;
            if (*(*basis.owner_inner())
                .root
                .schema_authority()
                .registry()
                .owner_native_private_kinds())
            .contains(&edge.kind)
            {
                let (state, funding) = match payload {
                    Some(payload) => {
                        let (state, funding) = payload.into_retained_parts();
                        (state, Some(funding))
                    }
                    None => (None, None),
                };
                aggregate_memberships.push(crate::aggregate_identity::RestoredMembership::capture(
                    edge.id,
                    edge.kind,
                    edge.source,
                    state,
                    funding,
                ));
                continue;
            }
            charge(context, entities.len().saturating_mul(2)).map_err(failure)?;
            let resolve = |id| {
                entities
                    .iter()
                    .find(|(old, _)| *old == id)
                    .map_or(EntityReference::Existing(id), |(_, target)| {
                        EntityReference::Created(target.clone())
                    })
            };
            let target = CreatedRelationRef {
                partition_id: edge.id.partition_id,
                kind_id: edge.kind,
                client_key: ClientKey::raw(format!(
                    "authored-restore-edge:{}:{}:{}",
                    edge.id.partition_id.0,
                    edge.id.local_slot_value(),
                    edge.id.generation_value()
                )),
                source: resolve(edge.source),
                target: resolve(edge.target),
            };
            inverse.push(MutationIntent::Create(CreateIntent::RelationAspects(
                RelationAspectCreateIntent {
                    partition_id: target.partition_id,
                    kind_id: target.kind_id,
                    client_key: target.client_key.clone(),
                    source: target.source.clone(),
                    target: target.target.clone(),
                    aspect_patch: creation_patch(
                        payload.as_ref().and_then(|p| p.state.as_ref()),
                        plan,
                        context,
                    )
                    .map_err(failure)?,
                },
            )));
            relations.push((edge.id, target));
        }
    }
    Ok(InverseRestoration {
        entities,
        relations,
        inverse,
        content,
        aggregate_memberships,
    })
}
