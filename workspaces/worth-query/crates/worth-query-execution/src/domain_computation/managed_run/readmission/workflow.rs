#![deny(private_interfaces)]

use std::sync::Arc;

use worth_runtime_bridge::facade::RuntimeBridge;

use self::workflow_preflight::validate_workflow_resume_preflight;
use self::workflow_state::WorthQueryWorkflowProvisionalAssociation;
use super::super::step_contract_admission::WorthQueryAdmittedManagedStepContract;
use super::super::WorthQueryYieldedWorkflowRun;
use super::evidence::WorthQueryReadmissionProgress;
use super::workflow_outcome::{
    WorthQueryWorkflowReadmissionDenialKind, WorthQueryWorkflowReadmissionDenied,
    WorthQueryWorkflowReadmissionOutcome,
};
use crate::domain_computation::WorthQueryExecutionRuntime;

#[path = "workflow_abort.rs"]
mod workflow_abort;
#[path = "workflow_completion.rs"]
mod workflow_completion;
#[path = "workflow_preflight.rs"]
mod workflow_preflight;
#[path = "workflow_state.rs"]
mod workflow_state;

pub(in crate::domain_computation::managed_run::readmission) use workflow_state::workflow_cleanup_owner;
pub(in crate::domain_computation::managed_run) use workflow_state::WorthQueryWorkflowReadmissionCommitState;
pub(in crate::domain_computation::managed_run) use workflow_state::WorthQueryWorkflowReadmissionRestoreMint;
pub(in crate::domain_computation::managed_run) use workflow_state::WorthQueryWorkflowYieldRestoredOwner;
pub(in crate::domain_computation::managed_run) use workflow_state::WorthQueryWorkflowYieldedAssociation;
pub(super) use workflow_state::{
    WorthQueryWorkflowBridgePendingAssociation, WorthQueryWorkflowBridgeRecoveryAssociation,
    WorthQueryWorkflowRestoredAssociation,
};

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowReadmissionProgressionPermit
{
    _owner: (),
}

impl WorthQueryWorkflowReadmissionProgressionPermit {
    fn mint() -> Self {
        Self { _owner: () }
    }
}

struct WorthQueryWorkflowProvisionalResourceAttempt {
    association: WorthQueryWorkflowProvisionalAssociation,
    contract: WorthQueryAdmittedManagedStepContract,
    stage_identity: String,
    progress: WorthQueryReadmissionProgress,
}

struct WorthQueryWorkflowBridgeReadmissionPending {
    association: WorthQueryWorkflowBridgePendingAssociation,
    contract: WorthQueryAdmittedManagedStepContract,
    stage_identity: String,
    progress: WorthQueryReadmissionProgress,
}

pub(in crate::domain_computation::managed_run) fn readmit_workflow(
    active_request: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    yielded: WorthQueryYieldedWorkflowRun,
    query_runtime: &WorthQueryExecutionRuntime,
    bridge_runtime: &RuntimeBridge,
) -> WorthQueryWorkflowReadmissionOutcome {
    let active_request = match yielded.request_for_phase(active_request) {
        Ok(request) => request,
        Err(_) => {
            return denied(
                WorthQueryWorkflowReadmissionDenialKind::ForeignAdvancementPhase,
                "the phase belongs to another installed runtime",
                yielded,
                WorthQueryReadmissionProgress::default(),
            )
        }
    };
    let owner = WorthQueryWorkflowReadmissionProgressionPermit::mint();
    let mut progress = WorthQueryReadmissionProgress::default();
    progress.checked_preflight();
    let preflight =
        match validate_workflow_resume_preflight(yielded, query_runtime, bridge_runtime, &owner) {
            Ok(preflight) => preflight,
            Err(denial) => return denial.into_outcome(progress),
        };
    let provisional = preflight.owner_begin_resource(progress, &owner);
    let pending = match begin_bridge_readmission(provisional, bridge_runtime, &owner) {
        Ok(pending) => pending,
        Err(outcome) => return outcome,
    };
    restore_workflow(active_request, pending, bridge_runtime, &owner)
}

fn begin_bridge_readmission(
    provisional: WorthQueryWorkflowProvisionalResourceAttempt,
    bridge_runtime: &RuntimeBridge,
    owner: &WorthQueryWorkflowReadmissionProgressionPermit,
) -> Result<WorthQueryWorkflowBridgeReadmissionPending, WorthQueryWorkflowReadmissionOutcome> {
    let WorthQueryWorkflowProvisionalResourceAttempt {
        association,
        contract,
        stage_identity,
        mut progress,
    } = provisional;
    progress.attempted_bridge_readmission();
    association
        .owner_readmit_bridge(bridge_runtime, owner)
        .owner_resolve(contract, stage_identity, progress, owner)
}

fn restore_workflow(
    active_request: worth_execution::ExecutionRequest<'_, '_>,

    pending: WorthQueryWorkflowBridgeReadmissionPending,
    bridge_runtime: &RuntimeBridge,
    owner: &WorthQueryWorkflowReadmissionProgressionPermit,
) -> WorthQueryWorkflowReadmissionOutcome {
    let WorthQueryWorkflowBridgeReadmissionPending {
        association,
        contract,
        stage_identity,
        mut progress,
    } = pending;
    progress.attempted_provider_restore();
    association
        .owner_restore_provider(active_request, contract, owner)
        .owner_resolve(stage_identity, bridge_runtime, progress, owner)
}

fn denied(
    kind: WorthQueryWorkflowReadmissionDenialKind,
    detail: impl Into<Arc<str>>,
    yielded: WorthQueryYieldedWorkflowRun,
    progress: WorthQueryReadmissionProgress,
) -> WorthQueryWorkflowReadmissionOutcome {
    WorthQueryWorkflowReadmissionOutcome::Denied(WorthQueryWorkflowReadmissionDenied::new(
        kind,
        detail,
        yielded,
        progress.evidence(),
    ))
}
