//! Retain the mutable conventional decisions after selected-root access is sealed.

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_bridge::facade::BridgeAuthorizationRuntime;

use super::WorthQueryRetainedAuthorizationDecisionFacts;
use crate::domain_computation::primary_graph::InvalidationEditAdmission;

impl WorthQueryRetainedAuthorizationDecisionFacts {
    /// The selected-access proof separately authenticates the principal on
    /// this issued root. This owner rechecks every mutable conventional
    /// authorization decision; capability facts require their own admitted
    /// recursive owner and cannot yield Current here.
    pub(in crate::domain_computation) fn conventional_decisions_current_admitted(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &BridgeAuthorizationRuntime,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        admission.charge_external_work(1)?;
        match self {
            Self::Principal(_) => Ok(true),
            Self::Abilities { decisions, .. } => {
                let visits = u64::try_from(decisions.len())
                    .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
                admission.charge_external_work(visits)?;
                for decision in decisions {
                    if !decision.remains_conventional_current_admitted(
                        runtime, snapshot, bridge, admission,
                    )? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Self::Capability(_) => Ok(false),
        }
    }
}
