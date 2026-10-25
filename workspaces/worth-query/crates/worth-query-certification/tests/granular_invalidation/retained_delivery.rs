use worth_foundational::facade::{AspectValue, FieldKey};
use worth_query::facade::domain::{
    bind_primary_runtime_granular_invalidations, maintain_primary_runtime_granular_batch,
    WorthQueryPrimaryGranularMaintenanceOutcome,
};
use worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationCoverage;
use worth_query_host::facade::primary_graph;

use crate::host_world::{request_scope, CourtroomWorld};
use crate::query_runtime_world::build_primary_query_world;
use crate::schema::{IntentGateField, UnrelatedValueField};

/// Retained canonical touches preserve exact invalidation after a relevant
/// write followed by many unrelated writes. The live projection must receive
/// that original write without reconstructing a lost subscription window.
pub fn assert_retained_observation_delivers_exact_live_projection(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    // Exceeds the retired 128-position window; canonical touches now retain continuity.
    const UNRELATED_COMMITS: u64 = 130;
    let mut world = CourtroomWorld::publish_with_unrelated_rows("blocked", 1);
    let mut query = build_primary_query_world(request_execution, &world);
    let mut suppressed = observe(request_execution, &mut world);
    assert_eq!(suppressed.committed_operation_count(), 0);
    assert!(suppressed.take_granular_invalidation_batch().is_empty());

    let branch = world.application.current_world();
    let unrelated = world
        .application
        .with_host_advancement(|host_phase| {
            world
                .application
                .on_branch(branch)
                .select(&host_phase)
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &host_phase,
                    UnrelatedValueField::reference(),
                    0,
                    &request_scope(),
                    primary_graph::WorthQueryPrincipalResolutionMode::Certification,
                )
                .expect("the unrelated row must remain exactly resolvable")
                .relational_record_identity_parts()
        })
        .expect("installed host call opens");
    world
        .application
        .publish_native_field_write_for_test(
            branch,
            world.intent_record_identity(),
            IntentGateField::reference(),
            "ready".to_string(),
            &request_scope(),
        )
        .expect("the fixture native write performs");
    for value in 1..=UNRELATED_COMMITS {
        world
            .application
            .publish_native_field_write_for_test(
                branch,
                unrelated,
                UnrelatedValueField::reference(),
                value,
                &request_scope(),
            )
            .expect("the fixture native write performs");
    }

    let mut observed = observe(request_execution, &mut world);
    let batch = observed.take_granular_invalidation_batch();
    assert_eq!(
        batch.coverage(),
        WorthQueryGranularInvalidationCoverage::Exact,
        "retained canonical touches keep exact coverage across unrelated writes"
    );
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        world.application.granular_invalidation_installation(),
    );
    let outcome = maintain_primary_runtime_granular_batch(
        request_execution,
        &query.live,
        &mut query.workspace,
        &binding,
        batch,
    )
    .expect("the retained batch must maintain the live projection");
    let WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the retained relevant write must reach the live projection")
    };
    let patch = performed.deliveries()[0]
        .effect()
        .projection_patch()
        .expect("the exact delivery must carry the performed field patch");
    let gate = patch
        .fields()
        .iter()
        .find(|fact| {
            fact.field_path()
                .canonical_field_path()
                .is_some_and(|path| {
                    path.fields()
                        .iter()
                        .map(FieldKey::as_str)
                        .eq(["IntentFacts", "IntentGateField"])
                })
        })
        .expect("the updated projection must carry the gate field");
    assert_eq!(
        gate.native_value().scalar(),
        Some(&AspectValue::String("ready".into()))
    );
}

fn observe(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut CourtroomWorld,
) -> primary_graph::WorthQueryConditionalClockObservationReceipt<crate::adapters::CourtroomClock> {
    match world.conditional_clock().observe(request_execution) {
        primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) => receipt,
        _ => panic!("the due courtroom observation must be accepted"),
    }
}
