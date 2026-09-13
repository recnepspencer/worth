use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryAuthenticationAdapterIdentity,
    WorthQueryPrincipalAttribute, WorthQueryRequestScope,
};
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryExternalPrincipalIdentity,
};

use super::WorthQueryApplicationQueryAccessReceipt;

pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationQueryRequestAffinity {
    principal: WorthQueryExternalPrincipalIdentity,
    adapter: WorthQueryAuthenticationAdapterIdentity,
    binding: ApplicationSchemaBindingIdentity,
    attributes: Vec<WorthQueryPrincipalAttribute>,
    valid_until: std::time::Instant,
    request_scope: WorthQueryRequestScope,
}

impl WorthQueryApplicationQueryRequestAffinity {
    pub(in crate::domain_computation::primary_graph) fn new<
        Schema,
        Principal,
        PrincipalIdentity,
    >(
        principal: &super::super::WorthQueryAuthenticatedPrincipal<
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        request_scope: &WorthQueryRequestScope,
    ) -> Self {
        Self {
            principal: principal.external_identity().clone(),
            adapter: principal.adapter_identity().clone(),
            binding: principal.binding_identity().clone(),
            attributes: principal.attributes().to_vec(),
            valid_until: principal.valid_until(),
            request_scope: request_scope.clone(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn admits<Schema>(
        &self,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
    ) -> bool {
        principal.identity() == &self.principal
            && principal.adapter_identity() == &self.adapter
            && principal.binding_identity() == &self.binding
            && principal.attributes() == self.attributes.as_slice()
            && principal.valid_until() == self.valid_until
            && !principal.is_expired()
            && self.request_scope.same_request(request_scope)
            && request_scope.interruption().is_none()
    }
}

/// Query-admitted consumer shape. Construction is private to completed query
/// lanes, after governed projection has replaced every protected slot with a
/// typed disclosed-or-omitted value.
///
/// A consumer holding a terminal receipt still cannot construct an admitted
/// result and bypass governed projection:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryAdmittedDisclosedApplicationResult,
///     WorthQueryApplicationQueryAccessReceipt,
/// };
///
/// fn counterfeit(
///     receipt: WorthQueryApplicationQueryAccessReceipt,
/// ) -> WorthQueryAdmittedDisclosedApplicationResult<(), ()> {
///     WorthQueryAdmittedDisclosedApplicationResult::new(vec![()], receipt)
/// }
/// ```
pub struct WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult> {
    rows: Vec<QueryResult>,
    observed_sources: Vec<super::WorthQueryObservedSource<Query>>,
    request_affinity: Option<WorthQueryApplicationQueryRequestAffinity>,
    receipt: WorthQueryApplicationQueryAccessReceipt,
}

/// Linear proof that one governed application query completed disclosure.
///
/// Demand progression consumes this proof before observing shared state, so a
/// descriptive source receipt cannot stand in for a fresh disclosed read.
pub struct WorthQueryApplicationOutputDemandDisclosure<Query> {
    observed_sources: Vec<super::WorthQueryObservedSource<Query>>,
    request_affinity: Option<WorthQueryApplicationQueryRequestAffinity>,
    receipt: WorthQueryApplicationQueryAccessReceipt,
}

impl<Query, QueryResult> WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult> {
    pub(super) fn new(
        rows: Vec<QueryResult>,
        receipt: WorthQueryApplicationQueryAccessReceipt,
    ) -> Self {
        Self {
            rows,
            observed_sources: Vec::new(),
            request_affinity: None,
            receipt,
        }
    }

    pub(super) fn new_with_sources(
        rows: Vec<QueryResult>,
        observed_sources: Vec<super::WorthQueryObservedSource<Query>>,
        request_affinity: WorthQueryApplicationQueryRequestAffinity,
        receipt: WorthQueryApplicationQueryAccessReceipt,
    ) -> Self {
        Self {
            rows,
            observed_sources,
            request_affinity: Some(request_affinity),
            receipt,
        }
    }

    pub fn rows(&self) -> &[QueryResult] {
        &self.rows
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationQueryAccessReceipt {
        &self.receipt
    }

    /// Consumes the governed result at a downstream publication boundary.
    /// The receipt remains execution-owned and must be projected before drop.
    pub fn into_parts(
        self,
    ) -> (
        Vec<QueryResult>,
        WorthQueryApplicationOutputDemandDisclosure<Query>,
    ) {
        (
            self.rows,
            WorthQueryApplicationOutputDemandDisclosure {
                observed_sources: self.observed_sources,
                request_affinity: self.request_affinity,
                receipt: self.receipt,
            },
        )
    }
}

impl<Query> WorthQueryApplicationOutputDemandDisclosure<Query> {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        Vec<super::WorthQueryObservedSource<Query>>,
        Option<WorthQueryApplicationQueryRequestAffinity>,
        WorthQueryApplicationQueryAccessReceipt,
    ) {
        (self.observed_sources, self.request_affinity, self.receipt)
    }

    pub fn observed_sources(&self) -> &[super::WorthQueryObservedSource<Query>] {
        &self.observed_sources
    }
}
