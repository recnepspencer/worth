//! Ordinary mutation retirement and current canonical retirement-successor custody.
use super::*;
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::{CanonicalCommitEnvelope, RequiredOutputRoutePosition},
    runtime::RelationalRuntime,
    transactions::data::EntityReference,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Reconstruct the selected complete publication behind real ordinary Retire
    /// targets. No matching canonical ownership leaves genuine ordinary deletion
    /// ordinary; any matched ownership requires its complete exact roster.
    pub fn readmit_authored_output_retirement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        if installed.runtime_instance_id != self.runtime_instance_id() {
            return Err(origin_denial(
                "foreign installed author for output retirement",
            ));
        }
        let Some(envelope) =
            crate::history::data::select_authored_retirement(execution, self, basis, roles)
                .map_err(custody::cold_stop)?
        else {
            return Ok(None);
        };
        self.readmit_canonical_publication(execution, installed, basis, &envelope)
            .map(Some)
    }

    /// Reissue succeeds this selected authored retirement, never a fresh Birth.
    /// The current route and complete retired membership come from Native's
    /// canonical history, not an old producer head or caller description.
    pub fn readmit_authored_output_retirement_successor(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        if installed.runtime_instance_id != self.runtime_instance_id() {
            return Err(origin_denial(
                "foreign installed owner for retirement successor",
            ));
        }
        if basis.descriptor().runtime_instance_id() != self.runtime_instance_id()
            || !basis.is_current()
        {
            return Err(origin_denial(
                "retirement successor requires the exact current native basis",
            ));
        }
        let selected = execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    basis
                        .inner
                        .root
                        .output_publication_positions
                        .get(meaning, context)
                })
        });
        consult_stage(execution, CommitStage::Admission)?;
        let selected = selected
            .map_err(custody::execution_stop)?
            .map_err(custody::execution_stop)?
            .0?;
        let selected = if selected.is_none()
            && basis
                .inner
                .root
                .output_publication_positions
                .unresolved_prefix()
                .is_some()
        {
            crate::history::data::readmit_required_output_route(execution, self, basis, meaning)
                .map_err(custody::cold_stop)?
        } else {
            selected
        };
        let Some(RequiredOutputRoutePosition::Published(commit)) = selected else {
            return Ok(None);
        };
        let envelope = self
            .history
            .canonical_envelope(commit)
            .ok_or_else(|| origin_denial("selected retirement successor history absent"))?;
        let origin = envelope
            .validate_output_membership_header()
            .map_err(custody::cold_stop)?;
        if !matches!(
            origin.publication,
            crate::history::data::CanonicalOutputPublication::AuthoredRetirement { .. }
        ) {
            return Ok(None);
        }
        self.readmit_canonical_publication(execution, installed, basis, &envelope)
            .map(Some)
    }

    fn readmit_canonical_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        envelope: &Arc<CanonicalCommitEnvelope>,
    ) -> Result<RelationalRequiredOutputPublication, TransactionCommitError> {
        let origin = envelope
            .validate_output_membership_header()
            .map_err(custody::cold_stop)?;
        // Full current payload admission below must enter the ordinary request's
        // storage regions before the historical association court accesses them.
        use crate::storage::overlay::{CurrentPartitionAccess, PartitionAccess};
        for member in &origin.members {
            basis
                .inner
                .root
                .demand_full_partition(execution, member.id.partition_id)
                .map_err(|stop| {
                    custody::execution_stop(crate::execution::RelationalExecutionDenialCause::from(
                        stop,
                    ))
                })?;
        }
        for relation in &origin.relations {
            basis
                .inner
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
                basis
                    .inner
                    .root
                    .demand_head_partition(execution, endpoint.partition_id)
                    .map_err(|stop| {
                        custody::execution_stop(
                            crate::execution::RelationalExecutionDenialCause::from(stop),
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
        self.readmit_required_output_publication(
            execution,
            installed,
            basis,
            &origin.producer,
            &origin.dependency_identity,
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
}
