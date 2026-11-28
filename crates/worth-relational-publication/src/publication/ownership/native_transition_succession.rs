//! Current canonical route succession, including independent ordinary producers.
use worth_relational_contracts::internal_port as contracts;
use crate::mvcc::publication::application_publication::{custody, *};
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use worth_relational_branch::internal_port::branch::basis::{AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _};
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs;
use worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputRoutePosition;
use contracts::transactions::data::primitives::EntityReference;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
pub mod independent;
pub mod selection;

pub fn readmit_carried_required_output_publication(
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

pub fn readmit_native_output_transition_successor(
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
        worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalOutputPublication::AuthoredRetirement { .. }
            | worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalOutputPublication::AuthoredRestoration { .. }
            | worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalOutputPublication::HistoricalSelection { .. }
    ) {
        return Ok(None);
    }
    readmit_canonical_publication(inputs, execution, installed, basis, &origin).map(Some)
}

pub fn readmit_canonical_publication(
    inputs: RetirementHistoryInputs<'_>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    origin: &Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin>,
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
        return custody::readmit_required_output_publication(
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
    use worth_relational_storage::internal_port::storage::overlay::{
        CurrentPartitionAccess, PartitionAccess,
    };
    for member in &origin.members {
        basis
            .demand_full_partition(execution, member.id.partition_id)
            .map_err(|stop| {
                custody::execution_stop(
                    contracts::execution::denial_cause::RelationalExecutionDenialCause::from(stop),
                )
            })?;
    }
    for relation in &origin.relations {
        basis
            .demand_full_partition(execution, relation.id.partition_id)
            .map_err(|stop| {
                custody::execution_stop(
                    contracts::execution::denial_cause::RelationalExecutionDenialCause::from(stop),
                )
            })?;
    }
    for relation in &origin.relations {
        for endpoint in [relation.source, relation.target] {
            basis
                .demand_head_partition(execution, endpoint.partition_id)
                .map_err(|stop| {
                    custody::execution_stop(
                        contracts::execution::denial_cause::RelationalExecutionDenialCause::from(
                            stop,
                        ),
                    )
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
    custody::readmit_required_output_publication(
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
