use worth_query_host::facade::primary_graph;

use super::super::courtroom_support::{observe, outcome_kind, wake_evidence};
use super::super::schema::IntentGateField;
use super::super::world::{request_scope, CourtroomWorld};

/// The installed bound on relevant commits one observation reconsiders.
const COMMITS_PER_OBSERVATION: usize = 8;

type Outcome = primary_graph::WorthQueryConditionalClockObservationOutcome<
    super::super::adapters::CourtroomClock,
>;
type Receipt = primary_graph::WorthQueryConditionalClockObservationReceipt<
    super::super::adapters::CourtroomClock,
>;

/// A clock reading that follows an accepted one after the observation already
/// consumed native writer commits from the subscription.
enum FollowingReading {
    Stale,
    Reordered,
    Duplicate,
}

/// The commit cursor moves past a native commit before the clock decides on
/// the reading. A stale reading must not lose that commit's invalidation: the
/// next accepted observation emits it.
pub fn a_stale_clock_reading_after_a_foreign_commit_keeps_its_invalidation_for_the_next_accepted_observation(
) {
    following_reading_emits_consumed_invalidation_once(FollowingReading::Stale);
}

/// Same as the stale case for a reading whose coordinate runs backwards.
pub fn a_reordered_clock_reading_after_a_foreign_commit_keeps_its_invalidation_for_the_next_accepted_observation(
) {
    following_reading_emits_consumed_invalidation_once(FollowingReading::Reordered);
}

/// A duplicate reading releases what its observation consumed: its own batch
/// carries the commit, and the next accepted observation does not repeat it.
pub fn a_duplicate_clock_reading_after_a_foreign_commit_emits_its_invalidation() {
    following_reading_emits_consumed_invalidation_once(FollowingReading::Duplicate);
}

fn following_reading_emits_consumed_invalidation_once(following: FollowingReading) {
    let world = CourtroomWorld::publish("blocked");
    observe(&world);
    let intent = world.intent_record_identity();
    // One more relevant native commit than a single observation reconsiders,
    // so the last one is consumed by the observation that follows.
    for ordinal in 1..=COMMITS_PER_OBSERVATION {
        write_gate(&world, format!("held-{ordinal}"));
    }
    write_gate(&world, "ready".to_string());

    // One port keeps one evaluation binding, so its managed clock remembers
    // the accepted reading that the following one is judged against.
    let selected = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    let mut clock = selected.conditional_clock(&world.clock).unwrap();
    world.clock_control.push(2, 10);
    let mut first = accepted(clock.observe());
    assert_eq!(first.authoritative_commit_count(), COMMITS_PER_OBSERVATION);
    assert!(
        first.authoritative_work_remaining(),
        "the last native commit must be left to the following reading"
    );
    // Value-free writes of one field are one invalidation; only the newest
    // commit carrying it is emitted.
    let held = delivered_intent_commits(intent, &first.take_granular_invalidation_batch());
    assert_eq!(held.len(), 1);

    match following {
        FollowingReading::Stale => world.clock_control.push(1, 10),
        FollowingReading::Reordered => world.clock_control.push(3, 9),
        FollowingReading::Duplicate => world.clock_control.push(2, 10),
    }
    let duplicate_batch = match (following, clock.observe()) {
        (FollowingReading::Stale, Outcome::Stale)
        | (FollowingReading::Reordered, Outcome::Reordered) => None,
        (FollowingReading::Duplicate, Outcome::Duplicate(mut duplicate)) => {
            Some(duplicate.take_granular_invalidation_batch())
        }
        (_, outcome) => panic!("{}", outcome_kind(&outcome)),
    };

    world.clock_control.push(4, 11);
    let mut next = accepted(clock.observe());
    assert!(!next.authoritative_work_remaining());
    let next_batch = next.take_granular_invalidation_batch();
    let (carrier, after) = match duplicate_batch {
        Some(duplicate) => (duplicate, Some(next_batch)),
        None => (next_batch, None),
    };
    assert_eq!(
        carrier.observation().direct_truth_delivery_count(),
        1,
        "the native commit consumed by the following reading must be delivered: {}",
        wake_evidence(&next)
    );
    assert_eq!(
        carrier.coverage(),
        primary_graph::WorthQueryGranularInvalidationCoverage::Exact
    );
    let ready = delivered_intent_commits(intent, &carrier);
    assert_eq!(ready.len(), 1);
    assert!(
        ready[0] > held[0],
        "the emitted invalidation must be the commit the following reading consumed"
    );
    if let Some(after) = after {
        assert!(
            !delivered_intent_commits(intent, &after).contains(&ready[0]),
            "the duplicate reading already emitted its invalidation"
        );
    }
}

fn write_gate(world: &CourtroomWorld, gate: String) {
    world
        .application
        .publish_native_field_write_for_test(
            world.application.current_world(),
            world.intent_record_identity(),
            IntentGateField::reference(),
            gate,
            &request_scope(),
        )
        .expect("the courtroom fixture requires an open application owner");
}

fn accepted(outcome: Outcome) -> Receipt {
    let Outcome::Accepted(receipt) = outcome else {
        panic!("{}", outcome_kind(&outcome))
    };
    receipt
}

/// The Relational commit of each delivered invalidation, after checking that
/// every delivered change names the intent record.
fn delivered_intent_commits(
    intent: primary_graph::RelationalBridgeRecordIdentityParts,
    batch: &primary_graph::WorthQueryGranularInvalidationDeliveryBatch,
) -> Vec<u64> {
    batch
        .bridge_deliveries()
        .iter()
        .map(|delivery| {
            let change_set = delivery.correspondence_receipt().change_set();
            assert!(
                change_set
                    .changes()
                    .iter()
                    .all(|change| change.relational_record_identity() == Some(intent)),
                "every delivered change must name the intent record"
            );
            change_set
                .commit_identity()
                .relational_commit_id()
                .expect("native writes are Relational commits")
        })
        .collect()
}
