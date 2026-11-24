//! Current canonical route succession, including independent ordinary producers.
use super::*;
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs;
use crate::history::data::RequiredOutputRoutePosition;
use crate::transactions::data::primitives::EntityReference;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
pub(crate) mod independent;
mod selection;

pub(crate) fn readmit_carried_required_output_publication(
    inputs: RetirementHistoryInputs<'_>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    carried: &RelationalRequiredOutputPublication,
) -> Result<RelationalRequiredOutputPublication, TransactionCommitError> {
    if carried.runtime_instance_id != inputs.projection.runtime_identity
        || installed.runtime_instance_id != inputs.projection.runtime_identity
        || basis.descriptor().runtime_instance_id() != inputs.projection.runtime_identity
    {
        return Err(origin_denial("foreign carried publication custody"));
    }
    if carried
        .readmitted_at
        .as_ref()
        .is_none_or(|at| at.descriptor() == basis.descriptor())
    {
        // Canonical carried custody still goes through the ordinary exact
        // predecessor validation. A cold/current proof additionally binds C.
        return Ok(carried.clone());
    }
    readmit_canonical_publication(inputs, execution, installed, basis, &carried.origin)
}

pub(crate) fn readmit_native_output_transition_successor(
    inputs: RetirementHistoryInputs<'_>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
    let Some((_, origin)) = selection::select(inputs, execution, installed, basis, meaning)? else {
        return Ok(None);
    };
    if !matches!(
        origin.publication,
        crate::history::data::CanonicalOutputPublication::AuthoredRetirement { .. }
            | crate::history::data::CanonicalOutputPublication::AuthoredRestoration { .. }
            | crate::history::data::CanonicalOutputPublication::HistoricalSelection { .. }
    ) {
        return Ok(None);
    }
    readmit_canonical_publication(inputs, execution, installed, basis, &origin).map(Some)
}

pub(crate) fn readmit_canonical_publication(
    inputs: RetirementHistoryInputs<'_>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    origin: &Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
) -> Result<RelationalRequiredOutputPublication, TransactionCommitError> {
    if origin.source_scope_owned {
        installed.require_current_ownership_storage(basis, origin.meaning.source_scope)?;
    }
    if origin.source_scope_owned
        && origin
            .members
            .iter()
            .any(|member| member.posture != RequiredOutputRolePosture::Retire)
    {
        let roles = origin
            .members
            .iter()
            .map(|member| RequiredOutputRoleTarget {
                role: member.role.clone(),
                entity_name: member.entity_name.clone(),
                kind: member.kind,
                posture: member.posture,
                target: EntityReference::Existing(member.id),
            })
            .collect::<Vec<_>>();
        return super::custody::readmit_required_output_publication(
            inputs,
            execution,
            installed,
            basis,
            &origin.producer,
            origin.dependency_identity.bytes(),
            &origin.meaning,
            &roles,
        );
    }
    // Full current payload admission below must enter the ordinary request's
    // storage regions before the historical association court accesses them.
    use crate::storage::overlay::{CurrentPartitionAccess, PartitionAccess};
    for member in &origin.members {
        (*basis.owner_inner())
            .root
            .demand_full_partition(execution, member.id.partition_id)
            .map_err(|stop| {
                custody::execution_stop(crate::execution::RelationalExecutionDenialCause::from(
                    stop,
                ))
            })?;
    }
    for relation in &origin.relations {
        (*basis.owner_inner())
            .root
            .demand_full_partition(execution, relation.id.partition_id)
            .map_err(|stop| {
                custody::execution_stop(crate::execution::RelationalExecutionDenialCause::from(
                    stop,
                ))
            })?;
    }
    for relation in &origin.relations {
        for endpoint in [relation.source, relation.target] {
            (*basis.owner_inner())
                .root
                .demand_head_partition(execution, endpoint.partition_id)
                .map_err(|stop| {
                    custody::execution_stop(crate::execution::RelationalExecutionDenialCause::from(
                        stop,
                    ))
                })?;
        }
    }
    consult_stage(execution, CommitStage::Admission)?;
    let roles = origin
        .members
        .iter()
        .map(|member| RequiredOutputRoleTarget {
            role: member.role.clone(),
            entity_name: member.entity_name.clone(),
            kind: member.kind,
            posture: member.posture,
            target: EntityReference::Existing(member.id),
        })
        .collect::<Vec<_>>();
    super::custody::readmit_required_output_publication(
        inputs,
        execution,
        installed,
        basis,
        &origin.producer,
        origin.dependency_identity.bytes(),
        &origin.meaning,
        &roles,
    )
    .map_err(|failure| match failure {
        TransactionCommitError::Publication {
            mut error,
            commit_log,
        } => {
            error.detail = format!(
                "{}; publication_stage=predecessor-readmission; before={:?}",
                error.detail,
                basis.descriptor()
            );
            TransactionCommitError::Publication { error, commit_log }
        }
        failure => failure,
    })
}
