use std::num::NonZeroUsize;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand;
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;

/// Optional restrictions on installed host policy. Ordinary graph authors use
/// `Default::default()`; absent work restrictions impose no intrinsic work ceiling.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryOutputDemandControls {
    maximum_source_currentness_work: Option<NonZeroUsize>,
    maximum_settlement_attempts: Option<NonZeroUsize>,
}

impl WorthQueryOutputDemandControls {
    pub const fn host_policy() -> Self {
        Self {
            maximum_source_currentness_work: None,
            maximum_settlement_attempts: None,
        }
    }

    pub const fn source_currentness_work(mut self, maximum: NonZeroUsize) -> Self {
        self.maximum_source_currentness_work = Some(maximum);
        self
    }

    pub const fn settlement_attempts(mut self, maximum: NonZeroUsize) -> Self {
        self.maximum_settlement_attempts = Some(maximum);
        self
    }

    pub(in crate::application_entry) fn resolve(
        self,
        profile: worth_query_execution::facade::runtime::WorthQueryOutputDemandResourceProfile,
    ) -> worth_query_execution::facade::runtime::WorthQueryOutputDemandLimits {
        let host = profile.limits();
        host.restricted(
            self.maximum_source_currentness_work.map(NonZeroUsize::get),
            self.maximum_settlement_attempts.map(NonZeroUsize::get),
        )
    }

    pub(in crate::application_entry) fn from_limits(
        limits: worth_query_execution::facade::runtime::WorthQueryOutputDemandLimits,
    ) -> Self {
        Self {
            maximum_source_currentness_work: limits
                .source_currentness_work()
                .map(|work| NonZeroUsize::new(work).expect("positive selected currentness limit")),
            maximum_settlement_attempts: limits.settlement_attempts().and_then(NonZeroUsize::new),
        }
    }

    pub const fn maximum_source_currentness_work(self) -> Option<NonZeroUsize> {
        self.maximum_source_currentness_work
    }
    pub const fn maximum_settlement_attempts(self) -> Option<NonZeroUsize> {
        self.maximum_settlement_attempts
    }
}

/// Why an output demand did not start or settle. `FreshRequestMismatch` means the fresh
/// request came from another runtime or carried no observation.
#[derive(Debug)]
pub enum WorthQueryApplicationOutputDemandDenial {
    Source(super::super::WorthQueryApplicationRequestQueryDenial),
    Demand(worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial),
    MissingSource,
    FreshRequestMismatch,
    ProgramOutputUndeclared,
    Superseded,
    Closed,
}

impl std::fmt::Display for WorthQueryApplicationOutputDemandDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "application output demand denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryApplicationOutputDemandDenial {}

/// A request to demand a declared output. Omitted controls use installed host policy;
/// explicit controls can only restrict that policy.
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

#[cfg(test)]
mod tests {
    use super::*;
    use worth_query_execution::facade::runtime::WorthQueryOutputDemandResourceProfile;

    #[test]
    fn default_controls_preserve_host_policy_and_explicit_controls_narrow_it() {
        let profile = WorthQueryOutputDemandResourceProfile::bounded(
            NonZeroUsize::new(19).unwrap(),
            NonZeroUsize::new(8).unwrap(),
        );
        let controls = WorthQueryOutputDemandControls::default();
        assert_eq!(controls, WorthQueryOutputDemandControls::host_policy());
        assert_eq!(controls.resolve(profile), profile.limits());
        let narrowed = controls
            .source_currentness_work(NonZeroUsize::new(7).unwrap())
            .settlement_attempts(NonZeroUsize::new(2).unwrap())
            .resolve(profile);
        assert_eq!(narrowed.source_currentness_work(), Some(7));
        assert_eq!(narrowed.settlement_attempts(), Some(2));
    }
}
