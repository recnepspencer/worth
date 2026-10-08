#![allow(dead_code)]

use worth_query_host::facade::installed::domain_computation as execution;
use worth_query_host::facade::{
    admission::resource_admission::{
        WorthQueryAdmittedExecutionResourcePlan, WorthQueryExecutionResourceAdmissionDenial,
    },
    declaration::domain_computation::WorthQueryExecutionResourceRequest,
    domain::{WorthQueryInstalledDomainOperationAuthority, WorthQueryPortableDomainPackage},
    installed::{
        domain_computation::{
            WorthQueryArtifactChunkRequest, WorthQueryArtifactNativeAccessCounters,
            WorthQueryArtifactNativeAccessDenial, WorthQueryDirectReadmissionCleanupOutcome,
            WorthQueryDirectReadmissionCleanupRequired,
            WorthQueryDirectReadmissionRecoveryRequired, WorthQueryDirectYieldOutcome,
            WorthQueryDirectYieldRecoveryRequired, WorthQueryPausedDirectGraphExecution,
            WorthQueryPausedWorkflowGraphExecution, WorthQueryTransferredArtifactHandle,
            WorthQueryWorkflowReadmissionCleanupOutcome,
            WorthQueryWorkflowReadmissionCleanupRequired,
            WorthQueryWorkflowReadmissionRecoveryRequired, WorthQueryWorkflowYieldOutcome,
            WorthQueryWorkflowYieldRecoveryReleaseOutcome, WorthQueryWorkflowYieldRecoveryRequired,
            WorthQueryYieldedDirectRun,
        },
        provider_session::{
            WorthQueryExecutionProviderSession, WorthQueryGraphProviderCheckpoint,
            WorthQueryGraphProviderExecutionStart, WorthQueryGraphProviderRestoreMemory,
            WorthQueryGraphProviderRetainedMemory,
        },
    },
    publication::domain_computation::WorthQueryDomainEvidenceMaterial,
    runtime::{WorthQueryExecutionRuntime, WorthQueryExecutionRuntimeInstaller},
};
use worth_query_replay::facade::WorthQueryCertificationReplayCounters;

mod phase_seven_ten_provider;

fn install_and_inspect(
    installer: WorthQueryExecutionRuntimeInstaller,
    package: WorthQueryPortableDomainPackage,
) {
    let _ = (installer, package);
}

fn phase_seven_through_ten_consumer_journey(
    running: &mut execution::WorthQueryRunningDirectRun,
    graph: &worth_query_host::facade::domain::WorthQueryInstalledGraphParticipationAuthority,
    requests: Vec<execution::WorthQueryDecisionFactRequest>,
    steps: Vec<execution::WorthQueryProvisionalEffectStep>,
    invariant_slot: &str,
    state_load: Vec<execution::WorthQueryInvariantStateLocator>,
) {
    let staged = running
        .admit_provider_execution_plan(graph)
        .unwrap()
        .readmit()
        .unwrap()
        .prepare()
        .unwrap()
        .bind_reads_and_effects();
    let read_set = {
        let reads = staged.read_authority();
        let captured = reads
            .capture_decision_read_set(
                requests,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        match reads.compare_decision_read_set(captured).unwrap() {
            execution::WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) => fresh,
            execution::WorthQueryDecisionReadSetFreshnessOutcome::Stale(_) => return,
        }
    };
    let program = staged
        .effect_authority()
        .lower_provisional_program(&read_set, steps)
        .unwrap();
    let inspection = staged
        .begin_provisional_attempt(read_set, program)
        .unwrap()
        .materialize_proposed_state()
        .inspect();
    let receipt = inspection
        .select_installed_invariant(invariant_slot)
        .unwrap()
        .admit_state_load_plan(state_load)
        .unwrap()
        .execute()
        .unwrap();
    let progression = inspection.admit_invariant_progression([receipt]).unwrap();
    let _ = progression.receipt_identities();
    inspection.discard();
}

fn production_builder_preserves_invariant_provider_capabilities<G: 'static, P>(
    builder: worth_query::facade::runtime::WorthQueryRuntimeBuilder,
    marker: G,
    provider: P,
) -> worth_query::facade::runtime::WorthQueryRuntimeBuilder
where
    P: worth_query_execution::facade::provider_session::WorthQueryGraphParticipationProvider<G>
        + worth_query_execution::facade::provider_session::WorthQueryProviderSessionLifecycle
        + worth_query_execution::facade::provider_session::WorthQueryDecisionFactProvider
        + worth_query_execution::facade::provider_session::WorthQueryProvisionalGraphProvider
        + worth_query_execution::facade::provider_session::WorthQueryInvariantExecutionProvider,
{
    builder.invariant_graph_participation_provider(marker, provider)
}

fn production_builder_preserves_atomic_invariant_provider_capabilities<G: 'static, C: 'static, P>(
    builder: worth_query::facade::runtime::WorthQueryRuntimeBuilder,
    marker: G,
    provider: P,
    commit: C,
) -> worth_query::facade::runtime::WorthQueryRuntimeBuilder
where
    P: worth_query_execution::facade::provider_session::WorthQueryGraphParticipationProvider<G>
        + worth_query_execution::facade::provider_session::WorthQueryProviderSessionLifecycle
        + worth_query_execution::facade::provider_session::WorthQueryDecisionFactProvider
        + worth_query_execution::facade::provider_session::WorthQueryProvisionalGraphProvider
        + worth_query_execution::facade::provider_session::WorthQueryInvariantExecutionProvider,
{
    builder.atomic_invariant_graph_participation_provider(marker, provider, commit)
}

fn production_builder_preserves_convergence_and_invariant_capabilities<G: 'static, P>(
    builder: worth_query::facade::runtime::WorthQueryRuntimeBuilder,
    marker: G,
    provider: P,
) -> worth_query::facade::runtime::WorthQueryRuntimeBuilder
where
    P: worth_query_execution::facade::provider_session::WorthQueryGraphParticipationProvider<G>
        + worth_query_execution::facade::provider_session::WorthQueryProviderSessionLifecycle
        + worth_query_execution::facade::provider_session::WorthQueryDecisionFactProvider
        + worth_query_execution::facade::provider_session::WorthQueryProvisionalGraphProvider
        + worth_query_execution::facade::provider_session::WorthQueryInvariantExecutionProvider
        + worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceDomainProvider,
{
    builder.convergent_invariant_graph_participation_provider(marker, provider)
}

fn production_builder_preserves_atomic_convergence_and_invariant_capabilities<
    G: 'static,
    C: 'static,
    P,
>(
    builder: worth_query::facade::runtime::WorthQueryRuntimeBuilder,
    marker: G,
    provider: P,
    commit: C,
) -> worth_query::facade::runtime::WorthQueryRuntimeBuilder
where
    P: worth_query_execution::facade::provider_session::WorthQueryGraphParticipationProvider<G>
        + worth_query_execution::facade::provider_session::WorthQueryProviderSessionLifecycle
        + worth_query_execution::facade::provider_session::WorthQueryDecisionFactProvider
        + worth_query_execution::facade::provider_session::WorthQueryProvisionalGraphProvider
        + worth_query_execution::facade::provider_session::WorthQueryInvariantExecutionProvider
        + worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceDomainProvider,
{
    builder.atomic_convergent_invariant_graph_participation_provider(marker, provider, commit)
}

fn inspect_runtime_and_installed_operation(
    runtime: &WorthQueryExecutionRuntime,
    operation: &WorthQueryInstalledDomainOperationAuthority,
) {
    let _ = runtime.authority_identity();
    let _ = runtime
        .installed_packages()
        .validate_domain_operation(operation);
}

fn inspect_resource_admission(
    plan: &WorthQueryAdmittedExecutionResourcePlan,
    denial: &WorthQueryExecutionResourceAdmissionDenial,
    session: &WorthQueryExecutionProviderSession,
) {
    let _: &WorthQueryExecutionResourceRequest = plan.request();
    let _ = plan.request_identity();
    let _ = plan.strategy();
    let _ = plan.envelope();
    let _ = denial.kind();
    let _ = session.identity();
    let _ = session.attempt_identity();
}

fn carry_artifact_and_publication(
    artifact: WorthQueryTransferredArtifactHandle,
    request: WorthQueryArtifactChunkRequest,
    counters: WorthQueryArtifactNativeAccessCounters,
    denial: WorthQueryArtifactNativeAccessDenial,
    evidence: WorthQueryDomainEvidenceMaterial,
) {
    let _ = (artifact, request, counters, denial, evidence);
}

fn yield_from_consumed_direct_safe_point(paused: WorthQueryPausedDirectGraphExecution) {
    match paused.yield_run() {
        WorthQueryDirectYieldOutcome::Yielded(yielded) => {
            let _ = yielded.inspection().checkpoint();
            let _ = yielded.cleanup();
        }
        WorthQueryDirectYieldOutcome::Denied(denied) => {
            let _ = denied.into_paused();
        }
        WorthQueryDirectYieldOutcome::RecoveryRequired(recovery) => {
            let _ = recovery.into_paused();
        }
    }
}

fn release_terminalized_direct_yield_recovery(recovery: WorthQueryDirectYieldRecoveryRequired) {
    let _ = recovery.cleanup_terminalized();
}

fn yield_from_consumed_workflow_safe_point(paused: WorthQueryPausedWorkflowGraphExecution) {
    match paused.yield_run() {
        WorthQueryWorkflowYieldOutcome::Yielded(yielded) => {
            let _ = yielded.inspection().artifact_evidence();
            let _ = yielded.cleanup();
        }
        WorthQueryWorkflowYieldOutcome::Denied(denied) => {
            let _ = denied.into_paused();
        }
        WorthQueryWorkflowYieldOutcome::RecoveryRequired(recovery) => {
            let _ = recovery.into_paused();
        }
    }
}

fn release_terminalized_workflow_yield_recovery(recovery: WorthQueryWorkflowYieldRecoveryRequired) {
    match recovery.release_terminalized() {
        Ok(outcome) => inspect_terminalized_workflow_yield_cleanup(outcome),
        Err(running) => {
            let _ = running.into_paused();
        }
    }
}

fn inspect_terminalized_workflow_yield_cleanup(
    outcome: WorthQueryWorkflowYieldRecoveryReleaseOutcome,
) {
    match outcome {
        WorthQueryWorkflowYieldRecoveryReleaseOutcome::Complete(release)
        | WorthQueryWorkflowYieldRecoveryReleaseOutcome::RecoveryRequired(release) => {
            let _ = release.inspection().artifact_evidence();
        }
        WorthQueryWorkflowYieldRecoveryReleaseOutcome::Pending(pending) => match pending.retry() {
            Ok(outcome) => inspect_terminalized_workflow_yield_cleanup(outcome),
            Err(running) => {
                let _ = running.into_paused();
            }
        },
    }
}

fn certification_entry(counters: WorthQueryCertificationReplayCounters) {
    let _ = counters;
}

fn carry_provider_authoring_contract(
    start: &mut WorthQueryGraphProviderExecutionStart,
    retained: WorthQueryGraphProviderRetainedMemory,
    restore: &mut WorthQueryGraphProviderRestoreMemory,
    checkpoint: &dyn WorthQueryGraphProviderCheckpoint,
) {
    let _ = (start, retained, restore, checkpoint);
}

fn carry_same_runtime_readmission_authority(yielded: WorthQueryYieldedDirectRun) {
    let _ = yielded;
}

fn resolve_readmission_cleanup(
    direct: WorthQueryDirectReadmissionCleanupRequired,
    workflow: WorthQueryWorkflowReadmissionCleanupRequired,
) {
    finish_direct_readmission_cleanup(direct);
    finish_workflow_readmission_cleanup(workflow);
}

fn resolve_typed_readmission_recovery(
    direct: WorthQueryDirectReadmissionRecoveryRequired,
    workflow: WorthQueryWorkflowReadmissionRecoveryRequired,
) {
    match direct {
        WorthQueryDirectReadmissionRecoveryRequired::YieldReassembly(recovery) => {
            let _ = recovery.retry_to_yielded();
        }
        WorthQueryDirectReadmissionRecoveryRequired::TerminalCleanup(recovery) => {
            finish_direct_readmission_cleanup(recovery.into_cleanup());
        }
    }
    match workflow {
        WorthQueryWorkflowReadmissionRecoveryRequired::YieldReassembly(recovery) => {
            let _ = recovery.retry_to_yielded();
        }
        WorthQueryWorkflowReadmissionRecoveryRequired::TerminalCleanup(recovery) => {
            finish_workflow_readmission_cleanup(recovery.into_cleanup());
        }
    }
}

fn finish_direct_readmission_cleanup(cleanup: WorthQueryDirectReadmissionCleanupRequired) {
    let mut outcome = cleanup.finish();
    loop {
        match outcome {
            WorthQueryDirectReadmissionCleanupOutcome::Complete(receipt)
            | WorthQueryDirectReadmissionCleanupOutcome::RecoveryRequired(receipt) => {
                let _ = receipt.inspection().readmission_evidence();
                break;
            }
            WorthQueryDirectReadmissionCleanupOutcome::Pending(pending) => {
                outcome = pending.retry();
            }
        }
    }
}

fn finish_workflow_readmission_cleanup(cleanup: WorthQueryWorkflowReadmissionCleanupRequired) {
    let mut outcome = cleanup.finish();
    loop {
        match outcome {
            WorthQueryWorkflowReadmissionCleanupOutcome::Complete(receipt)
            | WorthQueryWorkflowReadmissionCleanupOutcome::RecoveryRequired(receipt) => {
                let _ = receipt.inspection().readmission_evidence();
                break;
            }
            WorthQueryWorkflowReadmissionCleanupOutcome::Pending(pending) => {
                outcome = pending.retry();
            }
        }
    }
}

fn main() {
    phase_seven_ten_provider::exercise_combined_builder();
}
