//! Reconstructive selection for an ordinary author's exact retired output roster.
use super::{charge, require_linear_history, CodecRetirementClosureStop as Stop};
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::{CanonicalCommitEnvelope, CanonicalRequiredOutputOrigin},
    mvcc::{RequiredOutputRolePosture as Posture, RequiredOutputRoleTarget},
    runtime::RelationalRuntime,
    transactions::data::EntityReference,
};
use std::sync::Arc;
use worth_execution::{
    ExecutionRequest, ExecutionResourceLease, ExecutionWorkCeiling, MapKernelContext,
};

/// Explicit reconstruction: selected ancestry and the supplied member roster,
/// never another route or branch. Payload/readmission follows in its owner.
pub(crate) fn select_authored_retirement(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    roles: &[RequiredOutputRoleTarget],
) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop> {
    let result = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| {
                if !has_owned_birth(execution, runtime, basis, roles, context)? {
                    return Ok(None);
                }
                select(runtime, basis, roles, context, lease)
            },
        )
    });
    execution.consult().map_err(super::request)?;
    result.map_err(super::request)?.map_err(super::request)?.0
}

fn select(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    roles: &[RequiredOutputRoleTarget],
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop> {
    // The private caller has verified current full-GEN Retire targets at birth.
    let mut next = basis.inner.root.commit_id();
    let bound = runtime.history.catalog_len();
    for _ in 0..bound {
        let Some(commit) = next else { return Ok(None) };
        charge(context, bound.saturating_add(roles.len()))?;
        let envelope = runtime
            .history
            .canonical_envelope(commit)
            .ok_or("authored retirement lost selected canonical history")?;
        require_linear_history(&envelope)?;
        if let Some(origin) = &envelope.required_output_origin {
            let comparisons =
                roles
                    .len()
                    .checked_mul(origin.members.len())
                    .ok_or(Stop::Request(
                        crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                    ))?;
            charge(context, comparisons)?;
            if roles.iter().any(|role| {
                origin
                    .members
                    .iter()
                    .any(|member| role.target == EntityReference::Existing(member.id))
            }) {
                envelope.validate_output_membership_header()?;
                require_complete_membership(origin, roles)?;
                return Ok(Some(envelope));
            }
        }
        if let Some(event) = &envelope.required_output_materialization {
            super::materialization::validate_semantics(runtime, &envelope, context, lease)?;
            next = Some(event.source_commit);
        } else {
            next = envelope.commit.parents.first().copied();
        }
    }
    if next.is_none() {
        Ok(None)
    } else {
        Err("authored retirement ancestry incomplete or cyclic".into())
    }
}

fn require_complete_membership(
    origin: &CanonicalRequiredOutputOrigin,
    roles: &[RequiredOutputRoleTarget],
) -> Result<(), Stop> {
    let live = origin
        .members
        .iter()
        .filter(|member| member.posture != Posture::Retire)
        .count();
    let missing = roles.iter().any(|role| {
        !origin.members.iter().any(|member| {
            member.posture != Posture::Retire
                && member.kind == role.kind
                && member.entity_name == role.entity_name
                && role.target == EntityReference::Existing(member.id)
        })
    });
    let repeated = roles.iter().enumerate().any(|(index, role)| {
        roles[..index]
            .iter()
            .any(|other| other.target == role.target)
    });
    if live != roles.len() || missing || repeated {
        return Err(
            "authored retirement omits, repeats or mixes selected publication members".into(),
        );
    }
    Ok(())
}

/// Native ownership is born with an actual canonical Create. Ordinary Existing
/// roles cannot acquire producer ownership later, so proven unowned births do
/// not require the unrelated branch's producer-continuity history.
fn has_owned_birth(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    roles: &[RequiredOutputRoleTarget],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Stop> {
    use crate::{
        publication::patch::data::RecordStructuralChange, storage::overlay::PartitionAccess,
        transactions::data::RecordRef,
    };
    if basis.descriptor().runtime_instance_id() != runtime.runtime_instance_id()
        || !basis.is_current()
    {
        return Err("authored retirement requires the exact current native basis".into());
    }
    if roles.is_empty() {
        return Err("authored retirement requires actual existing Retire roles".into());
    }
    let mut owned = false;
    for role in roles {
        let EntityReference::Existing(id) = role.target else {
            return Err("authored retirement requires an existing full generation".into());
        };
        if role.posture != Posture::Retire {
            return Err("authored retirement requires only Retire roles".into());
        }
        basis
            .inner
            .root
            .demand_full_partition(execution, id.partition_id)
            .map_err(super::request)?;
        let partition = basis
            .inner
            .root
            .partition_state(id.partition_id)?
            .ok_or("retirement before-image partition absent")?;
        let row = partition
            .entity_arena
            .get(&id)
            .filter(|row| row.is_live())
            .ok_or("retirement before-image full generation absent or retired")?;
        if row.kind_id() != Some(role.kind) {
            return Err("retirement before-image kind differs".into());
        }
        let created = partition
            .entity_arena
            .created_at_for_slot(id.slot_index())
            .ok_or("retirement before-image birth absent")?;
        let envelope = runtime
            .history
            .canonical_envelope_for_version(created)
            .ok_or("retirement canonical birth unavailable")?;
        charge(
            context,
            envelope
                .record_allocations
                .len()
                .saturating_add(envelope.patch.authoritative_record_patches.len()),
        )?;
        if envelope.commit.version_id != created
            || envelope
                .record_allocations
                .iter()
                .filter(|allocation| allocation.record() == &RecordRef::Entity(id))
                .count()
                != 1
            || !envelope
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| {
                    patch.target == RecordRef::Entity(id)
                        && patch.structural_change == RecordStructuralChange::Created
                })
        {
            return Err("retirement lacks exact canonical full-generation allocation".into());
        }
        if let Some(origin) = &envelope.required_output_origin {
            envelope.validate_output_membership_header()?;
            owned |= origin.outputs.binary_search(&id).is_ok();
        }
    }
    Ok(owned)
}
