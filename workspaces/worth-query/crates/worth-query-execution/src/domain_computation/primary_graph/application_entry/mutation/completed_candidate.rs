use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram;

/// Candidate program and typed consumer result completed by the same handler.
pub struct WorthQueryCompletedMutationCandidate<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    program: WorthQueryApplicationEffectProgram<
        Schema,
        Binding::Operation,
        Binding::Input,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    >,
    result: Binding::Result,
    planning_context: bool,
}

impl<Schema, Binding> WorthQueryCompletedMutationCandidate<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(in crate::domain_computation::primary_graph) fn new(
        program: WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        result: Binding::Result,
    ) -> Self {
        Self {
            program,
            result,
            planning_context: false,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn record_planning_context(
        &mut self,
        context: crate::domain_computation::primary_graph::DecisionContextUse,
    ) {
        self.planning_context = context.supports_candidate_planning();
    }
    pub(in crate::domain_computation::primary_graph) fn candidate_footprint(&self, execution: &worth_execution::ExecutionRequest<'_, '_>) -> Result<Option<crate::domain_computation::primary_graph::application_attempt::effect_program::candidate_planning::CandidateFootprint>, crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>{
        if !self.planning_context {
            return Ok(None);
        }
        self.program.candidate_footprint(execution)
    }
    pub(in crate::domain_computation::primary_graph) fn require_planned_support(
        &self,
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        applied: Option<
            &worth_relational::facade::runtime::RelationalPinnedRead<
                worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
            >,
        >,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    {
        self.program
            .require_planned_support(application, execution, applied)
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        Binding::Result,
    ) {
        (self.program, self.result)
    }
}
