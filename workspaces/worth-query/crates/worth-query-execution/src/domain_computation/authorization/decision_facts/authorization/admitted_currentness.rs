//! Currentness of a conventional Query Ability on one carried preparation meter.

use worth_relational::facade::authorization::RelationalAuthorizationObservationFreshness;

use super::bridge_matches;
use super::WorthQueryAuthorizationDecisionFact;

impl WorthQueryAuthorizationDecisionFact {
    /// The selected Query permission mints a conventional fact with no
    /// delegation. If a different fact reaches this narrow route, it cannot
    /// produce a fresh verdict without that owner's admitted recursion.
    pub(in crate::domain_computation) fn remains_conventional_current_admitted(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    ) -> Result<bool, super::super::admitted_currentness::AuthorizationCurrentnessStop> {
        if self.delegation.is_some()
            || self.delegation_activation.is_some()
            || self.recorded_actor.is_some()
        {
            return Ok(false);
        }
        let bridge_current = bridge.retains(&self.bridge);
        if !bridge_current {
            return Ok(false);
        }

        if !bridge_matches(
            &self.relational,
            &self.bridge,
            self.recorded_actor.as_deref(),
        ) {
            return Ok(false);
        }
        let freshness = runtime.compare_authorization_observation(
            &self.relational,
            snapshot.clone(),
            execution,
        );
        if let RelationalAuthorizationObservationFreshness::ExecutionStopped(cause) = freshness {
            return Err(
                super::super::admitted_currentness::AuthorizationCurrentnessStop::Execution(cause),
            );
        }
        Ok(freshness == RelationalAuthorizationObservationFreshness::Fresh)
    }
}
