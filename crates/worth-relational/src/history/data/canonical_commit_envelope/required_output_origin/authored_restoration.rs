//! Canonical reissue of every recorded route in one Native inverse candidate.
use super::*;
use crate::{
    history::data::RequiredOutputPublicationPositions,
    mvcc::*,
    storage::overlay::{
        current_entity_partition_source_for_slot, CurrentPartitionAccess, OverlayStateView,
        WorkingState,
    },
    transactions::data::*,
};
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalAuthoredRestoration {
    pub(crate) retirement: CommitId,
    pub(crate) original_anchor: EntityId,
    pub(crate) anchor: EntityId,
    pub(crate) entities: Vec<(EntityId, EntityId)>,
    pub(crate) relations: Vec<(
        crate::identity::data::RelationId,
        crate::identity::data::RelationId,
    )>,
    pub(crate) publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    envelope: &mut CanonicalCommitEnvelope,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    state: &WorkingState,
    installed: RelationalInstalledApplicationPublicationAuthority,
    admission: worth_proof::AdmittedAuthoredOutputRetirement,
    restoration: RelationalAuthoredRestoration,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<RequiredOutputPublicationPositions, TransactionCommitError> {
    let old = restoration
        .retirement
        .authored_cascade_retirement
        .as_ref()
        .ok_or_else(|| crate::mvcc::origin_denial("restoration lost its exact retirement"))?;
    let refuse = || {
        TransactionCommitError::publication(
            crate::publication::data::PublicationError::authored_retirement(
                crate::publication::data::AuthoredRetirementRefusal::IncompleteRestoration,
                RecordRef::Entity(old.anchor),
            ),
        )
    };
    if restoration.before.descriptor() != basis.descriptor()
        || admission.publication_count() != restoration.publications.len()
    {
        return Err(refuse());
    }
    super::restoration_candidate::validate(
        basis,
        state,
        created,
        created_relations,
        &restoration,
        context,
    )?;
    let anchor = created.resolve(&restoration.anchor).ok_or_else(refuse)?;
    let mut entities = Vec::with_capacity(restoration.entities.len());
    for (prior, target) in &restoration.entities {
        context
            .checkpoint(created.resolve_work_bound(target).ok_or_else(refuse)?)
            .map_err(origin_request)?;
        entities.push((*prior, created.resolve(target).ok_or_else(refuse)?));
    }
    let mut relations = Vec::with_capacity(restoration.relations.len());
    for (prior, target) in &restoration.relations {
        context
            .checkpoint(
                created_relations
                    .resolve_work_bound(target)
                    .ok_or_else(refuse)?,
            )
            .map_err(origin_request)?;
        relations.push((
            *prior,
            created_relations.resolve(target).ok_or_else(refuse)?,
        ));
    }
    let after = OverlayStateView::new(basis.inner.root.as_ref(), state);
    let source = basis
        .inner
        .root
        .get_current_partition(old.anchor.partition_id)
        .and_then(|part| part.entity_arena().get(&old.anchor))
        .ok_or_else(refuse)?;
    let (_, new_partition) =
        current_entity_partition_source_for_slot(&after, anchor.partition_id, anchor.slot_index())
            .ok_or_else(refuse)?;
    let new_source = new_partition
        .entity_arena()
        .get(&anchor)
        .filter(|row| row.is_live())
        .ok_or_else(refuse)?;
    context
        .checkpoint(
            source
                .extra()
                .authoritative_aspect_state
                .as_ref()
                .map_or(1, |state| state.owned_allocation_capacity_bytes()) as u64,
        )
        .map_err(origin_request)?;
    if source.extra().authoritative_aspect_state != new_source.extra().authoritative_aspect_state {
        return Err(refuse());
    }
    let mut publications = Vec::with_capacity(restoration.publications.len());
    let mut positions = basis.inner.root.output_publication_positions.clone();
    for prior in &restoration.publications {
        if !installed
            .authority
            .admits_authored_output_retirement(&admission, prior.authority())
        {
            return Err(refuse());
        }
        let origin = &prior.origin;
        let predecessor = origin.publication.predecessor().ok_or_else(refuse)?;
        let original = runtime
            .history
            .canonical_envelope(predecessor)
            .ok_or_else(refuse)?;
        let live = original
            .output_origin_for_route(&origin.meaning, context)
            .map_err(super::installation::relation_stop)?;
        let mut roles = Vec::with_capacity(origin.members.len());
        for member in live
            .members
            .iter()
            .filter(|member| member.posture != RequiredOutputRolePosture::Retire)
        {
            context
                .checkpoint(restoration.entities.len() as u64)
                .map_err(origin_request)?;
            let target = &restoration
                .entities
                .iter()
                .find(|(old, _)| *old == member.id)
                .ok_or_else(refuse)?
                .1;
            roles.push(RequiredOutputRoleTarget {
                role: member.role.clone(),
                entity_name: member.entity_name.clone(),
                kind: member.kind,
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Created(target.clone()),
            });
        }
        let members = super::members::capture(
            envelope,
            &runtime.history,
            basis,
            state,
            created,
            &roles,
            None,
            &origin.producer,
            origin.dependency_identity,
            context,
        )?;
        let mut edges = Vec::with_capacity(live.relations.len());
        for edge in &live.relations {
            context
                .checkpoint((relations.len() + entities.len() * 2) as u64)
                .map_err(origin_request)?;
            let id = relations
                .iter()
                .find(|(old, _)| *old == edge.id)
                .ok_or_else(refuse)?
                .1;
            let mapped = |id| {
                entities
                    .iter()
                    .find(|(old, _)| *old == id)
                    .map_or(id, |(_, id)| *id)
            };
            edges.push(CanonicalRequiredOutputRelation {
                id,
                kind: edge.kind,
                source: mapped(edge.source),
                target: mapped(edge.target),
                birth: envelope.commit.commit_id,
                before_write: None,
                after_write: envelope.commit.version_id,
            });
        }
        let mut meaning = origin.meaning.clone();
        context
            .checkpoint(entities.len() as u64)
            .map_err(origin_request)?;
        meaning.source_scope = entities
            .iter()
            .find(|(old, _)| *old == meaning.source_scope)
            .ok_or_else(refuse)?
            .1;
        let bytes = origin_retention_bytes(
            envelope,
            basis,
            origin.producer.len(),
            members.len(),
            edges.len(),
        )?;
        let funding =
            worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
                .map_err(|cause| origin_request(crate::execution::denial::lease_denial(cause)))?;
        let restored = Arc::new(CanonicalRequiredOutputOrigin {
            commit: envelope.commit.clone(),
            basis: basis
                .descriptor()
                .in_canonical_frame()
                .map_err(|_| refuse())?,
            producer: origin.producer.clone(),
            source_scope_owned: true,
            dependency_identity: origin.dependency_identity,
            outputs: members.iter().map(|member| member.id).collect(),
            relations: edges,
            meaning,
            members,
            publication: CanonicalOutputPublication::AuthoredRestoration {
                retirement: restoration.retirement.commit.commit_id,
                predecessor,
            },
            birth: origin.birth,
            birth_producer: origin.birth_producer.clone(),
            birth_dependency: origin.birth_dependency,
            issuer: Some(installed.authority.clone()),
            _funding: Some(funding),
        });
        positions.insert(
            &origin.meaning,
            crate::history::data::RequiredOutputRoutePosition::Published(envelope.commit.commit_id),
            lease,
            context,
        )?;
        positions.insert(
            &restored.meaning,
            crate::history::data::RequiredOutputRoutePosition::Published(envelope.commit.commit_id),
            lease,
            context,
        )?;
        publications.push(restored);
    }
    envelope.authored_cascade_restoration = Some(Arc::new(CanonicalAuthoredRestoration {
        retirement: restoration.retirement.commit.commit_id,
        original_anchor: old.anchor,
        anchor,
        entities,
        relations,
        publications,
    }));
    Ok(positions)
}
