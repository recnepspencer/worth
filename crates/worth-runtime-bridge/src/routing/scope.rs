use std::sync::Arc;

use crate::policy::BridgeRoutePlanningPolicy;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RouteScope {
    route_planning_policy: Option<BridgeRoutePlanningPolicy>,
    route_planning_policy_digest: Option<Arc<str>>,
    omit_record_entries: bool,
}

impl RouteScope {
    pub(crate) fn begin() -> Self {
        Self {
            route_planning_policy: None,
            route_planning_policy_digest: None,
            omit_record_entries: false,
        }
    }

    pub(crate) fn with_route_planning_policy(
        mut self,
        route_planning_policy: BridgeRoutePlanningPolicy,
    ) -> Self {
        self.route_planning_policy_digest =
            Some(Arc::from(route_planning_policy.digest().to_owned()));
        self.route_planning_policy = Some(route_planning_policy);
        self
    }

    pub(crate) fn with_route_planning_policy_digest(
        mut self,
        route_planning_policy_digest: impl Into<Arc<str>>,
    ) -> Self {
        self.route_planning_policy = None;
        self.route_planning_policy_digest = Some(route_planning_policy_digest.into());
        self
    }

    pub(crate) fn for_standard_delivery(mut self, default_retention: bool) -> Self {
        self.omit_record_entries = !self.retains_route_record(default_retention);
        self
    }

    pub(crate) fn omit_record_entries(&self) -> bool {
        self.omit_record_entries
    }

    pub(crate) fn retains_route_record(&self, default_retention: bool) -> bool {
        !self.omit_record_entries
            && self.route_planning_policy().map_or(
                default_retention,
                BridgeRoutePlanningPolicy::route_artifacts,
            )
    }

    pub(crate) fn route_planning_policy(&self) -> Option<&BridgeRoutePlanningPolicy> {
        self.route_planning_policy.as_ref()
    }

    pub(crate) fn route_planning_policy_digest(&self) -> Option<&str> {
        self.route_planning_policy_digest.as_deref()
    }
}
