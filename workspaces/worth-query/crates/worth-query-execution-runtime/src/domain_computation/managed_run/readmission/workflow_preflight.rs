use std::sync::Arc;

use worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan;
use worth_runtime_bridge::facade::{BridgeExecutionBasisReadmissionCounters, RuntimeBridge};

use super::super::super::step_contract_admission::{
    admit_managed_step_contract, WorthQueryAdmittedManagedStepContract,
};
use super::super::super::WorthQueryYieldedWorkflowRun;
use super::super::workflow_outcome::WorthQueryWorkflowReadmissionDenialKind;
use super::workflow_state::WorthQueryWorkflowPreflightAssociation;
use super::WorthQueryWorkflowReadmissionProgressionPermit;
use crate::domain_computation::provider_session::graph_provider::WorthQueryGraphProviderCallReadmissionPlan;
use crate::domain_computation::WorthQueryExecutionRuntime;

pub(super) struct WorthQueryWorkflowResumePreflightValidated {
    association: WorthQueryWorkflowPreflightAssociation,
    contract: WorthQueryAdmittedManagedStepContract,
    call: WorthQueryGraphProviderCallReadmissionPlan,
    stage_resources: Arc<WorthQueryAdmittedExecutionResourcePlan>,
    stage_identity: String,
}

pub(super) struct WorthQueryWorkflowResumePreflightDenied {
    kind: WorthQueryWorkflowReadmissionDenialKind,
    detail: Arc<str>,
    yielded: WorthQueryYieldedWorkflowRun,
    bridge_counters: Option<BridgeExecutionBasisReadmissionCounters>,
}

pub(super) fn validate_workflow_resume_preflight(
    yielded: WorthQueryYieldedWorkflowRun,
    query_runtime: &WorthQueryExecutionRuntime,
    bridge_runtime: &RuntimeBridge,
    owner: &WorthQueryWorkflowReadmissionProgressionPermit,
) -> Result<WorthQueryWorkflowResumePreflightValidated, WorthQueryWorkflowResumePreflightDenied> {
    if let Some((kind, detail)) = yielded.query_readmission_denial(query_runtime) {
        return Err(WorthQueryWorkflowResumePreflightDenied::new(
            kind, detail, yielded, None,
        ));
    }
    let Some(stage_identity) = yielded.readmission_stage_identity().map(str::to_owned) else {
        return Err(WorthQueryWorkflowResumePreflightDenied::new(
            WorthQueryWorkflowReadmissionDenialKind::WorkflowStageResourcesUnavailable,
            "retained workflow provider call has no stage identity",
            yielded,
            None,
        ));
    };
    let Some((stage_resources, stage_evidence)) =
        yielded.readmission_stage_resources(&stage_identity)
    else {
        return Err(WorthQueryWorkflowResumePreflightDenied::new(
            WorthQueryWorkflowReadmissionDenialKind::WorkflowStageResourcesUnavailable,
            "yielded workflow attempt has no resources for the retained stage",
            yielded,
            None,
        ));
    };
    let call = match yielded.preflight_retained_provider_call(&stage_evidence) {
        Ok(call) => call,
        Err(denial) => {
            return Err(WorthQueryWorkflowResumePreflightDenied::new(
                WorthQueryWorkflowReadmissionDenialKind::ProviderCallBindingDenied,
                format!("workflow provider call readmission denied: {denial:?}"),
                yielded,
                None,
            ));
        }
    };
    let association = yielded.owner_begin_readmission(owner);
    let association = association
        .owner_preflight(bridge_runtime, owner)
        .owner_resolve()?;
    let contract = match admit_managed_step_contract(
        association.execution_contract(),
        Some(association.step_contract()),
    ) {
        Ok(contract) => contract,
        Err(denial) => {
            let (yielded, bridge_counters) = association.owner_abort(owner);
            return Err(WorthQueryWorkflowResumePreflightDenied::new(
                WorthQueryWorkflowReadmissionDenialKind::ProviderStepContractDenied(denial.kind()),
                denial.detail(),
                yielded,
                Some(bridge_counters),
            ));
        }
    };
    Ok(WorthQueryWorkflowResumePreflightValidated {
        association,
        contract,
        call,
        stage_resources,
        stage_identity,
    })
}

impl WorthQueryWorkflowResumePreflightValidated {
    pub(super) fn owner_begin_resource(
        self,
        mut progress: super::super::evidence::WorthQueryReadmissionProgress,
        owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> super::WorthQueryWorkflowProvisionalResourceAttempt {
        let association =
            self.association
                .owner_begin_resource(self.stage_resources, self.call, owner);
        progress.minted_fresh_resource_attempt();
        super::WorthQueryWorkflowProvisionalResourceAttempt {
            association,
            contract: self.contract,
            stage_identity: self.stage_identity,
            progress,
        }
    }
}

impl WorthQueryWorkflowResumePreflightDenied {
    pub(super) fn new(
        kind: WorthQueryWorkflowReadmissionDenialKind,
        detail: impl Into<Arc<str>>,
        yielded: WorthQueryYieldedWorkflowRun,
        bridge_counters: Option<BridgeExecutionBasisReadmissionCounters>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            yielded,
            bridge_counters,
        }
    }

    pub(super) fn into_outcome(
        self,
        mut progress: super::super::evidence::WorthQueryReadmissionProgress,
    ) -> super::super::workflow_outcome::WorthQueryWorkflowReadmissionOutcome {
        if let Some(counters) = self.bridge_counters {
            progress.observe_bridge(counters);
        }
        super::denied(self.kind, self.detail, self.yielded, progress)
    }
}
