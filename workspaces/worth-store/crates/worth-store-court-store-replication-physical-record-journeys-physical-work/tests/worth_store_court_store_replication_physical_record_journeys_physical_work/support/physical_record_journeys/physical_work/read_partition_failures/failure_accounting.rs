use super::{route_for, signal, RouteTarget, ServingPhysicalRuntime};

pub(crate) fn assert_failed_media_accounting(
    target: RouteTarget,
    before: worth_store_physical_backend::MediaCounterSnapshot,
    after: worth_store_physical_backend::MediaCounterSnapshot,
) {
    let role = worth_store_physical_backend::MediaOperationRole::PositionedRead;
    assert_eq!(
        after.fault_matches(),
        before.fault_matches() + 1,
        "{:?} must match exactly one backend fault",
        target.route
    );
    assert_eq!(
        after.identified_operation_attempts_for(role)
            - before.identified_operation_attempts_for(role),
        target.workload_ordinal,
        "{:?} identified work retried or advanced past the failed media effect",
        target.route
    );
    assert_eq!(
        after.attempts_for(role) - before.attempts_for(role),
        target.workload_ordinal,
        "{:?} emitted an unbound, repeated, or post-failure media attempt",
        target.route
    );
    assert_eq!(
        after.completed_operations_for(role) - before.completed_operations_for(role),
        target.workload_ordinal - 1,
        "{:?} did not complete each predecessor effect exactly once",
        target.route
    );
    assert_eq!(
        after.denied_before_effect_for(role) - before.denied_before_effect_for(role),
        1,
        "{:?} must have exactly one denied media attempt",
        target.route
    );
    assert_eq!(
        after.partial_effects_for(role) - before.partial_effects_for(role),
        0,
        "{:?} unexpectedly produced a partial media effect",
        target.route
    );
    assert_eq!(
        after.indeterminate_effects_for(role) - before.indeterminate_effects_for(role),
        0,
        "{:?} unexpectedly produced an indeterminate media effect",
        target.route
    );
}

pub(crate) fn assert_failed_route(
    target: RouteTarget,
    serving: &ServingPhysicalRuntime,
    causal_start: usize,
    invalidations_before: u64,
) {
    assert_eq!(
        signal(serving).aspect_invalidation_count(),
        invalidations_before,
        "{:?} transient denial must not invalidate physical truth",
        target.route
    );
    let records = serving.physical_work_observer().causal().records();
    let failures = records[causal_start..]
        .iter()
        .filter(|record| {
            record.operation()
                == worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead
                && record.effect_fate()
                    == worth_store::physical_runtime::PhysicalWorkEffectFate::ProvenNoEffect
        })
        .collect::<Vec<_>>();
    assert_eq!(
        failures.len(),
        1,
        "{:?} must retain one failed causal range record",
        target.route
    );
    let bindings = serving.physical_signal_aspect_binding_observations();
    assert_eq!(route_for(failures[0], &bindings), target.route);
}
