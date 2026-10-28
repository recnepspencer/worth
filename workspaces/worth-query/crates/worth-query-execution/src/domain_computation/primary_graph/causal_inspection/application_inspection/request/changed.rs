//! A paired original request requires a distinct real diagnostic capability.
use super::super::result::WorthQueryComparedApplicationInspection;
use super::*;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::ApplicationQueryScopeBinding,
};

/// Selects comparison of two actual originals under the same installed binding.
pub struct WorthQueryChangedInspectionRequest<'a, 'p, 'r, 'o, 'source, Schema, Binding> {
    request: WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>,
    before: &'o WorthQueryApplicationQueryObservation,
    source: Option<
        &'source crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    >,
}

/// Reports Changed or Unchanged without disclosing Native cells. An optional
/// source candidate must separately pass the exact performed producer joins.
pub struct WorthQueryChangedReferenceOnlyInspectionRequest<'a, 'p, 'r, 'o, 'source, Schema, Binding>(
    WorthQueryChangedInspectionRequest<'a, 'p, 'r, 'o, 'source, Schema, Binding>,
);

impl<'a, 'p, 'r, 'o, Schema, Binding>
    WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
{
    pub fn why_changed(
        self,
        before: &'o WorthQueryApplicationQueryObservation,
    ) -> WorthQueryChangedInspectionRequest<'a, 'p, 'r, 'o, 'static, Schema, Binding> {
        WorthQueryChangedInspectionRequest {
            request: self,
            before,
            source: None,
        }
    }
}

impl<'a, 'p, 'r, 'o, 'source, Schema, Binding>
    WorthQueryChangedInspectionRequest<'a, 'p, 'r, 'o, 'source, Schema, Binding>
{
    /// Supply a genuine source candidate. Fresh admission precedes checking
    /// its availability; exact performed input/output evidence must connect it.
    pub fn caused_by<'candidate>(
        self,
        source: &'candidate crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> WorthQueryChangedInspectionRequest<'a, 'p, 'r, 'o, 'candidate, Schema, Binding> {
        WorthQueryChangedInspectionRequest {
            request: self.request,
            before: self.before,
            source: Some(source),
        }
    }

    pub fn reference_only(
        self,
    ) -> WorthQueryChangedReferenceOnlyInspectionRequest<'a, 'p, 'r, 'o, 'source, Schema, Binding>
    {
        WorthQueryChangedReferenceOnlyInspectionRequest(self)
    }
}

impl<'a, 'p, 'r, 'o, 'source, Schema, Binding>
    WorthQueryChangedReferenceOnlyInspectionRequest<'a, 'p, 'r, 'o, 'source, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    pub fn with_allocation_policy<'funding>(
        self,
        allocation_policy: ExecutionAllocationPolicy<'funding, 'funding>,
    ) -> WorthQueryChangedReferenceOnlyInspectionRequest<
        'funding,
        'p,
        'r,
        'o,
        'source,
        Schema,
        Binding,
    >
    where
        'a: 'funding,
    {
        WorthQueryChangedReferenceOnlyInspectionRequest(WorthQueryChangedInspectionRequest {
            request: WorthQueryApplicationInspectionRequest {
                runtime: self.0.request.runtime,
                external: self.0.request.external,
                request: self.0.request.request,
                original: self.0.request.original,
                allocation_policy,
                binding: PhantomData,
            },
            before: self.0.before,
            source: self.0.source,
        })
    }

    /// Public inspection metadata still requires actual governed payload policy.
    /// A Public query has no diagnostic field contract and refuses this mode.
    pub fn execute_with_diagnostic<Capability, Operation, Input>(
        self,
        diagnostic: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<WorthQueryComparedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    where
        Input: ApplicationCapabilityRequest<
            Schema,
            Capability,
            Scope = <Binding::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
    {
        let runtime = self.0.request.runtime;
        let request = self.0.request.request;
        self.0.request.execute_comparison_with(
            self.0.before,
            self.0.source,
            |execution, installed, principal, scope, memory| {
                let diagnostic = governed::prepare_capability(
                    runtime, request, execution, installed, principal, scope, diagnostic, memory,
                )?;
                Ok((None, diagnostic))
            },
        )
    }

    /// Both independent current capabilities bind to one fresh Inspection session.
    pub fn execute_governed<
        MetadataCapability,
        MetadataOperation,
        MetadataInput,
        DiagnosticCapability,
        DiagnosticOperation,
        DiagnosticInput,
    >(
        self,
        metadata: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            MetadataCapability,
            MetadataOperation,
            MetadataInput,
        >,
        diagnostic: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            DiagnosticCapability,
            DiagnosticOperation,
            DiagnosticInput,
        >,
    ) -> Result<WorthQueryComparedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    where
        MetadataInput: ApplicationCapabilityRequest<
            Schema,
            MetadataCapability,
            Scope = <Binding::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
        DiagnosticInput: ApplicationCapabilityRequest<
            Schema,
            DiagnosticCapability,
            Scope = <Binding::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
    {
        let runtime = self.0.request.runtime;
        let request = self.0.request.request;
        self.0.request.execute_comparison_with(
            self.0.before,
            self.0.source,
            |execution, installed, principal, scope, memory| {
                let metadata = governed::prepare_capability(
                    runtime, request, execution, installed, principal, scope, metadata, memory,
                )?;
                let diagnostic = governed::prepare_capability(
                    runtime, request, execution, installed, principal, scope, diagnostic, memory,
                )?;
                Ok((Some(metadata), diagnostic))
            },
        )
    }
}
