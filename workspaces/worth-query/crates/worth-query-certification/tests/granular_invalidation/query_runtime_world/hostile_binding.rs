use super::{
    domain, try_build_primary_query_world_with_dimensions, PrimaryQueryScale, PrimaryQueryWorld,
};

pub fn assert_foreign_primary_source_is_denied_at_build(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let invalidation_host = crate::host_world::CourtroomWorld::publish("blocked");
    let source_host = crate::host_world::CourtroomWorld::publish("blocked");
    let source_installation = source_host.application.granular_invalidation_installation();
    let snapshot_installation = invalidation_host
        .application
        .granular_invalidation_installation();
    let denial = try_build_primary_query_world_with_dimensions(
        request_execution,
        &invalidation_host,
        domain::ConsumerProfile::ValuePatch,
        PrimaryQueryScale::default(),
        &source_installation,
        &snapshot_installation,
    )
    .err()
    .expect("a source adapter for runtime B must not compose with invalidations from runtime A");
    assert!(denial.contains("do not retain the same current runtime"));
}

pub fn build_with_foreign_snapshot_adapter(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &crate::host_world::CourtroomWorld,
    snapshot_host: &crate::host_world::CourtroomWorld,
) -> PrimaryQueryWorld {
    let source_installation = host.application.granular_invalidation_installation();
    let snapshot_installation = snapshot_host
        .application
        .granular_invalidation_installation();
    try_build_primary_query_world_with_dimensions(
        request_execution,
        host,
        domain::ConsumerProfile::ValuePatch,
        PrimaryQueryScale::default(),
        &source_installation,
        &snapshot_installation,
    )
    .expect("an ambient snapshot adapter cannot govern the granular source lane")
}
