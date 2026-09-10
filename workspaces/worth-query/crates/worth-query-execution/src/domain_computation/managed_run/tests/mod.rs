mod artifact_release_failure;
mod authority_substitution;
pub(crate) mod causal_fixture;
mod combined_provider_capability;
mod cost_bound;
mod decision_read_set;
mod decision_read_set_admission;
mod decision_read_set_fixture;
mod direct_lifecycle;
mod direct_yield_cleanup_fixture;
mod effect_posture;
mod invariant_budget;
mod invariant_execution;
mod invariant_execution_fixture;
mod invariant_provider_failures;
mod invariant_session_affinity;
mod invariant_state_load;
mod provider_compare_and_commit;
mod provider_contract_violation;
mod provider_execution_admission;
mod provider_execution_release;
mod provider_memory_cleanup;
mod provider_session_abandonment;
mod provider_session_failure_matrix;
mod provider_session_protocol;
mod provider_session_protocol_workflow;
mod provider_session_substitution;
mod provider_support_affinity;
mod provider_work;
mod provider_work_start;
mod provisional_attempt;
mod provisional_attempt_fixture;
mod provisional_provider_failures;
mod readmission_cleanup_inspection;
pub(in crate::domain_computation::managed_run) mod readmission_direct;
mod readmission_direct_association;
mod readmission_direct_recovery;
mod readmission_owner_evidence;
mod readmission_parity;
mod readmission_preflight;
mod readmission_recovery_topology;
pub(in crate::domain_computation::managed_run) mod readmission_workflow;
mod readmission_workflow_association;
mod safe_point_observation;
mod step_cost_bound;
mod step_interruption;
mod step_output;
mod terminal_matrix;
mod variable_width_output;
mod workflow_abandonment;
mod workflow_backpressure;
mod workflow_lifecycle;
mod workflow_provider_steps;
mod workflow_step_evidence;
mod yield_abandonment;
mod yield_binding_evidence;
mod yield_bridge_failure;
mod yield_checkpoint_ceiling;
mod yield_checkpoint_fixture;
mod yield_checkpoint_release;
mod yield_cost_bound;
mod yield_eligibility_workflow;
pub(in crate::domain_computation::managed_run) mod yield_fixture;
mod yield_generation;
mod yield_lifecycle_direct;
mod yield_lifecycle_direct_signal_recovery;
mod yield_lifecycle_workflow;
mod yield_production_freeze;
mod yield_provider_artifact;
mod yield_provider_configuration;
mod yield_signal_workflow;
mod yield_workflow_recovery;

pub(super) use direct_yield_cleanup_fixture::complete_direct_yield_cleanup;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey, InternedString};
use worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan;
use worth_query_admission::integration::WorthQueryExecutionCapacityReservationScope;
use worth_query_installation::facade::{
    WorthQueryInstalledGraphParticipationAuthority, WorthQueryOperationGraphAccess,
};
use worth_runtime_bridge::facade::{BridgeExecutionBasisFinalizationFailureKind, RuntimeBridge};

use super::{
    WorthQueryDirectGraphStepOutcome, WorthQueryManagedProviderSessionDisposition,
    WorthQueryManagedRunCleanupDisposition, WorthQueryManagedRunCleanupFailureKind,
    WorthQueryManagedRunDenialKind, WorthQueryManagedRunTerminalKind,
    WorthQueryWorkflowGraphStepOutcome,
};
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionRuntime, WorthQueryExecutionRuntimeInstaller,
};
use crate::domain_computation::operation_binding::{
    direct_authority, direct_authority_with_graph, direct_authority_with_graph_and_decision_facts,
    direct_authority_with_graph_effect, direct_authority_with_graph_effect_and_decision_facts,
    direct_authority_with_graph_effect_decision_facts_and_invariants, workflow_authority,
    workflow_authority_with_output_artifact, workflow_authority_with_stage_graph,
    workflow_authority_with_stage_graph_and_output_artifact,
};
use crate::domain_computation::provider_session::{
    admitted_plan, admitted_plan_with_graph_support,
};
use crate::domain_computation::{
    WorthQueryArtifactProductionEvidence, WorthQueryArtifactProviderResource,
    WorthQueryCooperativeGraphProviderExecution, WorthQueryGraphParticipationProvider,
    WorthQueryGraphProviderCall, WorthQueryGraphProviderCallKind, WorthQueryGraphProviderExecution,
    WorthQueryGraphProviderExecutionStart, WorthQueryGraphProviderFailure,
    WorthQueryGraphProviderRestoreMemory, WorthQueryGraphProviderRetainedMemory,
    WorthQueryGraphProviderStep, WorthQueryGraphProviderStepDenialKind,
    WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderStepInvocationDisposition,
    WorthQueryGraphReadMaterial, WorthQueryGraphReadRow, WorthQueryManagedGraphCallRequest,
    WorthQueryRunningDirectRun, WorthQueryWorkflowRunCleanupOutcome,
};

fn admit_provider_execution<E: WorthQueryGraphProviderExecution>(
    start: &mut WorthQueryGraphProviderExecutionStart,
    execution: E,
) -> Result<WorthQueryCooperativeGraphProviderExecution<E>, WorthQueryGraphProviderFailure> {
    start
        .admit_cooperative_execution(|| execution)
        .map_err(|denial| WorthQueryGraphProviderFailure::new(denial.detail()))
}

fn admit_restored_provider_execution(
    memory: &mut WorthQueryGraphProviderRestoreMemory,
    execution: Box<dyn WorthQueryGraphProviderExecution>,
) -> Result<
    WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
    WorthQueryGraphProviderFailure,
> {
    memory
        .admit_cooperative_execution(|| execution)
        .map_err(|denial| WorthQueryGraphProviderFailure::new(denial.detail()))
}

fn query_runtime() -> WorthQueryExecutionRuntime {
    WorthQueryExecutionRuntimeInstaller::new()
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("managed-run runtime should install")
        .into_parts()
        .0
}

#[derive(Clone, Copy)]
struct ManagedGraph;

fn managed_graph_run_with_provider<P>(
    access: WorthQueryOperationGraphAccess,
    provider: P,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    let (running, graph, _bridge) = managed_graph_run_with_provider_and_bridge(access, provider);
    (running, graph)
}

fn managed_graph_run_with_provider_and_bridge<P>(
    access: WorthQueryOperationGraphAccess,
    provider: P,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
    RuntimeBridge,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    let (running, graph, bridge, _runtime) = managed_graph_run_with_provider_and_admitted_support(
        provider,
        ManagedGraphRunConfiguration {
            access,
            touch: false,
            binding_identity: "managed-graph-binding",
        },
        |support| support.clone(),
    );
    (running, graph, bridge)
}

fn managed_graph_run_with_provider_and_runtime<P>(
    access: WorthQueryOperationGraphAccess,
    provider: P,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    managed_graph_run_with_provider_and_runtime_binding(access, provider, "managed-graph-binding")
}

fn managed_graph_run_with_provider_and_runtime_binding<P>(
    access: WorthQueryOperationGraphAccess,
    provider: P,
    binding_identity: &str,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    managed_graph_run_with_provider_and_admitted_support(
        provider,
        ManagedGraphRunConfiguration {
            access,
            touch: false,
            binding_identity,
        },
        |support| support.clone(),
    )
}

fn managed_graph_effect_run_with_provider<P>(
    provider: P,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    let (running, graph, _, _runtime) = managed_graph_run_with_provider_and_admitted_support(
        provider,
        ManagedGraphRunConfiguration {
            access: WorthQueryOperationGraphAccess::Observe,
            touch: true,
            binding_identity: "managed-graph-binding",
        },
        |support| support.clone(),
    );
    (running, graph)
}

struct ManagedGraphRunConfiguration<'a> {
    access: WorthQueryOperationGraphAccess,
    touch: bool,
    binding_identity: &'a str,
}

fn managed_graph_run_with_provider_and_admitted_support<P>(
    provider: P,
    configuration: ManagedGraphRunConfiguration<'_>,
    admitted_support: impl FnOnce(
        &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, P>(provider),
    );
    let provider_identity = provider_anchor.provider_identity();
    let provider_support = admitted_support(provider_anchor.resource_support());
    let graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        "managed-graph",
        provider_identity,
        false,
        Option::<String>::None,
        provider_anchor,
    )
    .expect("managed graph authority should install");
    let runtime = installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("managed graph runtime should install")
        .into_parts()
        .0;
    let plan = admitted_plan_with_graph_support(
        configuration.binding_identity,
        8,
        graph.role(),
        provider_support,
    );
    let operation = if configuration.touch {
        direct_authority_with_graph_effect(&runtime, &plan, &graph)
    } else {
        direct_authority_with_graph(&runtime, &plan, &graph, configuration.access)
    };
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("managed graph operation should start");
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request())
        .expect("managed graph run should admit through lower owners")
        .start();
    (running, graph, lower.bridge, runtime)
}

fn managed_session_graph_run_with_provider<P>(
    access: WorthQueryOperationGraphAccess,
    provider: P,
    touch: bool,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>
        + crate::domain_computation::WorthQueryProviderSessionLifecycle,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install_session_capable::<ManagedGraph, P>(provider),
    );
    let provider_identity = provider_anchor.provider_identity();
    let provider_support = provider_anchor.resource_support().clone();
    let graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        "managed-graph",
        provider_identity,
        false,
        Option::<String>::None,
        provider_anchor,
    )
    .expect("session-capable graph authority should install");
    let runtime = installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("session-capable graph runtime should install")
        .into_parts()
        .0;
    let plan = admitted_plan_with_graph_support(
        "managed-graph-binding",
        8,
        graph.role(),
        provider_support,
    );
    let operation = if touch {
        direct_authority_with_graph_effect(&runtime, &plan, &graph)
    } else {
        direct_authority_with_graph(&runtime, &plan, &graph, access)
    };
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("session-capable graph attempt should start");
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request())
        .expect("session-capable managed run should admit")
        .start();
    (running, graph)
}

fn graph_material() -> WorthQueryGraphReadMaterial {
    graph_material_rows(1)
}

fn graph_material_rows(row_count: usize) -> WorthQueryGraphReadMaterial {
    let path = CanonicalFieldPath::single(FieldKey::new("id").expect("valid field key"));
    WorthQueryGraphReadMaterial::new((0..row_count).map(|index| {
        WorthQueryGraphReadRow::from_native_fields(
            format!("managed-entity-{index}"),
            [(
                path.clone(),
                AspectValue::String(InternedString::from(format!("entity-{index}"))),
            )]
            .into_iter()
            .collect(),
        )
        .expect("managed graph row should construct")
    }))
}
