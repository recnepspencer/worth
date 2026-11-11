//! Current Native retirement, restoration and historical-selection succession.
use super::*;
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use crate::{
    branch::AdmittedRelationalBranchBasis, history::data::RequiredOutputRoutePosition,
    runtime::RelationalRuntime, transactions::data::EntityReference,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
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
            let current = if installed.source_scope_ownership.is_empty() {
                None
            } else {
                crate::history::data::current_route_presence::read_current(
                    execution, basis, meaning,
                )?
            };
            match current {
                Some(position) => Some(position),
                None => {
                    let historical = crate::history::data::readmit_required_output_route(
                        execution, self, basis, meaning,
                    )
                    .map_err(custody::cold_stop)?;
                    if installed.source_scope_ownership.is_empty() {
                        historical
                    } else {
                        crate::history::data::current_route_presence::require_current_owned_route(
                            execution, self, basis, meaning, historical,
                        )?
                    }
                }
            }
        } else {
            selected
        };
        let Some(RequiredOutputRoutePosition::Published(commit)) = selected else {
            return Ok(None);
        };
        let envelope = self
            .history_access()
            .demand_commit_envelope(execution, commit)
            .map_err(custody::execution_stop)?
            .ok_or_else(|| origin_denial("selected retirement successor history absent"))?;
        let origin = execution
            .in_scope(|lease| {
                worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                    .run_with_checkpoint(lease, None, |context| {
                        envelope
                            .validate_output_origins(context)
                            .and_then(|origins| origins.output_origin_for_route(meaning, context))
                            .map(Arc::clone)
                    })
            })
            .map_err(custody::execution_stop)?
            .map_err(custody::execution_stop)?
            .0
            .map_err(custody::cold_stop)?;
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
                origin.dependency_identity.bytes(),
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
}
