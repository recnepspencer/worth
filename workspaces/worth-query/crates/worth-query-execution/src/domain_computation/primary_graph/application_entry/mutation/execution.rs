use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

mod completion;
mod preparation;
pub(in crate::domain_computation::primary_graph) use completion::PreparedMutationHandler;

use super::{
    HandlerExecutionDenial, HandlerResult, WorthQueryCompletedMutationCandidate,
    WorthQueryMutationHandlerExecutionReport, WorthQueryMutationHandlerWork,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenial,
    WorthQueryOperationProjectionDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

/// Refusal to run a mutation handler for an admitted operation.
///
/// No candidate was produced and nothing was committed. A domain denial,
/// cancellation, or deadline from the handler is not here: it arrives inside
/// the `Ok` `HandlerResult`.
#[derive(Debug)]
pub enum MutationHandlerExecutionDenial {
    /// Another installed runtime lent the phase; no handler or read ran.
    ForeignAdvancementPhase,
    /// The projection that feeds the handler's decision could not be read.
    Projection(WorthQueryOperationProjectionDenial),
    /// The read attempt, resource reservation, or candidate program failed.
    Attempt(WorthQueryApplicationAttemptDenial),
    /// The handler returned `ExecutionDenied`.
    Handler(HandlerExecutionDenial),
    /// The binding is a workflow control step the workflow kernel records
    /// itself; no handler serves it, so this lane refuses it before any read.
    WorkflowControl,
    /// The admission governed a different input than the one the request
    /// carries, so no handler ran on it.
    InputNotAdmitted,
    /// This runtime installed no handler for the binding, so no handler ran.
    HandlerNotInstalled,
}

impl std::fmt::Display for MutationHandlerExecutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForeignAdvancementPhase => {
                formatter.write_str("the phase belongs to another installed runtime")
            }
            Self::Projection(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
            Self::Handler(denial) => denial.fmt(formatter),
            Self::WorkflowControl => formatter.write_str(
                "the workflow kernel records this control binding; no mutation handler serves it",
            ),
            Self::InputNotAdmitted => formatter
                .write_str("the admission governed a different input than the request carries"),
            Self::HandlerNotInstalled => {
                formatter.write_str("this runtime installed no handler for the mutation binding")
            }
        }
    }
}

impl std::error::Error for MutationHandlerExecutionDenial {}

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

    pub(in crate::domain_computation::primary_graph) fn execute_mutation_handler_report_observing_contact<
        Binding,
    >(
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
            Ok(prepared) => prepared.run_serial(self, identities),
            Err(report) => report,
        }
    }
}
