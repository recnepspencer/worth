use super::capacity_diagnostics::capacity_mismatch_detail;
use worth_query_declaration::facade::domain_computation::{
    WorthQueryExecutionMode, WorthQueryExecutionResourceRequest,
};
use worth_query_installation::facade::{
    WorthQueryExecutionResourceContract, WorthQueryExecutionStrategyContract,
};

use super::{
    WorthQueryAdmittedExecutionResourcePlan, WorthQueryExecutionResourceAdmissionCounters,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenialKind as Kind,
    WorthQueryExecutionResourceSupportSnapshot,
};

pub fn admit_execution_resource_plan(
    binding_identity: &str,
    contract: &WorthQueryExecutionResourceContract,
    request: &WorthQueryExecutionResourceRequest,
    support: WorthQueryExecutionResourceSupportSnapshot,
    counters: WorthQueryExecutionResourceAdmissionCounters,
) -> Result<WorthQueryAdmittedExecutionResourcePlan, WorthQueryExecutionResourceAdmissionDenial> {
    let (prepared, counters) =
        prepare_execution_resource_plan(contract, request, support, counters)?;
    Ok(prepared.bind(binding_identity, counters))
}

pub(crate) fn prepare_execution_resource_plan(
    contract: &WorthQueryExecutionResourceContract,
    request: &WorthQueryExecutionResourceRequest,
    support: WorthQueryExecutionResourceSupportSnapshot,
    mut counters: WorthQueryExecutionResourceAdmissionCounters,
) -> Result<
    (
        super::PreparedExecutionResourcePlan,
        WorthQueryExecutionResourceAdmissionCounters,
    ),
    WorthQueryExecutionResourceAdmissionDenial,
> {
    counters.resource_contract_lookups += 1;
    contract.validate().map_err(|detail| {
        WorthQueryExecutionResourceAdmissionDenial::new(Kind::ResourceContract, detail, counters)
    })?;
    request.validate().map_err(|detail| {
        WorthQueryExecutionResourceAdmissionDenial::new(Kind::ResourceContract, detail, counters)
    })?;
    let mut fitting = Vec::new();
    for strategy in contract.strategies() {
        counters.strategy_checks += 1;
        counters.envelope_dimension_checks +=
            request.scale().iter().count() + request.limits().iter().count();
        if strategy.envelope().admits(request) {
            fitting.push(strategy);
        }
    }
    fitting.sort_by_key(|strategy| strategy.envelope().degradation().is_some());
    for strategy in &fitting {
        counters.support_snapshot_checks += 1;
        if support.supports(strategy) {
            let prepared = super::PreparedExecutionResourcePlan::new(
                contract.canonical_identity(),
                request,
                support,
                (*strategy).clone(),
            );
            return Ok((prepared, counters));
        }
    }
    if let Some(strategy) = fitting.first() {
        return Err(support_mismatch(strategy, &support, counters));
    }
    Err(classify_request_mismatch(contract, request, counters))
}

fn support_mismatch(
    strategy: &WorthQueryExecutionStrategyContract,
    support: &WorthQueryExecutionResourceSupportSnapshot,
    counters: WorthQueryExecutionResourceAdmissionCounters,
) -> WorthQueryExecutionResourceAdmissionDenial {
    let required = strategy.provider_requirements();
    let Some((subject, actual)) = support.first_mismatch(strategy) else {
        return WorthQueryExecutionResourceAdmissionDenial::new(
            Kind::Backpressured,
            "resource support changed after strategy evaluation",
            counters,
        );
    };
    let (kind, detail) = if actual.provider() != required.provider() {
        (
            Kind::DifferentProviderRequired,
            format!("{subject} requires a different provider family"),
        )
    } else if actual.access_product() != required.access_product() {
        (
            Kind::DifferentAccessProductRequired,
            format!("{subject} requires a different access-product family"),
        )
    } else if actual.allocator() != required.allocator() {
        (
            Kind::DifferentAllocatorRequired,
            format!("{subject} requires a different allocator family"),
        )
    } else if actual.envelope().boundary() != strategy.envelope().boundary() {
        (
            Kind::ExecutionBoundaryUnsupported,
            format!("{subject} does not support the strategy execution boundary"),
        )
    } else if actual.envelope().mode() != strategy.envelope().mode() {
        (
            Kind::ExecutionModeUnsupported,
            format!("{subject} does not support the strategy execution mode"),
        )
    } else if actual.envelope().cancellation_safe_point()
        != strategy.envelope().cancellation_safe_point()
    {
        (
            Kind::CancellationSafePointUnsupported,
            format!("{subject} does not support the strategy cancellation safe point"),
        )
    } else if actual.envelope().degradation() != strategy.envelope().degradation() {
        (
            Kind::DegradationPostureUnsupported,
            format!("{subject} does not support the strategy degradation posture"),
        )
    } else if actual.envelope().partial_effect_posture()
        != strategy.envelope().partial_effect_posture()
    {
        (
            Kind::PartialEffectPostureUnsupported,
            format!("{subject} does not support the strategy partial-effect posture"),
        )
    } else if actual.envelope().yielded_state_posture()
        != strategy.envelope().yielded_state_posture()
    {
        (
            Kind::YieldedStatePostureUnsupported,
            format!("{subject} does not support the strategy yielded-state posture"),
        )
    } else if actual.envelope().retained_progress_posture()
        != strategy.envelope().retained_progress_posture()
    {
        (
            Kind::RetainedProgressPostureUnsupported,
            format!("{subject} does not support the strategy retained-progress posture"),
        )
    } else {
        (
            Kind::Backpressured,
            capacity_mismatch_detail(&subject, strategy, actual),
        )
    };
    WorthQueryExecutionResourceAdmissionDenial::new(kind, detail, counters)
}

fn classify_request_mismatch(
    contract: &WorthQueryExecutionResourceContract,
    request: &WorthQueryExecutionResourceRequest,
    counters: WorthQueryExecutionResourceAdmissionCounters,
) -> WorthQueryExecutionResourceAdmissionDenial {
    if contract
        .strategies()
        .iter()
        .all(|strategy| strategy.envelope().boundary() != request.boundary())
    {
        return WorthQueryExecutionResourceAdmissionDenial::new(
            Kind::ExecutionBoundaryUnsupported,
            "no installed strategy supports the requested execution boundary",
            counters,
        );
    }
    let capacity = contract
        .strategies()
        .iter()
        .filter(|strategy| request_fits_capacity(request, strategy))
        .collect::<Vec<_>>();
    if capacity.is_empty() {
        return WorthQueryExecutionResourceAdmissionDenial::new(
            Kind::ResourceCeilingExceeded,
            "no installed strategy admits every requested scale and resource dimension",
            counters,
        );
    }
    let safe_point = capacity
        .into_iter()
        .filter(|strategy| {
            request.cancellation_safe_point() == strategy.envelope().cancellation_safe_point()
        })
        .collect::<Vec<_>>();
    if safe_point.is_empty() {
        return WorthQueryExecutionResourceAdmissionDenial::new(
            Kind::CancellationSafePointUnsupported,
            "no capacity-fitting strategy supports the requested cancellation safe point",
            counters,
        );
    }
    let degradation = safe_point
        .into_iter()
        .filter(|strategy| {
            strategy
                .envelope()
                .degradation()
                .is_none_or(|posture| request.degradations().contains(&posture))
        })
        .collect::<Vec<_>>();
    if degradation.is_empty() {
        return WorthQueryExecutionResourceAdmissionDenial::new(
            Kind::DegradationPostureUnsupported,
            "capacity-fitting strategies require an unapproved named degradation",
            counters,
        );
    }
    let partial_effect = degradation
        .into_iter()
        .filter(|strategy| {
            request
                .partial_effect_postures()
                .contains(&strategy.envelope().partial_effect_posture())
        })
        .collect::<Vec<_>>();
    if partial_effect.is_empty() {
        return WorthQueryExecutionResourceAdmissionDenial::new(
            Kind::PartialEffectPostureUnsupported,
            "capacity-fitting strategies require an unapproved partial-effect posture",
            counters,
        );
    }
    let yielded_state = partial_effect
        .into_iter()
        .filter(|strategy| {
            request
                .yielded_state_postures()
                .contains(&strategy.envelope().yielded_state_posture())
        })
        .collect::<Vec<_>>();
    if yielded_state.is_empty() {
        return WorthQueryExecutionResourceAdmissionDenial::new(
            Kind::YieldedStatePostureUnsupported,
            "capacity-fitting strategies require an unapproved yielded-state posture",
            counters,
        );
    }
    let retained_progress = yielded_state
        .into_iter()
        .filter(|strategy| {
            request
                .retained_progress_postures()
                .contains(&strategy.envelope().retained_progress_posture())
        })
        .collect::<Vec<_>>();
    if retained_progress.is_empty() {
        return WorthQueryExecutionResourceAdmissionDenial::new(
            Kind::RetainedProgressPostureUnsupported,
            "capacity-fitting strategies require an unapproved retained-progress posture",
            counters,
        );
    }
    if retained_progress
        .iter()
        .any(|strategy| strategy.envelope().mode() == WorthQueryExecutionMode::Asynchronous)
    {
        return WorthQueryExecutionResourceAdmissionDenial::new(
            Kind::AsyncExecutionRequired,
            "the bounded request requires a declared asynchronous strategy",
            counters,
        );
    }
    WorthQueryExecutionResourceAdmissionDenial::new(
        Kind::ExecutionModeUnsupported,
        "no capacity-fitting strategy supports an allowed execution mode",
        counters,
    )
}

fn request_fits_capacity(
    request: &WorthQueryExecutionResourceRequest,
    strategy: &WorthQueryExecutionStrategyContract,
) -> bool {
    request.boundary() == strategy.envelope().boundary()
        && request
            .scale()
            .iter()
            .all(|(axis, value)| strategy.envelope().admits_scale(axis, value))
        && request
            .limits()
            .iter()
            .all(|(dimension, value)| strategy.envelope().admits_resource(dimension, value))
}
