//! Authorize the exact live source closure and its submitted retirement roster.
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisOwnerPort as _,
};
use worth_relational_contracts::internal_port as contracts;
pub mod foreign_member;
pub mod roster;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::authored_cascade::{authored_birth, reject, stopped};
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::{CodecRetirementClosureStop as Stop, charge, request};
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::scope_dependents::linked_owners;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::source_scope_closure::select;
use contracts::history::data::coordinates::CommitId;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::output_correspondence::SelectedOutputPublication;
use contracts::identity::data::record_ids::EntityId;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture as Posture;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use worth_relational_storage::internal_port::storage::overlay::access::PartitionAccess;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::EntityReference;
use contracts::transactions::data::primitives::RecordRef;
use roster::require_roster;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, MapKernelContext};

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
pub(crate) fn select_authored_source_retirement<'r>(
    execution: &ExecutionRequest<'_, '_>,
    runtime: impl Into<
        worth_relational_branch::internal_port::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs<
            'r,
        >,
    >,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    anchor: EntityId,
    co_anchors: &[EntityId],
    roles: &[RequiredOutputRoleTarget],
    selection: AuthoredRetirementSelection,
) -> Result<SelectedAuthoredRetirement, TransactionCommitError> {
    let runtime = &worth_relational_branch::internal_port::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest::new(runtime, execution);
    let result = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| {
                if installed.runtime_instance_id != runtime.input.projection.runtime_identity
                    || basis.descriptor().runtime_instance_id()
                        != runtime.input.projection.runtime_identity
                    || !basis.is_current()
                {
                    return Err(reject(Cause::InvalidAuthoredAnchor, anchor));
                }
                let birth = authored_birth(execution, runtime, basis, anchor, context)?;
                let records=std::iter::once(anchor).chain(co_anchors.iter().copied()).map(RecordRef::Entity).collect::<Vec<_>>();
                let latest = super::source_scope::current(execution,basis,Some(&installed.source_scope_ownership),&records,context)?;
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
                let selected = select(&anchors, &latest, context)
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

                        .demand_full_partition(execution, source.partition_id)
                        .map_err(|cause| stopped(request(cause)))?;
                    let partition = basis

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
                let coupled_owners = linked_owners(
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
                require_roster(execution, runtime, basis, anchor, &anchors, roles, &selected, &latest, context)?;
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
        .map_err(|cause| stopped(request(cause)))?;
    result
        .map_err(|cause| stopped(request(cause)))?
        .map_err(|cause| stopped(request(cause)))?
        .0
}
