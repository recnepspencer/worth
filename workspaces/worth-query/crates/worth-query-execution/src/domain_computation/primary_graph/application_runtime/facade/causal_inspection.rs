//! The runtime's consumer methods over `domain_computation::primary_graph::causal_inspection`; each only calls its subsystem.
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::primary_graph::causal_inspection::application_inspection::request::*;
use crate::domain_computation::primary_graph::causal_inspection::application_inspection::result::WorthQueryPerformedSourceInspection;
use crate::domain_computation::primary_graph::causal_inspection::application_inspection::stop::InspectionCause;
use crate::domain_computation::primary_graph::causal_inspection::application_inspection::stop::WorthQueryApplicationInspectionDenial;
use crate::domain_computation::primary_graph::causal_inspection::application_inspection::WorthQueryObservedApplicationInspection;
use crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationQueryObservation;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use std::marker::PhantomData;
use worth_execution::ExecutionAllocationPolicy;
use worth_execution::ExecutionMemoryReservation;
use worth_execution::ExecutionRequest;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_operation::ApplicationSourceDisclosureBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
use worth_query_installation::facade::ApplicationSchema;
impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn inspect_application_observation<'a, 'p, 'r, 'o, Binding>(
        &'a self,
        external: &'p WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &'r WorthQueryRequestScope,
        original: &'o WorthQueryApplicationQueryObservation,
    ) -> WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
    where
        Binding: ApplicationQueryBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::causal_inspection::RequestOps<Schema>>::inspect_application_observation(self, external, request, original)
    }

    pub fn inspect_application_source<
        'a,
        'p,
        'r,
        'source,
        Disclosure: ApplicationSourceDisclosureBinding<Schema>,
    >(
        &'a self,
        external: &'p WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &'r WorthQueryRequestScope,
        source: &'source WorthQueryApplicationCommitReceipt,
    ) -> WorthQuerySourceInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure> {
        <Self as crate::domain_computation::primary_graph::causal_inspection::SourceOps<Schema>>::inspect_application_source(self, external, request, source)
    }
}
