//! Assemble inverse intents from exact retained publication origins.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::authored_restoration::{creation_patch, failure};
use worth_relational_branch::internal_port::branch::basis::{AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::origin_history::OutputOriginHistory;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::authored_cascade::CanonicalAuthoredRetirement;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyFieldsPort as _};
use contracts::identity::data::record_ids::EntityId;
use contracts::identity::data::record_ids::RelationId;
use crate::mvcc::publication::application_publication::RelationalRequiredOutputPublication;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use worth_relational_storage::internal_port::storage::overlay::access::PartitionAccess;
use contracts::symbols::data::client_key::ClientKey;
use contracts::transactions::data::intents::mutation_intent::CreateIntent;
use contracts::transactions::data::intents::mutation_intent::EntityAspectCreateIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::intents::mutation_intent::RelationAspectCreateIntent;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::CreatedEntityRef;
use contracts::transactions::data::primitives::CreatedRelationRef;
use contracts::transactions::data::primitives::EntityReference;
use worth_execution::{ExecutionResourceLease, MapKernelContext};
use worth_relational_branch::internal_port::history::data::codec_retirement_closure as closure;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::authored_cascade::reject;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::charge;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest;
use contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;

pub struct RestorationSelection<'a> {
    pub basis: &'a AdmittedRelationalBranchBasis,
    pub envelope: &'a CanonicalCommitEnvelope,
    pub event: &'a CanonicalAuthoredRetirement,
    pub original: EntityId,
    pub anchor: &'a CreatedEntityRef,
    pub co_anchors: &'a [(EntityId, CreatedEntityRef)],
    pub publications: &'a [RelationalRequiredOutputPublication],
}

pub struct InverseRestoration {
    pub entities: Vec<(EntityId, CreatedEntityRef)>,
    pub relations: Vec<(RelationId, CreatedRelationRef)>,
    pub inverse: Vec<MutationIntent>,
    pub content: worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
    pub aggregate_memberships: Vec<worth_relational_mutation::internal_port::aggregate_identity::restoration::RestoredMembership>,
}

pub fn prepare(
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
        basis
            .owner_inner()
            .root
            .demand_full_partition(execution, original.partition_id)
            .map_err(|cause| {
                failure(worth_relational_branch::internal_port::history::data::codec_retirement_closure::request(
                    cause,
                ))
            })?;
        let partition = basis
            .owner_inner()
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
    let mut content = worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments::default();
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
            let plan = basis
                .owner_inner()
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
                basis.owner_inner().root.schema_authority(),
            )
            .map_err(failure)?;
            (*live.owner_content())
                .retain_restoration_content(
                    execution,
                    payload.as_ref().and_then(|p| p.state.as_ref()),
                    &mut content,
                )
                .map_err(
                    worth_relational_branch::internal_port::content::publication::commit_denial,
                )?;
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
            let plan = basis
                .owner_inner()
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
                basis.owner_inner().root.schema_authority(),
            )
            .map_err(failure)?;
            (*live.owner_content())
                .retain_restoration_content(
                    execution,
                    payload.as_ref().and_then(|p| p.state.as_ref()),
                    &mut content,
                )
                .map_err(
                    worth_relational_branch::internal_port::content::publication::commit_denial,
                )?;
            if (*basis
                .owner_inner()
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
                aggregate_memberships.push(worth_relational_mutation::internal_port::aggregate_identity::restoration::RestoredMembership::capture(
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
