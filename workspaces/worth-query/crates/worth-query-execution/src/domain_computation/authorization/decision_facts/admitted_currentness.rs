//! Retain the mutable conventional decisions after selected-root access is sealed.

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_bridge::facade::BridgeAuthorizationRuntime;

use super::WorthQueryRetainedAuthorizationDecisionFacts;
use crate::domain_computation::primary_graph::InvalidationEditAdmission;

pub(in crate::domain_computation) enum AuthorizationCurrentnessStop {
    Admission(CompanionPreflightStop),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
}
impl From<CompanionPreflightStop> for AuthorizationCurrentnessStop {
    fn from(stop: CompanionPreflightStop) -> Self {
        Self::Admission(stop)
    }
}

impl WorthQueryRetainedAuthorizationDecisionFacts {
    pub(in crate::domain_computation) fn source_query_principal_facts(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
        CompanionPreflightStop,
    > {
        super::principal(self).source_query_facts(admission)
    }

    /// The selected-access proof separately authenticates the principal on
    /// this issued root. This owner rechecks every mutable conventional
    /// authorization decision; capability facts require their own admitted
    /// recursive owner and cannot yield Current here.
    pub(in crate::domain_computation) fn conventional_decisions_current_admitted(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &BridgeAuthorizationRuntime,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, AuthorizationCurrentnessStop> {
        admission.charge_external_work(1)?;
        match self {
            Self::Principal(_) => Ok(true),
            Self::Abilities { decisions, .. } => {
                let visits = u64::try_from(decisions.len())
                    .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
                admission.charge_external_work(visits)?;
                for decision in decisions {
                    if !decision.remains_conventional_current_admitted(
                        execution, runtime, snapshot, bridge, admission,
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
