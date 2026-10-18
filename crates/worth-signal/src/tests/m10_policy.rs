use crate::data::performance::{ResolvedExecutionStrategy, ResolvedMaintenanceStrategy};
use crate::facade::{ObservationLevel, SignalGraph};
use crate::runtime_policy::{
    compile_signal_runtime_policy, SignalRuntimePolicy, SignalRuntimePolicyRequest,
};
use worth_foundational::{ExecutionObjectiveProfile, ObservationActivationProfile};

#[test]
fn operational_installs_throughput_on_demand_objective() {
    let policy = SignalRuntimePolicy::operational();
    assert_eq!(
        policy.execution_objective(),
        ExecutionObjectiveProfile::Throughput
    );
    assert_eq!(
        policy.observation_activation(),
        ObservationActivationProfile::OnDemand
    );

    let installed = compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(policy))
        .expect("operational request should compile");
    assert_eq!(
        installed.execution_strategy(),
        ResolvedExecutionStrategy::SparseIncremental
    );
    assert_eq!(
        installed.maintenance_strategy(),
        ResolvedMaintenanceStrategy::DensityAdaptive
    );
}

#[test]
fn operational_continuous_observation_is_admitted_as_an_independent_axis() {
    let policy = SignalRuntimePolicy::operational()
        .with_observation_activation(ObservationActivationProfile::Continuous);
    compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(policy))
        .expect("objective and activation are independent axes");
}

#[test]
fn objective_and_activation_form_independent_compiler_axes() {
    for objective in [
        ExecutionObjectiveProfile::Throughput,
        ExecutionObjectiveProfile::Balanced,
        ExecutionObjectiveProfile::LatencyBounded,
    ] {
        for activation in [
            ObservationActivationProfile::OnDemand,
            ObservationActivationProfile::Continuous,
        ] {
            let policy = SignalRuntimePolicy::development()
                .with_execution_objective(objective)
                .with_observation_activation(activation);
            let installed = compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(policy))
                .expect("all objective/activation combinations should compile");
            assert_eq!(installed.resolved().execution_objective(), objective);
            assert_eq!(installed.resolved().observation_activation(), activation);
        }
    }
}

#[test]
fn accepted_orthogonal_request_replaces_the_graph_installation() {
    let mut graph = SignalGraph::new();
    let before = graph.runtime_policy();
    let accepted = SignalRuntimePolicy::operational()
        .with_observation_activation(ObservationActivationProfile::Continuous);
    graph
        .try_set_runtime_policy(accepted)
        .expect("orthogonal request should install");
    assert_ne!(graph.runtime_policy(), before);
}

#[test]
fn graph_strategy_reads_installed_activation_not_a_caller_tier() {
    let mut graph = SignalGraph::new();
    graph
        .try_set_runtime_policy(SignalRuntimePolicy::operational())
        .expect("operational policy should install");
    assert_eq!(
        graph.derive_evaluation_strategy().observation_level,
        ObservationLevel::Minimal
    );
    assert_eq!(
        graph.runtime_policy().observation_activation(),
        ObservationActivationProfile::OnDemand
    );
}

#[test]
fn foundational_profile_preserves_objective_and_activation() {
    let policy = SignalRuntimePolicy::operational()
        .with_observation_activation(ObservationActivationProfile::Continuous);
    let profile = policy.foundational_profile();
    assert_eq!(
        profile.execution_objective(),
        ExecutionObjectiveProfile::Throughput
    );
    assert_eq!(
        profile.observation_activation(),
        ObservationActivationProfile::Continuous
    );
}

#[test]
fn parallel_thresholds_are_objective_resolved_not_diagnostic_richness_resolved() {
    let throughput = SignalRuntimePolicy::operational();
    let diagnostic_richness_only = SignalRuntimePolicy {
        tier: crate::diagnostics::profile::DiagnosticsTier::Forensic,
        retention_budget: SignalRuntimePolicy::forensic().retention_budget,
        ..throughput
    };
    let throughput_installed =
        compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(throughput))
            .expect("operational policy should compile");
    let rich_installed =
        compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(diagnostic_richness_only))
            .expect("richness-only policy should compile");
    assert_eq!(
        throughput_installed.parallel_min_tasks(),
        rich_installed.parallel_min_tasks()
    );
    assert_eq!(
        throughput_installed.full_parallel_min_tasks(),
        rich_installed.full_parallel_min_tasks()
    );
}

#[test]
fn diagnostics_request_mirror_cannot_change_installed_runtime_decisions() {
    let mut graph = SignalGraph::new();
    graph
        .try_set_runtime_policy(SignalRuntimePolicy::operational())
        .expect("operational policy should install");
    let installed_before = graph.installed_runtime_policy();
    let strategy_before = graph.derive_evaluation_strategy();
    let mirror_mutation = SignalRuntimePolicy::forensic()
        .with_execution_objective(ExecutionObjectiveProfile::LatencyBounded)
        .with_observation_activation(ObservationActivationProfile::Continuous);
    graph
        .diagnostics_state_mut()
        .set_request_mirror(mirror_mutation);
    let installed_after = graph.installed_runtime_policy();
    let strategy_after = graph.derive_evaluation_strategy();
    assert_eq!(installed_after, installed_before);
    assert_eq!(
        strategy_after, strategy_before,
        "the consequential strategy must remain installed-policy-derived"
    );
    assert_eq!(
        installed_after.execution_strategy(),
        installed_before.execution_strategy()
    );
    assert_eq!(
        installed_after.retention_budget(),
        installed_before.retention_budget()
    );
}

#[test]
fn changing_only_diagnostics_tier_does_not_change_compiled_strategy() {
    let operational = SignalRuntimePolicy::operational();
    let forensic = SignalRuntimePolicy {
        tier: crate::diagnostics::profile::DiagnosticsTier::Forensic,
        ..operational
    };
    let first = compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(operational))
        .expect("operational policy should compile");
    let second = compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(forensic))
        .expect("tier-only policy should compile");
    assert_ne!(operational.tier, forensic.tier);
    assert_eq!(first.execution_strategy(), second.execution_strategy());
    assert_eq!(first.maintenance_strategy(), second.maintenance_strategy());
}

#[test]
fn runtime_builder_installs_compiled_policy_and_accepts_orthogonal_request() {
    let runtime = crate::facade::SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .runtime_policy(SignalRuntimePolicy::operational())
        .build_validated()
        .expect("valid policy should build");
    assert_eq!(
        runtime.observe().evaluation_strategy().observation_level,
        ObservationLevel::Minimal
    );

    let accepted = crate::facade::SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .runtime_policy(
            SignalRuntimePolicy::operational()
                .with_observation_activation(ObservationActivationProfile::Continuous),
        )
        .build_validated();
    assert!(accepted.is_ok());

    let invalid = crate::facade::SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .runtime_policy(SignalRuntimePolicy::development().with_parallel_admission(
            crate::runtime_policy::ParallelAdmissionPolicy {
                throughput_min_parallel_tasks: 0,
                balanced_min_parallel_tasks: 0,
                latency_bounded_min_parallel_tasks: 0,
                full_parallel_min_tasks: 0,
            },
        ))
        .build_validated();
    assert!(invalid.is_err());
}

#[test]
fn runtime_reconfiguration_rejects_invalid_request_without_mutating_installed_policy() {
    let mut runtime = crate::facade::SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .runtime_policy(SignalRuntimePolicy::operational())
        .build_validated()
        .expect("operational policy should build");
    runtime.set_runtime_policy(SignalRuntimePolicy::operational());
    assert_eq!(
        runtime.observe().evaluation_strategy().observation_level,
        ObservationLevel::Minimal
    );
    let invalid = SignalRuntimePolicy::development().with_parallel_admission(
        crate::runtime_policy::ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 0,
            balanced_min_parallel_tasks: 0,
            latency_bounded_min_parallel_tasks: 0,
            full_parallel_min_tasks: 0,
        },
    );
    runtime
        .graph_mut()
        .try_set_runtime_policy(invalid)
        .expect_err("invalid request should be rejected");
    assert_eq!(
        runtime.runtime_policy().execution_objective(),
        ExecutionObjectiveProfile::Throughput
    );
}

#[test]
fn installed_policy_deserialization_reenters_compiler_admission() {
    let installed = compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(
        SignalRuntimePolicy::operational(),
    ))
    .expect("baseline policy should compile");
    let roundtrip = serde_json::from_value::<crate::runtime_policy::InstalledSignalRuntimePolicy>(
        serde_json::to_value(installed).expect("installed policy should serialize"),
    )
    .expect("compiler-produced installed authority should round-trip");
    assert_eq!(roundtrip, installed);
    let mut encoded = serde_json::to_value(installed).expect("installed policy should serialize");
    encoded["requested_policy"]["parallel_admission"]["throughput_min_parallel_tasks"] =
        serde_json::Value::from(0_u64);
    assert!(
        serde_json::from_value::<crate::runtime_policy::InstalledSignalRuntimePolicy>(encoded)
            .is_err()
    );

    let mut resolved_tamper = serde_json::to_value(installed).expect("policy should serialize");
    resolved_tamper["resolved"]["execution_objective"] =
        serde_json::Value::String("LatencyBounded".to_string());
    assert!(
        serde_json::from_value::<crate::runtime_policy::InstalledSignalRuntimePolicy>(
            resolved_tamper
        )
        .is_err(),
        "serialized resolved authority must be validated, not ignored"
    );

    let mut request_tamper = serde_json::to_value(installed).expect("policy should serialize");
    request_tamper["requested_policy"]["execution_objective"] =
        serde_json::Value::String("LatencyBounded".to_string());
    assert!(
        serde_json::from_value::<crate::runtime_policy::InstalledSignalRuntimePolicy>(
            request_tamper
        )
        .is_err(),
        "serialized request and resolved authority must agree"
    );
}
