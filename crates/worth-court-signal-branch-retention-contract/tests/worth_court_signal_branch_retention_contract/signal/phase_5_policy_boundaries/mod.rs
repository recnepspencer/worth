pub(crate) use {
    worth_signal::facade::compile_signal_runtime_policy,
    worth_signal::facade::InstalledSignalRuntimePolicy,
    worth_signal::facade::ParallelAdmissionPolicy,
};

#[test]
fn public_facade_compiles_the_policy_progression() {
    let request = worth_signal::facade::SignalRuntimePolicyRequest::new(
        worth_signal::facade::SignalRuntimePolicy::operational().with_observation_activation(
            worth_foundational::ObservationActivationProfile::Continuous,
        ),
    );
    let installed: InstalledSignalRuntimePolicy =
        compile_signal_runtime_policy(request).expect("public policy request should compile");
    assert_eq!(
        installed.execution_objective(),
        worth_foundational::ExecutionObjectiveProfile::Throughput
    );
    assert_eq!(
        installed.observation_activation(),
        worth_foundational::ObservationActivationProfile::Continuous
    );
}

#[test]
fn public_runtime_policy_mutation_returns_typed_denial_without_mutation() {
    let mut runtime =
        worth_signal::facade::SignalRuntime::builder(worth_signal::facade::SignalGraph::new())
            .with_kernel_defaults()
            .build_validated()
            .expect("default runtime should build");
    let before = runtime.runtime_policy();
    let invalid = worth_signal::facade::SignalRuntimePolicy::development().with_parallel_admission(
        ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 0,
            balanced_min_parallel_tasks: 0,
            latency_bounded_min_parallel_tasks: 0,
            full_parallel_min_tasks: 0,
        },
    );

    let denial = runtime
        .try_set_runtime_policy(invalid)
        .expect_err("zero history must be rejected through the public mutation path");
    let _ = denial;
    assert_eq!(runtime.runtime_policy(), before);
}

#[test]
fn public_runtime_policy_adjustment_returns_typed_denial_without_mutation() {
    let mut runtime =
        worth_signal::facade::SignalRuntime::builder(worth_signal::facade::SignalGraph::new())
            .with_kernel_defaults()
            .build_validated()
            .expect("default runtime should build");
    let before = runtime.runtime_policy();

    let denial = runtime
        .try_adjust_runtime_policy(|policy| {
            policy.with_parallel_admission(ParallelAdmissionPolicy {
                throughput_min_parallel_tasks: 0,
                balanced_min_parallel_tasks: 0,
                latency_bounded_min_parallel_tasks: 0,
                full_parallel_min_tasks: 0,
            })
        })
        .expect_err("invalid adjusted policy should return typed denial");
    let _ = denial;
    assert_eq!(runtime.runtime_policy(), before);
}
