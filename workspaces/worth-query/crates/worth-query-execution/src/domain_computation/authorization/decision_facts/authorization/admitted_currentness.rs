//! Currentness of a conventional Query Ability on one carried preparation meter.

use worth_relational::facade::authorization::{
    RelationalAuthorizationBudgetedObservationStop, RelationalAuthorizationObservationAdmission,
    RelationalAuthorizationObservationFreshness,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_bridge::facade::BridgeAuthorizationRetentionStop;

use super::{bridge_matches, WorthQueryAuthorizationDecisionFact};
use crate::domain_computation::primary_graph::InvalidationEditAdmission;

struct CurrentnessAdmission<'a>(&'a mut InvalidationEditAdmission);

impl RelationalAuthorizationObservationAdmission for CurrentnessAdmission<'_> {
    type Stop = CompanionPreflightStop;

    fn prepare(&mut self, work: u64, bytes: u64) -> Result<(), Self::Stop> {
        self.0.charge_external_work(work)?;
        self.0.admit_read_scratch(bytes)
    }
}

impl WorthQueryAuthorizationDecisionFact {
    /// The selected Query permission mints a conventional fact with no
    /// delegation. If a different fact reaches this narrow route, it cannot
    /// produce a fresh verdict without that owner's admitted recursion.
    pub(in crate::domain_computation) fn remains_conventional_current_admitted(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        admission.charge_external_work(2)?;
        if self.delegation.is_some() || self.delegation_activation.is_some() {
            return Ok(false);
        }
        let bridge_current = bridge
            .retains_admitted(&self.bridge, |work, bytes| {
                admission.charge_external_work(work)?;
                admission.admit_read_scratch(bytes)
            })
            .map_err(|stop| match stop {
                BridgeAuthorizationRetentionStop::Admission(stop) => stop,
                BridgeAuthorizationRetentionStop::AccountingOverflow => {
                    CompanionPreflightStop::WorkCounterOverflow
                }
            })?;
        if !bridge_current {
            return Ok(false);
        }
        admission.charge_external_work(33)?;
        if !bridge_matches(&self.relational, &self.bridge) {
            return Ok(false);
        }
        let freshness = match runtime.compare_authorization_observation_budgeted(
            &self.relational,
            snapshot,
            &mut CurrentnessAdmission(admission),
        ) {
            Ok(freshness) => freshness,
            Err(RelationalAuthorizationBudgetedObservationStop::Admission(stop)) => {
                return Err(stop);
            }
            Err(RelationalAuthorizationBudgetedObservationStop::AccountingOverflow) => {
                return Err(CompanionPreflightStop::WorkCounterOverflow);
            }
            Err(
                RelationalAuthorizationBudgetedObservationStop::Native(_)
                | RelationalAuthorizationBudgetedObservationStop::ExactBasisRequired,
            ) => return Ok(false),
        };
        Ok(freshness == RelationalAuthorizationObservationFreshness::Fresh)
    }
}
