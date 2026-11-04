//! Catalog-bounded closure from installed source ownership, never submitted reachability.
mod roster;
use super::{charge, CodecRetirementClosureStop as Stop};
use crate::storage::overlay::PartitionAccess;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::{CommitId, SelectedOutputPublication},
    identity::data::EntityId,
    mvcc::{
        RelationalInstalledApplicationPublicationAuthority, RequiredOutputRolePosture as Posture,
        RequiredOutputRoleTarget,
    },
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    runtime::RelationalRuntime,
    transactions::data::{EntityReference, RecordRef, TransactionCommitError},
};
use roster::require_roster;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, MapKernelContext};

pub(super) fn reject(cause: Cause, id: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        cause,
        RecordRef::Entity(id),
    ))
}
pub(super) fn stopped(stop: Stop) -> TransactionCommitError {
    match stop {
        Stop::Payload(detail) => crate::mvcc::origin_denial(detail),
        Stop::Request(cause) => {
            TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            })
        }
    }
}

pub(crate) struct SelectedAuthoredRetirement {
    pub(crate) birth: CommitId,
    pub(crate) co_anchors: Vec<(EntityId, CommitId)>,
    pub(crate) retirements: Vec<SelectedOutputPublication>,
    pub(crate) coupled_owners: Vec<SelectedOutputPublication>,
    pub(crate) current: Vec<SelectedOutputPublication>,
}

pub(crate) enum AuthoredRetirementSelection {
    SourceScopeClosure,
    ClosureAndLinkedOwners,
}

/// O(H² + P²M + R² + PMR), bounded by the actual catalog H, latest publications P,
/// their members M and submitted roles R. Every traversal/comparison is charged.
pub(crate) fn select_authored_source_retirement(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    anchor: EntityId,
    co_anchors: &[EntityId],
    roles: &[RequiredOutputRoleTarget],
    selection: AuthoredRetirementSelection,
) -> Result<SelectedAuthoredRetirement, TransactionCommitError> {
    let result = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| {
                if installed.runtime_instance_id != runtime.runtime_instance_id()
                    || basis.descriptor().runtime_instance_id() != runtime.runtime_instance_id()
                    || !basis.is_current()
                {
                    return Err(reject(Cause::InvalidAuthoredAnchor, anchor));
                }
                let birth = authored_birth(execution, runtime, basis, anchor, context)?;
                let records=std::iter::once(anchor).chain(co_anchors.iter().copied()).map(RecordRef::Entity).collect::<Vec<_>>();
                let latest = super::source_scope_closure::current(execution,basis,Some(installed),&records,context)?;
                let mut anchors = Vec::with_capacity(co_anchors.len() + 1);
                anchors.push(anchor);
                let mut births = Vec::with_capacity(co_anchors.len());
                for &id in co_anchors {
                    charge(context, anchors.len()).map_err(stopped)?;
                    if anchors.contains(&id) {
                        return Err(reject(Cause::RepeatedTarget, id));
                    }
                    births.push((id, authored_birth(execution, runtime, basis, id, context)?));
                    anchors.push(id);
                }
                for &id in &anchors {
                    for publication in &latest {
                        charge(context, publication.origin.members.len()).map_err(stopped)?;
                        if publication.origin.members.iter().any(|member| member.id == id && member.posture != Posture::Retire) {
                            return Err(reject(Cause::InvalidAuthoredAnchor, id));
                        }
                    }
                }
                let selected = super::source_scope_closure::select(&anchors, &latest, context)
                    .map_err(stopped)?;
                for publication in &selected {
                    let origin = &publication.origin;
                    let source = origin.meaning.source_scope;
                    charge(
                        context,
                        installed
                            .source_scope_ownership
                            .lookup_work(&origin.producer),
                    )
                    .map_err(stopped)?;
                    let kind = installed
                        .source_scope_ownership
                        .scope_kind(&origin.producer)
                        .ok_or_else(|| reject(Cause::CorruptPublication, source))?;
                    basis
                        .inner
                        .root
                        .demand_full_partition(execution, source.partition_id)
                        .map_err(|cause| stopped(super::request(cause)))?;
                    let partition = basis
                        .inner
                        .root
                        .partition_state(source.partition_id)
                        .map_err(|cause| stopped(cause.into()))?
                        .ok_or_else(|| reject(Cause::CorruptPublication, source))?;
                    if !partition
                        .entity_arena
                        .get(&source)
                        .is_some_and(|row| row.is_live() && row.kind_id() == Some(kind))
                    {
                        return Err(reject(Cause::CorruptPublication, source));
                    }
                }
                let coupled_owners = super::scope_dependents::linked_owners(
                    &anchors, &selected, &latest, context,
                ).map_err(stopped)?;
                if matches!(selection, AuthoredRetirementSelection::SourceScopeClosure)
                    && let Some(required) = coupled_owners.first()
                {
                    return Err(reject(Cause::MissingCoupledOwner, required.origin.meaning.source_scope));
                }
                if selected.is_empty() {
                    return Ok(SelectedAuthoredRetirement {
                        birth, co_anchors: births, retirements: selected, coupled_owners, current: latest,
                    });
                }
                require_roster(anchor, &anchors, roles, &selected, &latest, context)?;
                // Cover the exact fixed-width legality-seal comparisons performed by proof admission.
                charge(context, selected.len().saturating_mul(selected.len() + 1))
                    .map_err(stopped)?;
                charge(context, roles.len()).map_err(stopped)?;
                for role in roles
                    .iter()
                    .filter(|role| matches!(role.target, EntityReference::Existing(id) if anchors.contains(&id)))
                {
                    let EntityReference::Existing(id) = role.target else { unreachable!() };
                    let partition = basis
                        .inner
                        .root
                        .partition_state(id.partition_id)
                        .map_err(|cause| stopped(cause.into()))?
                        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, id))?;
                    if !partition
                        .entity_arena
                        .get(&id)
                        .is_some_and(|row| row.kind_id() == Some(role.kind))
                    {
                        return Err(reject(Cause::InvalidAuthoredAnchor, id));
                    }
                }
                Ok(SelectedAuthoredRetirement {
                    birth, co_anchors: births, retirements: selected, coupled_owners, current: latest,
                })
            },
        )
    });
    execution
        .consult()
        .map_err(|cause| stopped(super::request(cause)))?;
    result
        .map_err(|cause| stopped(super::request(cause)))?
        .map_err(|cause| stopped(super::request(cause)))?
        .0
}

fn authored_birth(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    anchor: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CommitId, TransactionCommitError> {
    if anchor.generation_value() == 0 {
        return Err(reject(Cause::InvalidAuthoredAnchor, anchor));
    }

    basis
        .inner
        .root
        .demand_full_partition(execution, anchor.partition_id)
        .map_err(|cause| stopped(super::request(cause)))?;
    let partition = basis
        .inner
        .root
        .partition_state(anchor.partition_id)
        .map_err(|cause| stopped(Stop::from(cause)))?
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    partition
        .entity_arena
        .get(&anchor)
        .filter(|row| row.is_live())
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    let created = partition
        .entity_arena
        .created_at_for_slot(anchor.slot_index())
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    let envelope = runtime
        .history
        .canonical_envelope_for_version(created)
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    if envelope.authored_source_restoration().is_some() {
        super::source_scope_closure::validate_historical_prefix(
            execution, runtime, basis, &envelope, anchor, context,
        )?;
    }
    require_authored_birth(&envelope, anchor, context).map_err(|failure| match failure {
        Stop::Request(_) => stopped(failure),
        Stop::Payload(_) => reject(Cause::InvalidAuthoredAnchor, anchor),
    })?;
    Ok(envelope.commit.commit_id)
}

pub(super) fn require_authored_birth(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    anchor: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    use crate::publication::patch::data::RecordStructuralChange as Change;
    charge(
        context,
        envelope.record_allocations.len() + envelope.patch.authoritative_record_patches.len() + 1,
    )?;
    let mut owned = false;
    for origin in envelope.output_origins() {
        charge(context, origin.outputs.len())?;
        owned |= origin.outputs.contains(&anchor);
    }
    if owned
        || envelope
            .record_allocations
            .iter()
            .filter(|allocation| allocation.record() == &RecordRef::Entity(anchor))
            .count()
            != 1
        || !envelope
            .patch
            .authoritative_record_patches
            .iter()
            .any(|patch| {
                patch.target == RecordRef::Entity(anchor)
                    && patch.structural_change == Change::Created
            })
    {
        return Err("authored birth is absent or belongs to a producer publication".into());
    }
    Ok(())
}
