//! Current custody is readmitted from Native's durable roster, without history.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::publication::data::{AuthoredRetirementRefusal as Cause, PublicationError};
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{EntityReference, RecordRef, TransactionCommitError};
use worth_execution::ExecutionRequest;

#[allow(clippy::too_many_arguments)]
pub(super) fn readmit(
    execution: &ExecutionRequest<'_, '_>,
    installed: &super::super::RelationalInstalledApplicationPublicationAuthority,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
    producer: &str,
    dependency: &[u8; 32],
    meaning: &super::super::RequiredOutputPublicationMeaning,
    roles: &[super::super::RequiredOutputRoleTarget],
) -> Result<RelationalRequiredOutputPublication, TransactionCommitError> {
    let refuse = |subject| {
        TransactionCommitError::publication(PublicationError::authored_retirement(
            Cause::CorruptPublication,
            RecordRef::Entity(subject),
        ))
    };
    let roster = crate::publication::ownership::select_current(
        execution,
        basis,
        &[RecordRef::Entity(meaning.source_scope)],
    )?;
    let (origin, ordinal) = roster
        .coordinates()
        .find(|(origin, _)| origin.meaning.same_route(meaning))
        .ok_or_else(|| refuse(meaning.source_scope))?;
    let result = execution
        .run_native_with_checkpoint(|context| {
            checkpoint(
                context,
                (producer.len()
                    + origin.producer.len()
                    + meaning.family.len()
                    + origin.meaning.family.len()
                    + roles.len()
                    + 96) as u64,
            )
            .map_err(execution_stop)?;
            if installed.runtime_instance_id != basis.descriptor().runtime_instance_id()
                || !basis.is_current()
                || roster.basis != *basis.descriptor()
                || origin.producer != producer
                || &origin.dependency_identity != dependency
                || &origin.meaning != meaning
                || roles.len() != origin.members.len()
            {
                return Err(refuse(meaning.source_scope));
            }
            let kind = installed
                .source_scope_ownership
                .scope_kind(producer)
                .ok_or_else(|| refuse(meaning.source_scope))?;
            if !basis
                .inner
                .root
                .get_current_partition(meaning.source_scope.partition_id)
                .and_then(|partition| partition.entity_arena().get(&meaning.source_scope))
                .is_some_and(|row| row.is_live() && row.kind_id() == Some(kind))
            {
                return Err(refuse(meaning.source_scope));
            }
            for (index, role) in roles.iter().enumerate() {
                checkpoint(
                    context,
                    (index + origin.members.len() + role.role.len() + role.entity_name.len() + 1)
                        as u64,
                )
                .map_err(execution_stop)?;
                if roles[..index]
                    .iter()
                    .any(|prior| prior.target == role.target)
                    || !origin.members.iter().any(|member| {
                        role.target == EntityReference::Existing(member.id)
                            && member.role == role.role
                            && member.entity_name == role.entity_name
                            && member.kind == role.kind
                            && member.posture == role.posture
                    })
                {
                    return Err(refuse(match role.target {
                        EntityReference::Existing(id) => id,
                        _ => meaning.source_scope,
                    }));
                }
            }
            Ok(())
        })
        .map_err(execution_stop)?
        .0;
    result?;
    let origin = Arc::clone(origin);
    Ok(RelationalRequiredOutputPublication {
        runtime_instance_id: installed.runtime_instance_id,
        authority: installed
            .authority
            .seal_required_output_publication_at(origin.commit.commit_id.0, ordinal),
        evidence: crate::publication::ownership::PublicationEvidence::Current(roster),
        origin,
        readmitted_at: Some(basis.clone()),
    })
}
