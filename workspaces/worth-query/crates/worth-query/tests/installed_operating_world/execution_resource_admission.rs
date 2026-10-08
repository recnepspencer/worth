use worth_query::facade::{domain, installed};

mod attempt_evidence;
mod live_saturation;
mod provider_surfaces;
mod rejection_order;
mod strategy_lattice;

const PROVIDER: &str = "resource-provider";
const ACCESS: &str = "resource-access";
const ALLOCATOR: &str = "resource-arena";

fn safe_point(name: &str) -> installed::operation::WorthQueryCancellationSafePointFamily {
    installed::operation::WorthQueryCancellationSafePointFamily::new(name).unwrap()
}

fn envelope(
    scale: u64,
    resources: u64,
    mode: installed::operation::WorthQueryExecutionMode,
    degradation: Option<installed::operation::WorthQueryExecutionDegradation>,
    safe_point: installed::operation::WorthQueryCancellationSafePointFamily,
) -> domain::WorthQueryExecutionResourceEnvelope {
    domain::WorthQueryExecutionResourceEnvelope::new(
        installed::operation::WorthQuerySemanticScaleRequest::bounded(scale),
        installed::operation::WorthQueryResourceLimitRequest::bounded(resources),
        mode,
        degradation,
        safe_point,
    )
}

fn strategy(
    name: &str,
    envelope: domain::WorthQueryExecutionResourceEnvelope,
) -> domain::WorthQueryExecutionStrategyContract {
    strategy_with_requirements(name, envelope, PROVIDER, ACCESS, ALLOCATOR)
}

fn strategy_with_requirements(
    name: &str,
    envelope: domain::WorthQueryExecutionResourceEnvelope,
    provider: &str,
    access: &str,
    allocator: &str,
) -> domain::WorthQueryExecutionStrategyContract {
    domain::WorthQueryExecutionStrategyContract::new(
        domain::WorthQueryExecutionStrategyName::new(name).unwrap(),
        envelope,
        domain::WorthQueryExecutionProviderRequirements::new(
            domain::WorthQueryExecutionProviderFamily::new(provider).unwrap(),
            domain::WorthQueryExecutionAccessProductFamily::new(access).unwrap(),
            domain::WorthQueryExecutionAllocatorFamily::new(allocator).unwrap(),
        ),
    )
}

fn contract(
    strategies: impl IntoIterator<Item = domain::WorthQueryExecutionStrategyContract>,
) -> domain::WorthQueryExecutionResourceContract {
    domain::WorthQueryExecutionResourceContract::declared(strategies).unwrap()
}

fn support(
    envelope: domain::WorthQueryExecutionResourceEnvelope,
) -> domain::WorthQueryExecutionResourceSupport {
    support_with_requirements(envelope, PROVIDER, ACCESS, ALLOCATOR)
}

fn support_with_requirements(
    envelope: domain::WorthQueryExecutionResourceEnvelope,
    provider: &str,
    access: &str,
    allocator: &str,
) -> domain::WorthQueryExecutionResourceSupport {
    let concurrent_attempt_limit =
        usize::try_from(envelope.concurrency_width_ceiling()).unwrap_or(usize::MAX);
    domain::WorthQueryExecutionResourceSupport::new(
        domain::WorthQueryExecutionProviderFamily::new(provider).unwrap(),
        domain::WorthQueryExecutionAccessProductFamily::new(access).unwrap(),
        domain::WorthQueryExecutionAllocatorFamily::new(allocator).unwrap(),
        envelope,
        std::sync::Arc::new(
            domain::WorthQueryFixedExecutionCapacity::mint(provider, concurrent_attempt_limit)
                .unwrap(),
        ),
    )
}

fn request(
    scale: installed::operation::WorthQuerySemanticScaleRequest,
    resources: installed::operation::WorthQueryResourceLimitRequest,
    safe_point: installed::operation::WorthQueryCancellationSafePointFamily,
) -> installed::operation::WorthQueryExecutionResourceRequest {
    installed::operation::WorthQueryExecutionResourceRequest::new(scale, resources, safe_point)
        .unwrap()
}

mod atomic;
