use std::num::NonZeroUsize;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand;
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;

/// Optional restrictions on installed host policy. Ordinary graph authors use
/// `Default::default()`; absent restrictions use finite installed host limits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryOutputDemandControls {
    maximum_work: Option<NonZeroUsize>,
    maximum_source_currentness_work: Option<NonZeroUsize>,
    maximum_retained_bytes: Option<NonZeroUsize>,
    maximum_settlement_attempts: Option<NonZeroUsize>,
}

impl WorthQueryOutputDemandControls {
    pub const fn host_policy() -> Self {
        Self {
            maximum_work: None,
            maximum_source_currentness_work: None,
            maximum_retained_bytes: None,
            maximum_settlement_attempts: None,
        }
    }

    /// Restricts both work lanes and retained bytes; settlement uses host policy.
    pub const fn new(maximum_work: NonZeroUsize, maximum_retained_bytes: NonZeroUsize) -> Self {
        Self {
            maximum_work: Some(maximum_work),
            maximum_source_currentness_work: Some(maximum_work),
            maximum_retained_bytes: Some(maximum_retained_bytes),
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
            self.maximum_source_currentness_work
                .map_or(host.source_currentness_work(), NonZeroUsize::get),
            self.maximum_work
                .map_or(host.producer_work(), NonZeroUsize::get),
            self.maximum_retained_bytes
                .map_or(host.producer_retained_bytes(), NonZeroUsize::get),
            self.maximum_settlement_attempts
                .map_or(host.settlement_attempts(), NonZeroUsize::get),
        )
    }

    pub(in crate::application_entry) fn from_limits(
        limits: worth_query_execution::facade::runtime::WorthQueryOutputDemandLimits,
    ) -> Self {
        Self::new(
            NonZeroUsize::new(limits.producer_work()).expect("positive admitted producer limit"),
            NonZeroUsize::new(limits.producer_retained_bytes())
                .expect("positive admitted retained limit"),
        )
        .source_currentness_work(
            NonZeroUsize::new(limits.source_currentness_work())
                .expect("positive admitted source limit"),
        )
        .settlement_attempts(
            NonZeroUsize::new(limits.settlement_attempts())
                .expect("positive admitted settlement limit"),
        )
    }

    pub const fn maximum_work(self) -> Option<NonZeroUsize> {
        self.maximum_work
    }
    pub const fn maximum_source_currentness_work(self) -> Option<NonZeroUsize> {
        self.maximum_source_currentness_work
    }
    pub const fn maximum_retained_bytes(self) -> Option<NonZeroUsize> {
        self.maximum_retained_bytes
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
    fn absent_restrictions_use_the_actual_host_not_standard_values() {
        let large = NonZeroUsize::new(8_388_608).unwrap();
        let profile = WorthQueryOutputDemandResourceProfile::bounded(
            large,
            large,
            large,
            NonZeroUsize::new(128).unwrap(),
        );
        let controls = WorthQueryOutputDemandControls::default();
        assert_eq!(controls, WorthQueryOutputDemandControls::host_policy());
        assert_eq!(controls.maximum_work(), None);
        assert_eq!(controls.maximum_retained_bytes(), None);
        assert_eq!(controls.maximum_settlement_attempts(), None);
        assert_eq!(controls.resolve(profile), profile.limits());
    }

    #[test]
    fn explicit_caps_narrow_both_work_lanes_and_can_separate_source_work() {
        let controls = WorthQueryOutputDemandControls::new(
            NonZeroUsize::new(7).unwrap(),
            NonZeroUsize::new(11).unwrap(),
        );
        let profile = WorthQueryOutputDemandResourceProfile::standard();
        let limits = controls.resolve(profile);
        assert_eq!(limits.source_currentness_work(), 7);
        assert_eq!(limits.producer_work(), 7);
        assert_eq!(limits.producer_retained_bytes(), 11);
        assert_eq!(limits.settlement_attempts(), 64);
        let separated = controls
            .source_currentness_work(NonZeroUsize::new(19).unwrap())
            .settlement_attempts(NonZeroUsize::new(2).unwrap())
            .resolve(profile);
        assert_eq!(separated.source_currentness_work(), 19);
        assert_eq!(separated.producer_work(), 7);
        assert_eq!(separated.settlement_attempts(), 2);
    }
}
