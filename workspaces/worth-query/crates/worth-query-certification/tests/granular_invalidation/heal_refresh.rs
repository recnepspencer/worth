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

/// A relevant write followed by more unrelated writes than the subscription
/// retains forces the conditional observation to heal from truth. The gap's
/// invalidations are never delivered, so the healed batch must make every
/// live projection refresh its full scope, and the projection must show the
/// relevant write.
pub fn assert_healed_observation_refreshes_live_projection() {
    // Strictly more than the installation's retained positions (128).
    const UNRELATED_COMMITS: u64 = 130;
    let mut world = CourtroomWorld::publish_with_unrelated_rows("blocked", 1);
    let mut query = build_primary_query_world(&world);
    let mut suppressed = observe(&mut world);
    assert_eq!(suppressed.committed_operation_count(), 0);
    assert!(suppressed.take_granular_invalidation_batch().is_empty());

    let branch = world.application.current_world();
    let unrelated = world
        .application
        .on_branch(branch)
        .select()
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            UnrelatedValueField::reference(),
            0,
            &request_scope(),
            primary_graph::WorthQueryPrincipalResolutionMode::Certification,
        )
        .expect("the unrelated row must remain exactly resolvable")
        .relational_record_identity_parts();
    world
        .application
        .publish_native_field_write_for_test(
            branch,
            world.intent_record_identity(),
            IntentGateField::reference(),
            "ready".to_string(),
            &request_scope(),
        )
        .expect("the native-writer fixture requires an open application owner");
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
            .expect("the native-writer fixture requires an open application owner");
    }

    let mut healed = observe(&mut world);
    let batch = healed.take_granular_invalidation_batch();
    assert_eq!(
        batch.coverage(),
        WorthQueryGranularInvalidationCoverage::RefreshAll,
        "a healed observation must not claim exact coverage of its gap"
    );
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        world.application.granular_invalidation_installation(),
    );
    let outcome =
        maintain_primary_runtime_granular_batch(&query.live, &mut query.workspace, &binding, batch)
            .expect("the healed batch must maintain the live projection");
    let WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the gap's relevant write must reach the live projection")
    };
    let patch = performed.deliveries()[0]
        .effect()
        .projection_patch()
        .expect("the full-scope refresh must carry the performed field patch");
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
        .expect("the refreshed projection must carry the gate field");
    assert_eq!(
        gate.native_value().scalar(),
        Some(&AspectValue::String("ready".into()))
    );
}

fn observe(
    world: &mut CourtroomWorld,
) -> primary_graph::WorthQueryConditionalClockObservationReceipt<crate::adapters::CourtroomClock> {
    match world.conditional_clock().observe() {
        primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) => receipt,
        _ => panic!("the due courtroom observation must be accepted"),
    }
}
