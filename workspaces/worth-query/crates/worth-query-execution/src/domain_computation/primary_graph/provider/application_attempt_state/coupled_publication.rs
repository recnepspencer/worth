//! Native publication admission uses only the genuine registered writer's effects.
use super::*;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind as Cause, WorthQueryInvariantExecutionFailure as Failure,
};
use worth_relational::facade::{
    mvcc::{
        RelationalCoupledProducerPosture, RelationalRequiredOutputPublication,
        RequiredOutputPublication, RequiredOutputPublicationMeaning,
    },
    transactions::WorkerIntentBatch,
};

pub(in crate::domain_computation::primary_graph) struct CoupledWriterCommands {
    pub(in crate::domain_computation::primary_graph) publication: RelationalCoupledProducerPosture,
    pub(in crate::domain_computation::primary_graph) batch: WorkerIntentBatch,
    pub(in crate::domain_computation::primary_graph) content:
        worth_relational::facade::content::RelationalContentAttachments,
}
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn authored_stage_commands(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Result<
        (
            WorkerIntentBatch,
            worth_relational::facade::content::RelationalContentAttachments,
            Vec<worth_relational::facade::mvcc::RequiredOutputRoleTarget>,
        ),
        Failure,
    > {
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let staged = attempts.staged_attempt(session).ok_or_else(|| {
            Failure::new(
                Cause::StateLoadClosureMismatch,
                "authored stage lost its exact registered session",
            )
        })?;
        if staged
            .attempt
            .idempotency()
            .authored_stage_identity()
            .is_none()
            || staged.attempt.receipt_appendix().is_some()
        {
            return Err(Failure::new(
                Cause::StateLoadClosureMismatch,
                "authored stage is not owned by an envelope receipt",
            ));
        }
        let (roles, _) = super::super::native_publication_roles::collect(
            &self.graph.layout,
            staged.attempt.native_publication_roles(),
        )
        .ok_or_else(|| {
            Failure::new(
                Cause::StateLoadClosureMismatch,
                "authored stage lost its installed output roles",
            )
        })?;
        Ok((
            staged.batch().clone(),
            staged.content_attachments().clone(),
            roles,
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn coupled_writer_commands(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        session: WorthQueryProviderSessionView<'_>,
        producer: &str,
        family: &str,
        predecessor: Option<RelationalRequiredOutputPublication>,
    ) -> Result<CoupledWriterCommands, Failure> {
        let unavailable = || {
            Failure::new(
                Cause::StateLoadClosureMismatch,
                "coupled writer lost its registered publication input",
            )
        };
        let publisher = self
            .graph
            .application_publication
            .as_ref()
            .ok_or_else(unavailable)?;
        let attempts = self.attempts.lock().unwrap_or_else(|p| p.into_inner());
        let staged = attempts.staged_attempt(session).ok_or_else(unavailable)?;
        let attempt = staged.attempt;
        let idempotency = attempt.idempotency();
        let dependency = idempotency
            .producer_dependency_identity()
            .ok_or_else(unavailable)?;
        let (roles, all_created) = super::super::native_publication_roles::collect(
            &self.graph.layout,
            attempt.native_publication_roles(),
        )
        .ok_or_else(unavailable)?;
        let scope = attempt.affinity().operation_scope().scope();
        let meaning = RequiredOutputPublicationMeaning {
            family: family.to_owned(),
            implementation: idempotency
                .producing_implementation()
                .ok_or_else(unavailable)?,
            source_scope: worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::new(scope.partition_id()),
                scope.local_slot(),
                scope.generation(),
            ),
            source_partition: idempotency
                .source_partition_identity()
                .ok_or_else(unavailable)?,
            source_epoch: attempt
                .coupled_preparation()
                .ok_or_else(unavailable)?
                .completion
                .source()
                .checkpoint_identity(),
        };
        let batch = staged.batch().clone();
        let content = staged.content_attachments().clone();
        drop(attempts);
        use super::super::publication_posture::{self, ProducerPublicationPosture as Posture};
        let absence = if predecessor.is_some() {
            None
        } else {
            // No attempts lock is held while the existing Native owner admits absence.
            Some(
                self.graph
                    .with_runtime(execution, |native| {
                        native.readmit_required_output_route_absence(
                            execution,
                            &publisher.native,
                            basis,
                            &meaning,
                        )
                    })
                    .map_err(super::super::invariant_execution_failure::map_validation_failure)?
                    .ok_or_else(unavailable)?,
            )
        };
        let running = publisher.owner.admit_required_output(producer, dependency);
        let publication = match publication_posture::select(absence, all_created) {
            Posture::Ordinary(absence) => RelationalCoupledProducerPosture::Ordinary {
                admission: running,
                absence,
                meaning,
                roles,
            },
            Posture::Birth(absence) => {
                RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
                    admission: running,
                    absence: Some(absence),
                    meaning,
                    roles,
                })
            }
            Posture::Preserve => {
                let predecessor = predecessor.ok_or_else(unavailable)?;
                RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Preserve {
                    admission: publisher
                        .owner
                        .admit_required_output_succession(running, predecessor.authority())
                        .ok_or_else(unavailable)?,
                    predecessor,
                    meaning,
                    roles,
                })
            }
        };
        Ok(CoupledWriterCommands {
            publication,
            batch,
            content,
        })
    }
}

pub(in crate::domain_computation::primary_graph::provider) fn authored_retirement_input(
    layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
    staged: &WorthQueryStagedApplicationAttempt<'_>,
) -> Result<
    Option<(
        worth_relational::facade::identity::EntityId,
        Vec<worth_relational::facade::identity::EntityId>,
        Vec<worth_relational::facade::mvcc::RequiredOutputRoleTarget>,
    )>,
    Failure,
> {
    let anchor = super::super::authored_retirement_selection::authored_anchor(staged.attempt)
        .map_err(|_| {
            Failure::new(
                Cause::StateLoadClosureMismatch,
                "authored retirement lost its declared anchor correspondence",
            )
        })?;
    let Some((anchor, co_anchors)) = anchor else {
        return Ok(None);
    };
    let (roles, _) = super::super::native_publication_roles::collect(
        layout,
        staged.attempt.native_publication_roles(),
    )
    .ok_or_else(|| {
        Failure::new(
            Cause::StateLoadClosureMismatch,
            "authored retirement lost an installed role",
        )
    })?;
    Ok(Some((anchor, co_anchors, roles)))
}

pub(in crate::domain_computation::primary_graph::provider) fn authored_source(
    staged: &WorthQueryStagedApplicationAttempt<'_>,
) -> worth_relational::facade::identity::EntityId {
    let scope = staged.attempt.affinity().operation_scope().scope();
    worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::new(scope.partition_id()),
        scope.local_slot(),
        scope.generation(),
    )
}
