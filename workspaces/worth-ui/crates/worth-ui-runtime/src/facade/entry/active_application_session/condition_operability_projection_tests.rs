//! A condition that reads a Query-scalar projection, across an authored
//! content successor whose own published frame moves that projection. The
//! committed successor, the standing fact, the painted appearance and the
//! hovered pointer family all follow the published value; what the consumers
//! re-observe is measured against the owner the successor replaced, identity
//! by identity, whatever slot each now holds; an application fact updated
//! while the publication is in flight is caught up; and no generation commits
//! under that publication.

use super::projection_world::{
    close, execute, hovered_online, in_flight, observe, online, outcome, status, submission,
    Mutability, Recovery,
};
use super::*;
use crate::facade::expression::UiExpressionOutcome;
use crate::runtime::rebind::{UiChangeProfile, UiRebindOutcome};

#[test]
fn an_authored_successor_follows_the_projection_its_own_frame_publishes() {
    let (mut world, owner, target) = hovered_online(UiChangeProfile::platform_pulse());
    assert_eq!(
        outcome(&world, MUTABLE_WHEN),
        UiExpressionOutcome::Condition(true)
    );

    let plan = observe(
        &mut world,
        status(&owner, "OFFLINE", 2),
        Some(Recovery {
            mutability: Mutability::Projected,
            ahead: false,
        }),
        6,
    );
    execute(&mut world, plan, 6);

    assert_eq!(
        outcome(&world, MUTABLE_WHEN),
        UiExpressionOutcome::Condition(false),
        "the successor was prepared before its frame published OFFLINE; the \
         commit catches it up"
    );
    let fact = standing(&world.session, world.graph, target);
    assert_eq!(
        fact.decision().mutability(),
        UiIntentMutabilityPosture::Readonly
    );
    close(&mut world.session, "repaint");
    assert_eq!(
        super::succession_tests::hovered_family(&world, target),
        (crate::declaration::UiPointerAffordance::Default, true),
        "the next turn reads the readonly consumer, which the displayed \
         pointer does not show yet"
    );
    let frame = project(
        &mut world.session,
        &[(target, 20)],
        &[(
            world.surface,
            Some((target, UiPointerAffordanceFamily::Default)),
        )],
    );
    publish(&mut world.session, &world.host, frame, 7);
    assert_eq!(
        super::succession_tests::hovered_family(&world, target),
        (crate::declaration::UiPointerAffordance::Default, false)
    );
    let _ = world.session.shutdown();
}

#[test]
fn a_successor_reports_against_the_owner_it_replaces() {
    let (mut world, owner, target) = hovered_online(UiChangeProfile::platform_pulse());

    let plan = observe(
        &mut world,
        status(&owner, "OFFLINE", 2),
        Some(Recovery {
            mutability: Mutability::Fact,
            ahead: false,
        }),
        6,
    );
    execute(&mut world, plan, 6);

    assert_eq!(
        outcome(&world, MUTABLE_WHEN),
        UiExpressionOutcome::Condition(true),
        "the new body reads the mutability fact, not the projection"
    );
    assert_eq!(
        standing(&world.session, world.graph, target)
            .decision()
            .mutability(),
        UiIntentMutabilityPosture::Writable,
        "the replaced owner settled OFFLINE to false as the frame published; \
         the successor holds true, so its consumer is re-observed although \
         the successor's own preparation changed nothing"
    );
    let _ = world.session.shutdown();
}

#[test]
fn a_fact_updated_while_the_successor_is_in_flight_is_caught_up() {
    let (mut world, owner, target) = hovered_online(UiChangeProfile::platform_pulse());
    let plan = observe(
        &mut world,
        status(&owner, "OFFLINE", 2),
        Some(Recovery {
            mutability: Mutability::Projected,
            ahead: false,
        }),
        6,
    );
    let pending = in_flight(&mut world, plan, 6);

    set(&mut world.session, fixture::POLICY, false);
    assert!(matches!(
        pending.complete(&mut world.session, 7),
        UiRebindOutcome::Published(_)
    ));

    assert_eq!(
        outcome(&world, POLICY_WHEN),
        UiExpressionOutcome::Condition(false),
        "the successor was prepared before the update; the commit catches it up"
    );
    let decision = standing(&world.session, world.graph, target)
        .decision()
        .clone();
    assert_eq!(decision.policy(), UiIntentPolicyPosture::Denied);
    assert_eq!(decision.mutability(), UiIntentMutabilityPosture::Readonly);
    let _ = world.session.shutdown();
}

#[test]
fn a_shifted_condition_is_measured_by_its_identity() {
    let (mut world, owner, target) = hovered_online(UiChangeProfile::platform_pulse());
    let before = slot(&world, MUTABLE_WHEN);

    let plan = observe(
        &mut world,
        status(&owner, "OFFLINE", 2),
        Some(Recovery {
            mutability: Mutability::Fact,
            ahead: true,
        }),
        6,
    );
    execute(&mut world, plan, 6);

    assert_eq!(slot(&world, MUTABLE_WHEN), before + 1);
    assert_eq!(
        outcome(&world, MUTABLE_WHEN),
        UiExpressionOutcome::Condition(true)
    );
    assert_eq!(
        standing(&world.session, world.graph, target)
            .decision()
            .mutability(),
        UiIntentMutabilityPosture::Writable,
        "the replaced owner held false for this identity one slot earlier; \
         the slot it moved into held a true condition of another identity"
    );
    let _ = world.session.shutdown();
}

#[test]
fn a_shifted_condition_with_its_outcome_kept_is_not_reobserved() {
    let (in_place, _) = reobserved_across(false);
    let (shifted, moved) = reobserved_across(true);
    assert!(
        moved,
        "the authored condition moves every consumer condition"
    );
    assert_eq!(
        shifted, in_place,
        "a condition whose outcome holds re-observes nothing, whatever slot \
         it moved to"
    );
}

/// The consumer re-observations of the turn whose authored successor keeps
/// the projected condition, with or without a condition ahead of it, and
/// whether its slot moved.
fn reobserved_across(ahead: bool) -> (u64, bool) {
    let (mut world, owner, _) = hovered_online(UiChangeProfile::platform_pulse());
    let before = slot(&world, MUTABLE_WHEN);
    let attempts = reobservations(&world.session);

    let plan = observe(
        &mut world,
        status(&owner, "OFFLINE", 2),
        Some(Recovery {
            mutability: Mutability::Projected,
            ahead,
        }),
        6,
    );
    execute(&mut world, plan, 6);

    assert_eq!(
        outcome(&world, MUTABLE_WHEN),
        UiExpressionOutcome::Condition(false)
    );
    let moved = slot(&world, MUTABLE_WHEN) != before;
    let reobserved = reobservations(&world.session) - attempts;
    let _ = world.session.shutdown();
    (reobserved, moved)
}

fn slot(world: &World, identity: &str) -> usize {
    world
        .session
        .expressions
        .catalog()
        .slot_of(identity)
        .unwrap()
        .index()
}

#[test]
fn an_evidence_only_successor_waits_for_the_presentation_in_flight() {
    let pulse = UiChangeProfile::platform_pulse();
    let concurrency = crate::runtime::rebind::UiRebindConcurrencyInput {
        completion_handles: 2,
        ..pulse.rebind().concurrency()
    };
    let rebind =
        crate::runtime::rebind::UiRebindProfile::bounded(pulse.rebind().budget(), concurrency)
            .unwrap();
    let (mut world, owner) = online(UiChangeProfile::new(pulse.observation(), rebind));
    let plan = observe(
        &mut world,
        status(&owner, "OFFLINE", 2),
        Some(Recovery {
            mutability: Mutability::Fact,
            ahead: false,
        }),
        6,
    );
    let pending = in_flight(&mut world, plan, 6);
    let predecessor = world.session.active_generation_identity();

    let source = submission(
        world.session.capabilities(),
        "revision",
        Recovery {
            mutability: Mutability::Projected,
            ahead: true,
        },
        "evidence-7",
    );
    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) =
        world.session.classify_observations(admitted).unwrap()
    else {
        panic!("a condition is not part of the runtime artifact");
    };
    let plan = world
        .session
        .compile_preservation_rebind(
            evidence,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let denial = world
        .session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(7),
        )
        .err();
    assert_eq!(
        denial,
        Some(crate::runtime::rebind::UiRebindPreparationDenial::FrameBoundaryUnavailable),
        "a second completion handle admits the rebind, but no generation commits \
         under a presentation in flight"
    );
    assert_eq!(world.session.active_generation_identity(), predecessor);

    assert!(matches!(
        pending.complete(&mut world.session, 8),
        UiRebindOutcome::Published(_)
    ));
    assert_ne!(world.session.active_generation_identity(), predecessor);
    assert_eq!(
        outcome(&world, MUTABLE_WHEN),
        UiExpressionOutcome::Condition(true)
    );
    let _ = world.session.shutdown();
}
