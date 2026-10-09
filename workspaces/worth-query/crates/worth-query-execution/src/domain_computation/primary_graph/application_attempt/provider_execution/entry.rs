mod output_source;

use crate::domain_computation::application_aftermath::ApplicationCommitCausality;
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
};
use super::elevation_currentness::WorthQueryElevationCommitCurrentness;
use super::phase::{
    finish_application_commit, prepare_application_commit, progress_application_commit,
    start_managed_application_commit, WorthQueryApplicationCommitPreparation,
    WorthQueryApplicationCommitPreparationRequest,
};
use super::program_occurrence_gate::{
    require_occurrence_acts_through, require_selected_program_matches_occurrence,
    resolve_occurrence_program,
};
use crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryPresentedProgram;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation) fn has_installed_application_program(&self) -> bool {
        self.program_support.is_some()
    }

    pub(in crate::domain_computation) fn direct_operation_commit_denial<Operation: 'static>(
        &self,
    ) -> Option<WorthQueryApplicationCommitDenial> {
        if self.operation_requires_workflow_authority::<Operation>() {
            Some(WorthQueryApplicationCommitDenial::workflow_authority_required())
        } else if self.operation_requires_application_program::<Operation>() {
            Some(WorthQueryApplicationCommitDenial::program_lane_required::<
                Operation,
            >())
        } else {
            None
        }
    }

    pub fn compare_and_commit_application<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if let Some(denial) = self.direct_operation_commit_denial::<Operation>() {
            return WorthQueryApplicationCommitOutcome::Denied(denial);
        }
        self.compare_and_commit_application_with_output_observation(
            program,
            idempotency,
            false,
            allocation_policy,
        )
    }

    /// Resolves the active occurrence, then checks selection and operation authority.
    fn require_occurrence_program_commit_binding<Operation, Input, Scope>(
        &self,
        program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        selected_program: Option<(
            &worth_query_declaration::facade::application_program::ApplicationProgramIdentity,
            &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        )>,
    ) -> Result<(), WorthQueryApplicationCommitOutcome>
    where
        Operation: 'static,
    {
        let Some(support) = self.program_support.as_ref() else {
            return Err(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            ));
        };
        let occurrence = resolve_occurrence_program(support, program)
            .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
        require_selected_program_matches_occurrence(&occurrence, selected_program)
            .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
        require_occurrence_acts_through::<Operation>(&occurrence)
            .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_program_action<
        Operation,
        Input,
        Scope,
    >(
        &self,
        presented: &WorthQueryPresentedProgram,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        causality: ApplicationCommitCausality<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if self
            .installed_conditionals
            .contains_operation::<Operation>()
            || !presented.acts_through_operation(std::any::TypeId::of::<Operation>())
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        let Some(support) = self.program_support.as_ref() else {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        };
        let occurrence = match resolve_occurrence_program(support, &program) {
            Ok(occurrence) => occurrence,
            Err(denial) => return WorthQueryApplicationCommitOutcome::Denied(denial),
        };
        if occurrence.rendering() != presented.rendering() {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::program_not_active_on_occurrence(
                    presented.identity(),
                    occurrence.entry().identity(),
                    occurrence.entry().revision(),
                ),
            );
        }
        self.compare_and_commit_application_with_causality(
            program,
            idempotency,
            false,
            causality,
            allocation_policy,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_conditional_operation<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if !self.has_installed_application_program() {
            return self.compare_and_commit_application(
                program,
                idempotency,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        }
        if !self
            .installed_conditionals
            .contains_operation::<Operation>()
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        if let Err(outcome) = self.require_occurrence_program_commit_binding(&program, None) {
            return outcome;
        }
        self.compare_and_commit_application_with_output_observation(
            program,
            idempotency,
            false,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    }

    fn compare_and_commit_application_with_output_observation<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        retain_output_observation: bool,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_with_causality(
            program,
            idempotency,
            retain_output_observation,
            ApplicationCommitCausality::Ordinary,
            allocation_policy,
        )
    }

    pub(crate) fn compare_and_commit_application_with_causality<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        retain_output_observation: bool,
        causality: ApplicationCommitCausality<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Some(denial) = plain_publication_posture(&program.read_set.admission) {
            return WorthQueryApplicationCommitOutcome::Denied(denial);
        }
        let program = if retain_output_observation {
            program.with_output_demand_observation()
        } else {
            program
        };
        let pending = match causality.admit(&program.read_set.admission) {
            Ok(pending) => pending,
            Err(denial) => return WorthQueryApplicationCommitOutcome::Denied(denial),
        };
        self.compare_and_commit_application_inner_with_currentness_and_aftermath(
            program,
            idempotency,
            None,
            pending,
            allocation_policy,
        )
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn compare_and_commit_application_inner<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_inner_with_currentness(
            program,
            idempotency,
            None,
            allocation_policy,
        )
    }

    pub(super) fn compare_and_commit_application_inner_with_currentness<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_inner_with_currentness_and_aftermath(
            program,
            idempotency,
            elevation_currentness,
            None,
            allocation_policy,
        )
    }

    #[cfg(test)]
    pub(crate) fn compare_and_commit_application_with_aftermath<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        aftermath_causality: WorthQueryPendingAftermathCausality,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Some(denial) = plain_publication_posture(&program.read_set.admission) {
            return WorthQueryApplicationCommitOutcome::Denied(denial);
        }
        self.compare_and_commit_application_inner_with_currentness_and_aftermath(
            program,
            idempotency,
            None,
            Some(aftermath_causality),
            allocation_policy,
        )
    }

    fn compare_and_commit_application_inner_with_currentness_and_aftermath<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let prepared = match prepare_application_commit(
            self,
            WorthQueryApplicationCommitPreparationRequest::new(
                program,
                idempotency,
                elevation_currentness,
                aftermath_causality,
            ),
        ) {
            WorthQueryApplicationCommitPreparation::Ready(prepared) => prepared,
            WorthQueryApplicationCommitPreparation::Terminal(outcome) => return outcome,
        };
        let running = match start_managed_application_commit(self, prepared) {
            Ok(running) => running,
            Err(outcome) => return outcome,
        };
        finish_application_commit(
            self,
            progress_application_commit(self, running, allocation_policy),
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
