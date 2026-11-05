//! The runtime's consumer methods over `domain_computation::primary_graph::application_entry`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_entry::mutation::HandlerExecutionDenial;
use crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult;
use crate::domain_computation::primary_graph::application_entry::mutation::WorthQueryCompletedMutationCandidate;
use crate::domain_computation::primary_graph::application_entry::mutation::WorthQueryMutationHandlerExecutionReport;
use crate::domain_computation::primary_graph::application_entry::mutation::WorthQueryMutationHandlerWork;
use crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess;
use crate::domain_computation::primary_graph::MutationHandlerExecutionDenial;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
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
        <Self as crate::domain_computation::primary_graph::application_entry::ExecutionOps<
            Schema,
        >>::execute_mutation_handler(
            self,
            phase,
            identities,
            principal_identity,
            admission,
            allocation_policy,
        )
    }

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
        <Self as crate::domain_computation::primary_graph::application_entry::ExecutionOps<
            Schema,
        >>::execute_mutation_handler_report(
            self,
            phase,
            identities,
            principal_identity,
            admission,
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
        <Self as crate::domain_computation::primary_graph::application_entry::ExecutionOps<
            Schema,
        >>::execute_mutation_handler_report_observing_contact(
            self,
            phase,
            identities,
            principal_identity,
            admission,
            on_contact,
            allocation_policy,
        )
    }
}

impl<S: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn execute_mutation_handler_report_with_query_reads<B: ApplicationMutationBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        principal_identity: &B::PrincipalIdentity,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        admission: WorthQueryAdmittedApplicationOperation<
            S,
            B::Operation,
            B::Input,
            <B::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope,
        >,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryMutationHandlerExecutionReport<
        WorthQueryCompletedMutationCandidate<S, B>,
        B::Denial,
    > {
        <Self as crate::domain_computation::primary_graph::application_entry::DecisionQueryOps<S>>::execute_mutation_handler_report_with_query_reads(self, phase, identities, principal_identity, principal, admission, allocation)
    }
}
