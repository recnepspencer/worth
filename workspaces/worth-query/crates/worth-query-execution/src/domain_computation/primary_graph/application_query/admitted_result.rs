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
    result_set_observation: Option<super::WorthQueryObservedResultSet<Query>>,
    request_affinity: Option<WorthQueryApplicationQueryRequestAffinity>,
    receipt: WorthQueryApplicationQueryAccessReceipt,
}

/// Linear proof that one governed application query completed disclosure.
///
/// Demand progression consumes this proof before observing shared state, so a
/// descriptive source receipt cannot stand in for a fresh disclosed read.
pub struct WorthQueryApplicationOutputDemandDisclosure<Query> {
    observed_sources: Vec<super::WorthQueryObservedSource<Query>>,
    result_set_observation: Option<super::WorthQueryObservedResultSet<Query>>,
    request_affinity: Option<WorthQueryApplicationQueryRequestAffinity>,
    receipt: WorthQueryApplicationQueryAccessReceipt,
}

/// Owner-issued pairing of disclosed projection values with their exact source
/// evidence. Consumers can inspect values but cannot substitute them before
/// required-output admission.
pub struct WorthQueryApplicationOutputDemandSource<Query, QueryResult> {
    rows: Vec<QueryResult>,
    disclosure: WorthQueryApplicationOutputDemandDisclosure<Query>,
}

impl<Query, QueryResult> WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult> {
    pub(super) fn new(
        rows: Vec<QueryResult>,
        receipt: WorthQueryApplicationQueryAccessReceipt,
    ) -> Self {
        Self {
            rows,
            observed_sources: Vec::new(),
            result_set_observation: None,
            request_affinity: None,
            receipt,
        }
    }

    pub(super) fn new_with_sources(
        rows: Vec<QueryResult>,
        observed_sources: Vec<super::WorthQueryObservedSource<Query>>,
        result_set_observation: super::WorthQueryObservedResultSet<Query>,
        request_affinity: WorthQueryApplicationQueryRequestAffinity,
        receipt: WorthQueryApplicationQueryAccessReceipt,
    ) -> Self {
        Self {
            rows,
            observed_sources,
            result_set_observation: Some(result_set_observation),
            request_affinity: Some(request_affinity),
            receipt,
        }
    }

    pub fn rows(&self) -> &[QueryResult] {
        &self.rows
    }

    pub fn result_set_observation(&self) -> Option<&super::WorthQueryObservedResultSet<Query>> {
        self.result_set_observation.as_ref()
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
                result_set_observation: self.result_set_observation,
                request_affinity: self.request_affinity,
                receipt: self.receipt,
            },
        )
    }

    pub fn into_output_demand_source(
        self,
    ) -> WorthQueryApplicationOutputDemandSource<Query, QueryResult> {
        let (rows, disclosure) = self.into_parts();
        WorthQueryApplicationOutputDemandSource { rows, disclosure }
    }
}

impl<Query, QueryResult> WorthQueryApplicationOutputDemandSource<Query, QueryResult> {
    pub fn rows(&self) -> &[QueryResult] {
        &self.rows
    }

    pub fn observed_sources(&self) -> &[super::WorthQueryObservedSource<Query>] {
        self.disclosure.observed_sources()
    }

    pub fn result_set_observation(&self) -> Option<&super::WorthQueryObservedResultSet<Query>> {
        self.disclosure.result_set_observation.as_ref()
    }

    pub fn into_result_set_observation(self) -> Option<super::WorthQueryObservedResultSet<Query>> {
        self.disclosure.result_set_observation
    }

    pub(in crate::domain_computation::primary_graph) fn into_single_source(
        self,
    ) -> Option<(QueryResult, super::WorthQueryObservedSource<Query>)> {
        let (mut sources, _, _) = self.disclosure.into_parts();
        let mut rows = self.rows.into_iter();
        let pair = (rows.next()?, sources.pop()?);
        (rows.next().is_none() && sources.is_empty()).then_some(pair)
    }

    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        Vec<QueryResult>,
        WorthQueryApplicationOutputDemandDisclosure<Query>,
    ) {
        (self.rows, self.disclosure)
    }

    pub fn receipt(&self) -> &WorthQueryApplicationQueryAccessReceipt {
        &self.disclosure.receipt
    }

    pub fn into_disclosure(self) -> WorthQueryApplicationOutputDemandDisclosure<Query> {
        self.disclosure
    }

    pub fn into_rows(self) -> Vec<QueryResult> {
        self.rows
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
