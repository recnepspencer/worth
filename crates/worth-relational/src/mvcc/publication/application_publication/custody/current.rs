//! Current custody is readmitted from Native's durable roster, without history.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::publication::data::{AuthoredRetirementRefusal as Cause, PublicationError};
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{EntityReference, RecordRef, TransactionCommitError};
use crate::{
    branch::AdmittedRelationalBranchBasis, history::data::CanonicalRequiredOutputOrigin,
    identity::data::KindId, publication::ownership::CurrentPublicationRoster,
};
use worth_execution::ExecutionRequest;
mod comparison;

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
            super::description::charge(context, origin, producer, meaning, roles.len())?;
            let kind = installed.source_scope_ownership.scope_kind(producer);
            if !kind.is_some_and(|kind| owner_matches(installed, basis, &roster, origin, kind))
                || origin.producer != producer
                || origin.dependency_identity.bytes() != dependency
                || &origin.meaning != meaning
                || roles.len() != origin.members.len()
            {
                return Err(refuse(meaning.source_scope));
            }
            super::description::require(
                context,
                origin,
                producer,
                dependency,
                meaning,
                roles.iter().map(|role| super::RecordedRequiredOutputRole {
                    role: &role.role,
                    entity_name: &role.entity_name,
                    kind: role.kind,
                    posture: role.posture,
                    entity: match role.target {
                        EntityReference::Existing(entity) => Some(entity),
                        _ => None,
                    },
                }),
            )?
            .map_err(refuse)?;
            Ok(())
        })
        .map_err(execution_stop)?
        .0;
    result?;
    let origin = Arc::clone(origin);
    Ok(issue(installed, basis, roster, origin, ordinal))
}

/// The current owned publication is producer-succession custody even when a
/// Query checkpoint retains an older association. This does not read history.
impl crate::runtime::RelationalRuntime {
    pub fn readmit_current_owned_output_successor(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &super::super::RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        running_producer: &str,
        meaning: &super::super::RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        readmit_successor(
            self.runtime_instance_id(),
            execution,
            installed,
            basis,
            running_producer,
            meaning,
        )
    }
}

/// The runtime and its scoped preparation port use this one current-custody owner.
pub(in crate::mvcc::publication::application_publication) fn readmit_successor(
    runtime_instance_id: u64,
    execution: &ExecutionRequest<'_, '_>,
    installed: &super::super::RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    running_producer: &str,
    meaning: &super::super::RequiredOutputPublicationMeaning,
) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
    let Some(kind) = installed
        .source_scope_ownership
        .scope_kind(running_producer)
    else {
        return Ok(None);
    };
    if installed.runtime_instance_id != runtime_instance_id
        || basis.descriptor().runtime_instance_id() != runtime_instance_id
        || !basis.is_current()
    {
        return Err(super::super::origin_denial(
            "foreign current publication owner",
        ));
    }
    installed.require_current_ownership_storage(basis, meaning.source_scope)?;
    let roster = crate::publication::ownership::select_current(
        execution,
        basis,
        &[RecordRef::Entity(meaning.source_scope)],
    )?;
    let selected = execution
        .run_native_with_checkpoint(|context| {
            for (origin, ordinal) in roster.coordinates() {
                checkpoint(
                    context,
                    (origin.meaning.family.len() + meaning.family.len() + 1) as u64,
                )
                .map_err(execution_stop)?;
                if !origin.meaning.same_route(meaning) {
                    continue;
                }
                checkpoint(
                    context,
                    (origin.producer.len() + running_producer.len() + 96) as u64,
                )
                .map_err(execution_stop)?;
                if !owner_matches(installed, basis, &roster, origin, kind) {
                    return Err(TransactionCommitError::publication(
                        PublicationError::authored_retirement(
                            Cause::CorruptPublication,
                            RecordRef::Entity(meaning.source_scope),
                        ),
                    ));
                }
                return Ok(Some((Arc::clone(origin), ordinal)));
            }
            Ok(None)
        })
        .map_err(execution_stop)?
        .0?;
    Ok(selected.map(|(origin, ordinal)| issue(installed, basis, roster, origin, ordinal)))
}

fn owner_matches(
    installed: &super::super::RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    roster: &CurrentPublicationRoster,
    origin: &CanonicalRequiredOutputOrigin,
    kind: KindId,
) -> bool {
    installed.runtime_instance_id == basis.descriptor().runtime_instance_id()
        && basis.is_current()
        && roster.basis.as_ref() == Some(basis.descriptor())
        && origin.source_scope_owned
        && installed
            .source_scope_ownership
            .scope_kind(&origin.producer)
            == Some(kind)
        && basis
            .inner
            .root
            .get_current_partition(origin.meaning.source_scope.partition_id)
            .and_then(|partition| partition.entity_arena().get(&origin.meaning.source_scope))
            .is_some_and(|row| row.is_live() && row.kind_id() == Some(kind))
}

fn issue(
    installed: &super::super::RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    roster: Arc<CurrentPublicationRoster>,
    origin: Arc<CanonicalRequiredOutputOrigin>,
    ordinal: usize,
) -> RelationalRequiredOutputPublication {
    RelationalRequiredOutputPublication {
        runtime_instance_id: installed.runtime_instance_id,
        authority: installed
            .authority
            .seal_required_output_publication_at(origin.commit.commit_id.0, ordinal),
        evidence: crate::publication::ownership::PublicationEvidence::Current(roster),
        origin,
        readmitted_at: Some(basis.clone()),
    }
}
