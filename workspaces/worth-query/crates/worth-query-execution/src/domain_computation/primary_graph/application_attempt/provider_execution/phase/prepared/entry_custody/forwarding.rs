use super::*;
use crate::domain_computation::application_aftermath::ApplicationCommitCausality;
use crate::domain_computation::primary_graph::application_attempt::HistoryProducerCommit;
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    >
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_with_output_observation(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
        retain_output_observation: bool,
        history: Option<&'_ dyn HistoryProducerCommit>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_with_causality(
            execution,
            idempotency,
            retain_output_observation,
            ApplicationCommitCausality::Ordinary,
            history,
            allocation_policy,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_with_causality(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        retain_output_observation: bool,
        causality: ApplicationCommitCausality,
        history: Option<&'_ dyn HistoryProducerCommit>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Some(denial) = plain_publication_posture(&self.program.read_set.admission) {
            return WorthQueryApplicationCommitOutcome::Denied(denial);
        }
        if self
            .program
            .read_set
            .admission
            .allowed_graph_contract()
            .aftermath()
            .is_some_and(|contract| contract.correction_disclosure().is_some())
            && self.program.read_set.admission.original_program().is_none()
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        let mut entry = self;
        if retain_output_observation {
            entry.program = entry.program.with_output_demand_observation();
        }
        let pending = match causality.admit(&entry.program.read_set.admission) {
            Ok(pending) => pending,
            Err(denial) => return WorthQueryApplicationCommitOutcome::Denied(denial),
        };
        entry.compare_and_commit_application_inner_with_currentness_and_aftermath(
            execution,
            idempotency,
            None,
            pending,
            history,
            allocation_policy,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_inner(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_inner_with_currentness(
            execution,
            idempotency,
            None,
            allocation_policy,
        )
    }
    pub(super) fn compare_and_commit_application_inner_with_currentness(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_inner_with_currentness_and_aftermath(
            execution,
            idempotency,
            elevation_currentness,
            None,
            None,
            allocation_policy,
        )
    }
}
fn plain_publication_posture<Schema, Operation, Input, Scope>(
    admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
) -> Option<WorthQueryApplicationCommitDenial> {
    if admission.has_elevation_lifecycle_binding() {
        return Some(WorthQueryApplicationCommitDenial::elevation_transition_required());
    }
    if admission
        .allowed_graph_contract()
        .execution_posture()
        .requires_delegation_activation()
    {
        return Some(WorthQueryApplicationCommitDenial::delegation_activation_required());
    }
    if admission
        .allowed_graph_contract()
        .execution_posture()
        .requires_capability_revocation()
    {
        return Some(WorthQueryApplicationCommitDenial::capability_revocation_required());
    }
    None
}
