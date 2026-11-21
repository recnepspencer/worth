use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

mod candidate;
mod completion;
mod decision_query_runtime;
mod input_capture;
mod preparation;
mod projection;
pub use candidate::build_handler_candidate;
pub use completion::PreparedMutationHandler;
pub use input_capture::PreparedMutationInputs;

use super::{
    HandlerExecutionDenial, HandlerResult, WorthQueryCompletedMutationCandidate,
    WorthQueryMutationHandlerExecutionReport, WorthQueryMutationHandlerWork,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

pub use worth_query_execution_lineage::facade::primary_graph::application_entry::mutation::execution::MutationHandlerExecutionDenial;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Runs the installed handler for one admitted request and builds its candidate.
    ///
    /// `identities` hold the request's key and input together with the identities
    /// encoded from exactly those two values, so the handler decides on the very
    /// input whose identity the candidate program records; no other input can be
    /// paired with them. The handler reads the key, input and identities from its
    /// `DecisionReader`, and the commit checks the recorded input identity against
    /// the idempotency binding. A capability admission also fixes the identity of
    /// the input it governed, and a request whose input encodes to another
    /// identity is refused with `InputNotAdmitted` before any read.
    pub fn execute_mutation_handler<Binding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        principal_identity: &Binding::PrincipalIdentity,
        admission: WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        HandlerResult<WorthQueryCompletedMutationCandidate<Schema, Binding>, Binding::Denial>,
        MutationHandlerExecutionDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.execute_mutation_handler_report::<Binding>(
            phase,
            identities,
            principal_identity,
            admission,
            allocation_policy,
        )
        .into_outcome()
    }

    /// Runs the installed handler once, retaining actual decision projection
    /// work on success, refusal and interruption. Entry denials report NotStarted.
    pub fn execute_mutation_handler_report<Binding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        principal_identity: &Binding::PrincipalIdentity,
        admission: WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryMutationHandlerExecutionReport<
        WorthQueryCompletedMutationCandidate<Schema, Binding>,
        Binding::Denial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.execute_mutation_handler_report_observing_contact::<Binding>(
            phase,
            identities,
            principal_identity,
            admission,
            || {},
            allocation_policy,
        )
    }

    pub fn execute_mutation_handler_report_observing_contact<Binding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        principal_identity: &Binding::PrincipalIdentity,
        admission: WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        on_contact: impl FnOnce(),
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryMutationHandlerExecutionReport<
        WorthQueryCompletedMutationCandidate<Schema, Binding>,
        Binding::Denial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        match self.prepare_mutation_handler_plan::<Binding>(
            phase,
            identities,
            principal_identity,
            admission,
            on_contact,
            allocation_policy,
        ) {
            Ok(prepared) => prepared.run(self, identities),
            Err(report) => report.into_uncompleted_report(),
        }
    }
}
