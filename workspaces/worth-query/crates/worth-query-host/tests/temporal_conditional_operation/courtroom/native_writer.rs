use worth_query_host::facade::primary_graph;

use super::super::courtroom_support::{
    assert_authoritative_value, observe, outcome_kind, raw_observe, wake_evidence,
};
use super::super::schema::{IntentEffectField, IntentGateField, UnrelatedValueField};
use super::super::world::{request_scope, CourtroomWorld};

/// A commit made by an ordinary native Relational writer, never by Query,
/// reaches the pending conditional operation through the one canonical
/// subscription and is reconsidered against the new truth.
pub fn native_writer_commit_reconsiders_a_suppressed_wake() {
    let world = CourtroomWorld::publish("blocked");
    let suppressed = observe(&world);
    assert_eq!(
        suppressed.committed_operation_count(),
        0,
        "{}",
        wake_evidence(&suppressed)
    );
    assert_eq!(
        suppressed.retained_suppressed_wake_count(),
        1,
        "{}",
        wake_evidence(&suppressed)
    );

    world.application.publish_native_field_write_for_test(
        world.application.current_world(),
        world.intent_record_identity(),
        IntentGateField::reference(),
        "ready".to_string(),
        &request_scope(),
    );

    let mut reconsidered = observe(&world);
    assert_eq!(
        reconsidered.authoritative_commit_count(),
        1,
        "the native commit must reach the pending operation: {}",
        wake_evidence(&reconsidered)
    );
    assert_eq!(
        reconsidered.committed_operation_count(),
        1,
        "{}",
        wake_evidence(&reconsidered)
    );
    assert_eq!(
        reconsidered.retained_due_wake_count(),
        0,
        "{}",
        wake_evidence(&reconsidered)
    );
    let batch = reconsidered.take_granular_invalidation_batch();
    assert_eq!(batch.observation().direct_truth_delivery_count(), 1);
    assert_eq!(batch.observation().signal_performed_delivery_count(), 1);
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "payload".to_string(),
    );
}

/// A busy branch commits more unrelated work than the subscription retains
/// between two observations, with one relevant change inside that gap. The
/// observation heals in place, rebuilding from truth without a reinstall, and
/// still acts on the relevant change.
pub fn busy_branch_heals_a_lagging_cursor_inside_observation() {
    // Strictly more than the installation's retained positions (128).
    const UNRELATED_COMMITS: u64 = 130;
    let world = CourtroomWorld::publish_with_unrelated_rows("blocked", 1);
    let suppressed = observe(&world);
    assert_eq!(
        suppressed.retained_suppressed_wake_count(),
        1,
        "{}",
        wake_evidence(&suppressed)
    );
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
    world.application.publish_native_field_write_for_test(
        branch,
        world.intent_record_identity(),
        IntentGateField::reference(),
        "ready".to_string(),
        &request_scope(),
    );
    for value in 1..=UNRELATED_COMMITS {
        world.application.publish_native_field_write_for_test(
            branch,
            unrelated,
            UnrelatedValueField::reference(),
            value,
            &request_scope(),
        );
    }

    let healed = raw_observe(&world);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(healed) = healed
    else {
        panic!(
            "a lagging cursor must heal inside the observation: {}",
            outcome_kind(&healed)
        )
    };
    assert_eq!(
        healed.committed_operation_count(),
        1,
        "the relevant change inside the gap must still be acted on: {}",
        wake_evidence(&healed)
    );
    assert_eq!(
        healed.retained_due_wake_count(),
        0,
        "{}",
        wake_evidence(&healed)
    );
    assert_authoritative_value(
        &world,
        IntentEffectField::reference(),
        "payload".to_string(),
    );
}
