//! Native publication admission uses only the genuine registered writer's effects.
use super::*;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind as Cause, WorthQueryInvariantExecutionFailure as Failure,
};
use worth_relational::facade::{
    mvcc::{
        RelationalRequiredOutputPublication, RequiredOutputPublication,
        RequiredOutputPublicationMeaning,
    },
    transactions::WorkerIntentBatch,
};

pub(in crate::domain_computation::primary_graph) struct CoupledWriterCommands {
    pub(in crate::domain_computation::primary_graph) publication: RequiredOutputPublication,
    pub(in crate::domain_computation::primary_graph) batch: WorkerIntentBatch,
}
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn coupled_writer_commands(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        producer: &str,
        family: &str,
        predecessor: RelationalRequiredOutputPublication,
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
        let running = publisher.owner.admit_required_output(producer, dependency);
        let admission = publisher
            .owner
            .admit_required_output_succession(running, predecessor.authority())
            .ok_or_else(unavailable)?;
        let (roles, _) = super::super::native_publication_roles::collect(
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
        Ok(CoupledWriterCommands {
            publication: RequiredOutputPublication::Preserve {
                admission,
                predecessor,
                meaning,
                roles,
            },
            batch: staged.batch().clone(),
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
