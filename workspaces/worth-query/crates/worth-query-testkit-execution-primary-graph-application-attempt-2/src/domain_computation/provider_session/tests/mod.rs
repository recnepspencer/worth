use worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceAdmissionCounters;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot;
use worth_query_admission::facade::resource_admission::WorthQueryFixedExecutionCapacity;
use worth_query_admission::integration::admit_execution_resource_plan;
use worth_query_admission::integration::reserve_execution_resource_plan;
use worth_query_declaration::facade::domain_computation::WorthQueryCancellationSafePointFamily;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionMode;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest;
use worth_query_declaration::facade::domain_computation::WorthQueryPartialEffectPosture;
use worth_query_declaration::facade::domain_computation::WorthQueryResourceDimension;
use worth_query_declaration::facade::domain_computation::WorthQueryResourceLimitRequest;
use worth_query_declaration::facade::domain_computation::WorthQueryRetainedProgressPosture;
use worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleRequest;
use worth_query_declaration::facade::domain_computation::WorthQueryYieldedStatePosture;
use worth_query_installation::facade::WorthQueryExecutionAccessProductFamily;
use worth_query_installation::facade::WorthQueryExecutionAllocatorFamily;
use worth_query_installation::facade::WorthQueryExecutionProviderFamily;
use worth_query_installation::facade::WorthQueryExecutionProviderRequirements;
use worth_query_installation::facade::WorthQueryExecutionResourceContract;
use worth_query_installation::facade::WorthQueryExecutionResourceEnvelope;
use worth_query_installation::facade::WorthQueryExecutionStrategyContract;
use worth_query_installation::facade::WorthQueryExecutionStrategyName;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_execution::facade::domain_computation::WorthQueryDirectExecutionResourceAttempt;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority;
pub fn runtime() -> worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime {
    WorthQueryExecutionRuntimeInstaller::new()
        .install(
            WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .unwrap()
        .into_parts()
        .0
}
pub fn safe_point() -> WorthQueryCancellationSafePointFamily {
    WorthQueryCancellationSafePointFamily::new("execution-chunk").unwrap()
}
pub fn provider() -> WorthQueryExecutionProviderFamily {
    WorthQueryExecutionProviderFamily::new("execution-provider").unwrap()
}
pub fn access() -> WorthQueryExecutionAccessProductFamily {
    WorthQueryExecutionAccessProductFamily::new("execution-access").unwrap()
}
pub fn allocator() -> WorthQueryExecutionAllocatorFamily {
    WorthQueryExecutionAllocatorFamily::new("execution-arena").unwrap()
}
pub fn envelope(limit: u64) -> WorthQueryExecutionResourceEnvelope {
    WorthQueryExecutionResourceEnvelope::new(
        WorthQuerySemanticScaleRequest::bounded(limit),
        WorthQueryResourceLimitRequest::bounded(limit)
            .with(WorthQueryResourceDimension::RetainedBytes, 4_096),
        WorthQueryExecutionMode::Synchronous,
        None,
        safe_point(),
    )
}
pub fn admitted_plan(binding: &str, limit: u64) -> WorthQueryAdmittedExecutionResourcePlan {
    admitted_plan_with_support(binding, limit, limit, std::iter::empty())
}
pub fn admitted_yield_plan(binding: &str, limit: u64) -> WorthQueryAdmittedExecutionResourcePlan {
    let executor = execution_resource_support_with_yield(binding, limit);
    admit_plan_with_supports(binding, limit, executor, Vec::new())
}
pub fn admitted_plan_with_graph_support(
    binding: &str,
    limit: u64,
    graph_role: &str,
    graph_support: WorthQueryExecutionResourceSupport,
) -> WorthQueryAdmittedExecutionResourcePlan {
    let executor =
        execution_resource_support_for_envelope(binding, graph_support.envelope().clone());
    admit_plan_with_supports(
        binding,
        limit,
        executor,
        vec![(graph_role.to_owned(), graph_support)],
    )
}
pub fn execution_resource_support(binding: &str, limit: u64) -> WorthQueryExecutionResourceSupport {
    execution_resource_support_for_envelope(binding, envelope(limit))
}
pub fn execution_resource_support_with_partial_effects(
    binding: &str,
    limit: u64,
) -> WorthQueryExecutionResourceSupport {
    execution_resource_support_for_envelope(
        binding,
        envelope(limit)
            .with_partial_effect_posture(WorthQueryPartialEffectPosture::PartialEffectsMayRemain),
    )
}
pub fn execution_resource_support_with_yield(
    binding: &str,
    limit: u64,
) -> WorthQueryExecutionResourceSupport {
    execution_resource_support_for_envelope(
        binding,
        envelope(limit)
            .with_yielded_state_posture(WorthQueryYieldedStatePosture::ProviderCheckpoint)
            .with_retained_progress_posture(
                WorthQueryRetainedProgressPosture::RetainAttemptCapacity,
            ),
    )
}
pub fn execution_resource_support_with_yield_and_partial_effects(
    binding: &str,
    limit: u64,
) -> WorthQueryExecutionResourceSupport {
    execution_resource_support_for_envelope(
        binding,
        envelope(limit)
            .with_partial_effect_posture(WorthQueryPartialEffectPosture::PartialEffectsMayRemain)
            .with_yielded_state_posture(WorthQueryYieldedStatePosture::ProviderCheckpoint)
            .with_retained_progress_posture(
                WorthQueryRetainedProgressPosture::RetainAttemptCapacity,
            ),
    )
}
pub fn execution_resource_support_for_envelope(
    binding: &str,
    envelope: WorthQueryExecutionResourceEnvelope,
) -> WorthQueryExecutionResourceSupport {
    WorthQueryExecutionResourceSupport::new(
        provider(),
        access(),
        allocator(),
        envelope,
        std::sync::Arc::new(
            WorthQueryFixedExecutionCapacity::new(format!("provider-session-test:{binding}"), 8)
                .unwrap(),
        ),
    )
}
pub fn admitted_plan_with_support<'a>(
    binding: &str,
    limit: u64,
    support_limit: u64,
    graph_roles: impl Iterator<Item = &'a str>,
) -> WorthQueryAdmittedExecutionResourcePlan {
    let executor = execution_resource_support(binding, support_limit);
    let graph_providers = graph_roles
        .map(|role| (role.to_owned(), executor.clone()))
        .collect();
    admit_plan_with_supports(binding, limit, executor, graph_providers)
}
pub fn admit_plan_with_supports(
    binding: &str,
    limit: u64,
    executor: WorthQueryExecutionResourceSupport,
    graph_providers: Vec<(String, WorthQueryExecutionResourceSupport)>,
) -> WorthQueryAdmittedExecutionResourcePlan {
    let envelope = executor.envelope().clone();
    let contract =
        WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
            WorthQueryExecutionStrategyName::new("bounded").unwrap(),
            envelope.clone(),
            WorthQueryExecutionProviderRequirements::new(
                executor.provider().clone(),
                executor.access_product().clone(),
                executor.allocator().clone(),
            ),
        )])
        .unwrap();
    let mut request = WorthQueryExecutionResourceRequest::bounded(
        limit,
        limit,
        envelope.cancellation_safe_point().clone(),
    )
    .allow_mode(envelope.mode())
    .allow_partial_effect_posture(envelope.partial_effect_posture())
    .allow_yielded_state_posture(envelope.yielded_state_posture())
    .allow_retained_progress_posture(envelope.retained_progress_posture());
    if let Some(degradation) = envelope.degradation() {
        request = request.allow_degradation(degradation);
    }
    let support = WorthQueryExecutionResourceSupportSnapshot::new(
        executor,
        Vec::new(),
        graph_providers,
        Vec::new(),
        None,
    );

    admit_execution_resource_plan(
        binding,
        &contract,
        &request,
        support,
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap()
}
pub fn direct_attempt(binding: &str, limit: u64) -> WorthQueryDirectExecutionResourceAttempt {
    let resources = admitted_plan(binding, limit);
    let runtime = runtime();
    let authority = direct_authority(&runtime, &resources);
    let reserved = reserve_execution_resource_plan(resources).unwrap();
    WorthQueryDirectExecutionResourceAttempt::start_for_test(reserved, &authority)
}
