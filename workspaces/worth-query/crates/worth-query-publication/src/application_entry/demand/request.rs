use std::num::NonZeroUsize;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand;
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandControls {
    maximum_work: NonZeroUsize,
    maximum_retained_bytes: NonZeroUsize,
}

impl WorthQueryOutputDemandControls {
    pub const fn new(maximum_work: NonZeroUsize, maximum_retained_bytes: NonZeroUsize) -> Self {
        Self {
            maximum_work,
            maximum_retained_bytes,
        }
    }

    pub const fn maximum_work(self) -> NonZeroUsize {
        self.maximum_work
    }

    pub const fn maximum_retained_bytes(self) -> NonZeroUsize {
        self.maximum_retained_bytes
    }
}

#[derive(Debug)]
pub enum WorthQueryApplicationOutputDemandDenial {
    Source(super::super::WorthQueryApplicationRequestQueryDenial),
    Demand(worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial),
    MissingSource,
    FreshRequestMismatch,
    Superseded,
    Closed,
}

impl std::fmt::Display for WorthQueryApplicationOutputDemandDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "application output demand denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryApplicationOutputDemandDenial {}

pub struct WorthQueryApplicationOutputDemandRequest<
    'application,
    'principal,
    'scope,
    Schema,
    Demand,
> {
    pub(super) application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope WorthQueryRequestScope,
    pub(super) branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    pub(super) observation: Option<
        std::sync::Arc<
            worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
        >,
    >,
    pub(super) demand: Demand,
    pub(super) controls: Option<WorthQueryOutputDemandControls>,
}

impl<'application, 'principal, 'scope, Schema, Demand>
    WorthQueryApplicationOutputDemandRequest<'application, 'principal, 'scope, Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub(in crate::application_entry) const fn new(
        application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        demand: Demand,
    ) -> Self {
        Self {
            application,
            principal,
            scope,
            branch,
            observation: None,
            demand,
            controls: None,
        }
    }

    pub(in crate::application_entry) fn new_at(
        application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        observation: std::sync::Arc<
            worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
        >,
        demand: Demand,
    ) -> Self {
        Self {
            application,
            principal,
            scope,
            branch,
            observation: Some(observation),
            demand,
            controls: None,
        }
    }

    pub fn controls(mut self, controls: WorthQueryOutputDemandControls) -> Self {
        self.controls = Some(controls);
        self
    }
}
