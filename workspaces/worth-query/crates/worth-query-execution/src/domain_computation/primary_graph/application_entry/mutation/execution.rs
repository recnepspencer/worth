use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

mod candidate;
mod completion;
mod decision_query;
pub(super) use decision_query::native_decision_cause;
mod preparation;
pub(in crate::domain_computation::primary_graph) use candidate::build_handler_candidate;
pub(in crate::domain_computation::primary_graph) use completion::PreparedMutationHandler;

use super::{
    HandlerExecutionDenial, HandlerResult, WorthQueryCompletedMutationCandidate,
    WorthQueryMutationHandlerExecutionReport, WorthQueryMutationHandlerWork,
};
use {
    crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial,
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation,
    crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenial,
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
};

/// Refusal to run a mutation handler for an admitted operation.
///
/// No candidate was produced and nothing was committed. A domain denial,
/// cancellation, or deadline from the handler is not here: it arrives inside
/// the `Ok` `HandlerResult`.
#[derive(Debug)]
pub enum MutationHandlerExecutionDenial {
    /// Another installed runtime lent the phase; no handler or read ran.
    ForeignAdvancementPhase(
        crate::domain_computation::request_execution::WorthQueryForeignAdvancementPhase,
    ),
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
            Self::ForeignAdvancementPhase(_) => {
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

pub trait ExecutionOps<Schema>:
    crate::domain_computation::primary_graph::application_entry::host::ApplicationEntryHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn execute_mutation_handler<Binding>(
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

    fn execute_mutation_handler_report<Binding>(
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

    fn execute_mutation_handler_report_observing_contact<Binding>(
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
            Ok(prepared) => prepared.run(self.application_entry_host(), identities),
            Err(report) => report,
        }
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_entry::host::ApplicationEntryHost<
                Schema,
            > + ?Sized,
    > ExecutionOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}
pub use decision_query::DecisionQueryOps;
pub(in crate::domain_computation::primary_graph) use preparation::PreparationScopedOps;
