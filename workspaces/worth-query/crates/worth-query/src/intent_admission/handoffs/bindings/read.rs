use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::query_context::ScopedQueryBasisContext;
use crate::runtime::{
    WorthQueryAdmittedGraphReadAccessPlan, WorthQueryLiveGraphReadAccessPlan, WorthQueryReadFamily,
    WorthQueryRuntimeLiveSubscriptionInstallation,
};

use super::handoff_execution_binding_identity;
use crate::intent_admission::{
    WorthQueryIntentAdmissionCoveredEntrypoint, WorthQueryIntentAdmissionExecutionSeam,
    WorthQueryIntentAdmissionFamily, WorthQueryLiveReadExecutionHandoff,
    WorthQueryReadExecutionHandoff,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryReadExecutionBinding {
    handoff: WorthQueryReadExecutionHandoff,
    graph_read_access_plan: WorthQueryAdmittedGraphReadAccessPlan,
    binding_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryLiveReadExecutionBinding {
    handoff: WorthQueryLiveReadExecutionHandoff,
    live_graph_read_access_plan: WorthQueryLiveGraphReadAccessPlan,
    binding_digest: String,
}

impl WorthQueryReadExecutionBinding {
    pub(crate) fn from_handoff(
        handoff: WorthQueryReadExecutionHandoff,
        graph_read_access_plan: WorthQueryAdmittedGraphReadAccessPlan,
    ) -> Result<Self, IdentityDerivationDenial> {
        let binding_digest = handoff_execution_binding_identity(
            "read-execution",
            &format!(
                "{}:{}",
                handoff.handoff_digest(),
                graph_read_access_plan.digest()
            ),
        )?;
        Ok(Self {
            handoff,
            graph_read_access_plan,
            binding_digest,
        })
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.handoff.family()
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.handoff.entrypoint()
    }

    pub fn execution_seam(&self) -> WorthQueryIntentAdmissionExecutionSeam {
        self.handoff.execution_seam()
    }

    pub fn read_family(&self) -> &WorthQueryReadFamily {
        self.handoff.read_family()
    }

    pub fn basis_context(&self) -> Option<&ScopedQueryBasisContext> {
        self.handoff.basis_context()
    }

    pub fn handoff(&self) -> &WorthQueryReadExecutionHandoff {
        &self.handoff
    }

    pub fn graph_read_access_plan(&self) -> &WorthQueryAdmittedGraphReadAccessPlan {
        &self.graph_read_access_plan
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }
}

impl WorthQueryLiveReadExecutionBinding {
    pub(crate) fn from_handoff(
        handoff: WorthQueryLiveReadExecutionHandoff,
        live_graph_read_access_plan: WorthQueryLiveGraphReadAccessPlan,
    ) -> Result<Self, IdentityDerivationDenial> {
        let binding_digest = handoff_execution_binding_identity(
            "live-read-execution",
            &format!(
                "{}:{}",
                handoff.handoff_digest(),
                live_graph_read_access_plan.digest()
            ),
        )?;
        Ok(Self {
            handoff,
            live_graph_read_access_plan,
            binding_digest,
        })
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.handoff.family()
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.handoff.entrypoint()
    }

    pub fn execution_seam(&self) -> WorthQueryIntentAdmissionExecutionSeam {
        self.handoff.execution_seam()
    }

    pub fn installation(&self) -> &WorthQueryRuntimeLiveSubscriptionInstallation {
        self.handoff.installation()
    }

    pub fn handoff(&self) -> &WorthQueryLiveReadExecutionHandoff {
        &self.handoff
    }

    pub fn live_graph_read_access_plan(&self) -> &WorthQueryLiveGraphReadAccessPlan {
        &self.live_graph_read_access_plan
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }
}
