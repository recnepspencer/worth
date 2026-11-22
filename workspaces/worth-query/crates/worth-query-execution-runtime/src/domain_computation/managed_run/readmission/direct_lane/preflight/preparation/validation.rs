use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeExecutionBasisReadmissionCounters, BridgeYieldedExecutionBasisPreflight, RuntimeBridge,
};

use super::{
    WorthQueryDirectProvisionalResourceAttempt, WorthQueryDirectYieldedParts,
    WorthQueryDirectYieldedState,
};
use crate::domain_computation::managed_run::{
    readmission::{
        direct_outcome::WorthQueryDirectReadmissionDenialKind,
        evidence::WorthQueryReadmissionProgress,
    },
    retained_graph_execution::WorthQueryRetainedManagedGraphExecution,
    step_contract_admission::{admit_managed_step_contract, WorthQueryAdmittedManagedStepContract},
    WorthQueryYieldedDirectRun,
};
use crate::domain_computation::provider_session::graph_provider::WorthQueryGraphProviderCallReadmissionPlan;
use crate::domain_computation::WorthQueryExecutionRuntime;

pub(super) struct WorthQueryDirectResumePreflightValidated {
    state: WorthQueryDirectYieldedState,
    bridge: BridgeYieldedExecutionBasisPreflight,
    execution: WorthQueryRetainedManagedGraphExecution,
    contract: WorthQueryAdmittedManagedStepContract,
    call: WorthQueryGraphProviderCallReadmissionPlan,
}

pub(super) struct WorthQueryDirectResumePreflightDenied {
    kind: WorthQueryDirectReadmissionDenialKind,
    detail: Arc<str>,
    yielded: WorthQueryYieldedDirectRun,
    bridge_counters: Option<BridgeExecutionBasisReadmissionCounters>,
}

pub(super) fn validate_direct_resume_preflight(
    yielded: WorthQueryYieldedDirectRun,
    query_runtime: &WorthQueryExecutionRuntime,
    bridge_runtime: &RuntimeBridge,
) -> Result<WorthQueryDirectResumePreflightValidated, WorthQueryDirectResumePreflightDenied> {
    if let Some((kind, detail)) = query_preflight_denial(&yielded, query_runtime) {
        return Err(WorthQueryDirectResumePreflightDenied::new(
            kind, detail, yielded, None,
        ));
    }
    let call = match yielded.preflight_retained_provider_call() {
        Ok(call) => call,
        Err(denial) => {
            return Err(WorthQueryDirectResumePreflightDenied::new(
                WorthQueryDirectReadmissionDenialKind::ProviderCallBindingDenied,
                format!("provider call readmission denied: {denial:?}"),
                yielded,
                None,
            ));
        }
    };
    let parts = WorthQueryDirectYieldedParts::from_yielded(yielded);
    let bridge = match bridge_runtime.preflight_yielded_execution_basis(
        parts.bridge,
        parts.state.affinity.operation_binding_identity(),
    ) {
        Ok(preflight) => preflight,
        Err(denial) => {
            let detail = Arc::from(denial.detail());
            let (bridge, bridge_counters) = denial.into_returned_yielded().into_parts();
            return Err(WorthQueryDirectResumePreflightDenied::new(
                WorthQueryDirectReadmissionDenialKind::BridgeReadmissionDenied,
                detail,
                WorthQueryDirectYieldedParts {
                    state: parts.state,
                    bridge,
                    execution: parts.execution,
                }
                .into_yielded(),
                Some(bridge_counters),
            ));
        }
    };
    let contract = match admit_managed_step_contract(
        parts.execution.contract().clone(),
        Some(bridge.step_contract()),
    ) {
        Ok(contract) => contract,
        Err(denial) => {
            let (bridge, bridge_counters) = bridge.into_returned_yielded().into_parts();
            return Err(WorthQueryDirectResumePreflightDenied::new(
                WorthQueryDirectReadmissionDenialKind::ProviderStepContractDenied(denial.kind()),
                denial.detail(),
                WorthQueryDirectYieldedParts {
                    state: parts.state,
                    bridge,
                    execution: parts.execution,
                }
                .into_yielded(),
                Some(bridge_counters),
            ));
        }
    };
    Ok(WorthQueryDirectResumePreflightValidated {
        state: parts.state,
        bridge,
        execution: parts.execution,
        contract,
        call,
    })
}

impl WorthQueryDirectResumePreflightValidated {
    pub(super) fn begin_resource_attempt(
        self,
        progress: &mut WorthQueryReadmissionProgress,
    ) -> WorthQueryDirectProvisionalResourceAttempt {
        let WorthQueryDirectYieldedState { affinity, retained } = self.state;
        let resource = affinity.begin_readmission(
            self.call,
            &crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit::mint(),
        );
        progress.minted_fresh_resource_attempt();
        WorthQueryDirectProvisionalResourceAttempt {
            state: retained,
            execution: self.execution,
            resource,
            bridge: self.bridge,
            contract: self.contract,
        }
    }
}

impl WorthQueryDirectResumePreflightDenied {
    fn new(
        kind: WorthQueryDirectReadmissionDenialKind,
        detail: impl Into<Arc<str>>,
        yielded: WorthQueryYieldedDirectRun,
        bridge_counters: Option<BridgeExecutionBasisReadmissionCounters>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            yielded,
            bridge_counters,
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        WorthQueryDirectReadmissionDenialKind,
        Arc<str>,
        WorthQueryYieldedDirectRun,
        Option<BridgeExecutionBasisReadmissionCounters>,
    ) {
        (self.kind, self.detail, self.yielded, self.bridge_counters)
    }
}

fn query_preflight_denial(
    yielded: &WorthQueryYieldedDirectRun,
    runtime: &WorthQueryExecutionRuntime,
) -> Option<(WorthQueryDirectReadmissionDenialKind, &'static str)> {
    yielded.query_readmission_denial(runtime)
}
