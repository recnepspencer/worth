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
use super::program_occurrence_gate::resolve_occurrence_program;
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

    pub fn compare_and_commit_application<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if self.has_installed_application_program()
            || self
                .program_required_operations
                .contains(&std::any::TypeId::of::<Operation>())
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        self.compare_and_commit_application_with_output_observation(program, idempotency, false)
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_required_output_source<
        Operation,
        Input,
        Scope,
    >(
        &self,
        presented: &WorthQueryPresentedProgram<'_>,
        source_binding: std::any::TypeId,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(outcome) =
            self.require_occurrence_owns_output_source(presented, source_binding, &program)
        {
            return outcome;
        }
        self.compare_and_commit_application_with_output_observation(program, idempotency, true)
    }

    /// Requires that the program presented for this output source is the one
    /// active on the attempt's own occurrence, and that the occurrence's
    /// program acts through the mutation binding producing the source.
    fn require_occurrence_owns_output_source<Operation, Input, Scope>(
        &self,
        presented: &WorthQueryPresentedProgram<'_>,
        source_binding: std::any::TypeId,
        program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryApplicationCommitOutcome> {
        let Some(support) = self.program_support.as_ref() else {
            return Err(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            ));
        };
        let occurrence = resolve_occurrence_program(support, program)
            .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
        if occurrence.rendering() != presented.rendering() {
            return Err(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::program_not_active_on_occurrence(
                    presented.identity(),
                    occurrence.entry().identity(),
                ),
            ));
        }
        if !occurrence
            .entry()
            .acts_through_mutation_binding(source_binding)
        {
            return Err(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            ));
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_program_output_producer<
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
        if !self
            .installed_conditionals
            .contains_operation::<Operation>()
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        if let Err(outcome) = self.require_occurrence_acts_through(&program) {
            return outcome;
        }
        self.compare_and_commit_application_with_output_observation(program, idempotency, true)
    }

    /// Requires that the program active on this attempt's occurrence acts
    /// through the operation being committed.
    fn require_occurrence_acts_through<Operation, Input, Scope>(
        &self,
        program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
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
        if !occurrence
            .entry()
            .acts_through_operation(std::any::TypeId::of::<Operation>())
        {
            return Err(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            ));
        }
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
                ),
            );
        }
        self.compare_and_commit_application_with_output_observation(program, idempotency, false)
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
            return self.compare_and_commit_application(program, idempotency);
        }
        if !self
            .installed_conditionals
            .contains_operation::<Operation>()
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        if let Err(outcome) = self.require_occurrence_acts_through(&program) {
            return outcome;
        }
        self.compare_and_commit_application_with_output_observation(program, idempotency, false)
    }

    fn compare_and_commit_application_with_output_observation<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        retain_output_observation: bool,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if program.read_set.admission.has_elevation_lifecycle_binding() {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_transition_required(),
            );
        }
        if program
            .read_set
            .admission
            .allowed_graph_contract()
            .execution_posture()
            .requires_delegation_activation()
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::delegation_activation_required(),
            );
        }
        if program
            .read_set
            .admission
            .allowed_graph_contract()
            .execution_posture()
            .requires_capability_revocation()
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::capability_revocation_required(),
            );
        }
        let program = if retain_output_observation {
            program.with_output_demand_observation()
        } else {
            program
        };
        self.compare_and_commit_application_inner(program, idempotency)
    }

    pub(super) fn compare_and_commit_application_inner<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_inner_with_currentness(program, idempotency, None)
    }

    pub(super) fn compare_and_commit_application_inner_with_currentness<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_inner_with_currentness_and_aftermath(
            program,
            idempotency,
            elevation_currentness,
            None,
        )
    }

    pub(crate) fn compare_and_commit_application_with_aftermath<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        aftermath_causality: WorthQueryPendingAftermathCausality,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_inner_with_currentness_and_aftermath(
            program,
            idempotency,
            None,
            Some(aftermath_causality),
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
        finish_application_commit(self, progress_application_commit(self, running))
    }
}
