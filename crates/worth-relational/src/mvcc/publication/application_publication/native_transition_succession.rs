//! Current canonical route succession, including independent ordinary producers.
use super::*;
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use crate::{
    branch::AdmittedRelationalBranchBasis, history::data::RequiredOutputRoutePosition,
    runtime::RelationalRuntime, transactions::data::EntityReference,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
mod independent;
mod selection;
impl RelationalRuntime {
    /// Reissue succeeds the actual Native route transition, never a fresh Birth.
    /// Active owned membership is authenticated by the current-custody owner.
    pub fn readmit_native_output_transition_successor(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        let Some((_, origin)) = selection::select(self, execution, installed, basis, meaning)?
        else {
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
        self.readmit_canonical_publication(execution, installed, basis, &origin)
            .map(Some)
    }

    pub(crate) fn readmit_canonical_publication(
        &self,
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
            return self.readmit_required_output_publication(
                execution,
                installed,
                basis,
                &origin.producer,
                &origin.dependency_identity,
                &origin.meaning,
                &roles,
            );
        }
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
