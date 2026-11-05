//! Ordinary mutation retirement and current canonical retirement-successor custody.
use super::*;
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use crate::{
    branch::AdmittedRelationalBranchBasis, history::data::RequiredOutputRoutePosition,
    runtime::RelationalRuntime, transactions::data::EntityReference,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
mod current_retirement;

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
        let current = self.current_retirement_custody(execution, installed, basis, roles)?;
        self.refuse_live_owned_retirement(execution, roles, current.as_deref())?;
        let selected =
            crate::history::data::select_authored_retirement(execution, self, basis, roles, false)
                .map_err(custody::cold_stop)?;
        if let Some(envelope) = &selected
            && envelope.origin.source_scope_owned
        {
            let origin = &envelope.origin;
            return Err(TransactionCommitError::publication(
                crate::publication::data::PublicationError::owned_publication(
                    crate::publication::data::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                        publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
                    }, origin.outputs[0], origin.meaning.source_scope,
                )));
        }
        self.refuse_retired_scope_owners(execution, roles, current.as_deref())?;
        let Some(envelope) = selected else {
            return Ok(None);
        };
        self.readmit_canonical_publication(execution, installed, basis, &envelope.origin)
            .map(Some)
    }

    /// Mixed ordinary candidates retain legacy behavior for independent outputs,
    /// but cannot hide an owned member or its live source beside Create/Preserve roles.
    pub fn refuse_owned_output_retirement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<(), TransactionCommitError> {
        if installed.runtime_instance_id != self.runtime_instance_id() {
            return Err(origin_denial(
                "foreign installed author for owned retirement",
            ));
        }
        if installed.source_scope_ownership.is_empty() {
            return Ok(());
        }
        let current = self.current_retirement_custody(execution, installed, basis, roles)?;
        self.refuse_live_owned_retirement(execution, roles, current.as_deref())?;
        if let Some(selected) =
            crate::history::data::select_authored_retirement(execution, self, basis, roles, true)
                .map_err(custody::cold_stop)?
        {
            let origin = selected.origin;
            return Err(TransactionCommitError::publication(
                crate::publication::data::PublicationError::owned_publication(
                    crate::publication::data::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                        publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
                    }, origin.outputs[0], origin.meaning.source_scope,
                )));
        }
        self.refuse_retired_scope_owners(execution, roles, current.as_deref())
    }

    fn refuse_retired_scope_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        roles: &[RequiredOutputRoleTarget],
        current: Option<&crate::publication::ownership::CurrentPublicationRoster>,
    ) -> Result<(), TransactionCommitError> {
        let Some(current) = current else {
            return Ok(());
        };
        let selected = execution
            .run_native_with_checkpoint(|context| {
                context.checkpoint(roles.len() as u64).map_err(|stop| {
                    crate::history::data::CodecRetirementClosureStop::Request(stop.into())
                })?;
                let removed = roles
                    .iter()
                    .map(|role| match role.target {
                        EntityReference::Existing(id) => Ok(id),
                        _ => Err(crate::history::data::CodecRetirementClosureStop::from(
                            "retirement source must be an existing native generation",
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                crate::history::data::select_owned_scope_dependent(
                    crate::history::data::OwnedScopeDependentSelection::RemovedSources(&removed),
                    current,
                    context,
                )
                .map_err(|failure| failure.stop)
            })
            .map_err(custody::execution_stop)?
            .0
            .map_err(custody::cold_stop)?;
        consult_stage(execution, CommitStage::Admission)?;
        if let Some(origin) = selected {
            return Err(TransactionCommitError::publication(crate::publication::data::PublicationError::owned_publication(
                crate::publication::data::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                    publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
                }, origin.outputs[0], origin.meaning.source_scope)));
        }
        Ok(())
    }

    /// Reissue succeeds the selected authored retirement or its Native restoration, never a fresh Birth.
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
            .history
            .canonical_envelope(commit)
            .ok_or_else(|| origin_denial("selected retirement successor history absent"))?;
        let origin = execution
            .in_scope(|lease| {
                worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                    .run_with_checkpoint(lease, None, |context| {
                        envelope
                            .output_origin_for_route(meaning, context)
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
