//! Operability read from authored conditions: a condition change re-derives
//! the standing facts of its consumers with no activation, and only theirs.

use super::*;
use crate::runtime::intent::UiIntentOperabilityAppearanceClass as Class;
use worth_ui_dsl::*;

#[path = "condition_operability_admission_tests.rs"]
mod admission_tests;
#[path = "condition_operability_projection_tests.rs"]
mod projection_tests;
#[path = "condition_operability_projection_world.rs"]
mod projection_world;
#[path = "condition_operability_succession_tests.rs"]
mod succession_tests;

const MUTABLE_WHEN: &str = "test.appearance.mutable_when";
const READY_WHEN: &str = "test.appearance.ready_when";
const POLICY_WHEN: &str = "test.appearance.policy_when";

/// Where the consumer's policy axis reads from. `POLICY_WHEN` is authored
/// either way, so an update of the fact it reads always changes a condition.
#[derive(Clone, Copy)]
enum Policy {
    Fact,
    Condition,
}

/// The consumer module whose mutability and readiness read conditions over
/// the fixture facts; `mutable_when` is the mutability condition's body.
fn module(mutable_when: &str, policy: Policy) -> WorthUiRustAuthoredArtifactInputModule {
    let policy = match policy {
        Policy::Fact => WorthUiIntentPolicySourceSpec::application_boolean(fixture::POLICY),
        Policy::Condition => WorthUiIntentPolicySourceSpec::condition(POLICY_WHEN),
    };
    let operability = WorthUiIntentOperabilityContractSpec::new(
        "test.appearance.operability",
        WorthUiIntentMutabilitySourceSpec::condition(MUTABLE_WHEN),
        WorthUiIntentReadinessSourceSpec::condition(READY_WHEN),
        policy,
    );
    let mut module = fixture::consumer_module_with(Some(&fixture::role()), 1, &operability);
    for (identity, fact, body) in [
        (MUTABLE_WHEN, fixture::MUTABLE, mutable_when),
        (READY_WHEN, fixture::READY, "f"),
        (POLICY_WHEN, fixture::POLICY, "f"),
    ] {
        module = module
            .try_with_condition(
                identity,
                [WorthUiExpressionOperand::new(
                    "f",
                    WorthUiExpressionOperandSource::ApplicationBoolean {
                        fact: fact.to_owned(),
                    },
                )],
                body,
            )
            .unwrap();
    }
    module
}

fn source(mutable_when: &str, policy: Policy) -> WorthUiRustAuthoredArtifactInput {
    WorthUiRustAuthoredArtifactInput::from_modules([module(mutable_when, policy)])
}

fn submission(
    session: &crate::facade::WorthUiActiveApplicationSession,
    mutable_when: &str,
    policy: Policy,
    name: &str,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored(name)
            .with_rust_authored_input(source(mutable_when, policy)),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(name)],
        session.capabilities(),
    )
}

/// Closes an observation turn over the unchanged source, as `close` does for
/// the fact-backed fixture.
fn classify(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    policy: Policy,
    name: &str,
) {
    let source = submission(session, "f", policy, name);
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    session.classify_observations(admitted).unwrap();
}

struct World {
    session: crate::facade::WorthUiActiveApplicationSession,
    host: crate::certification_support::ScriptedPresentationHost,
    surface: UiSemanticSurfaceIdentity,
    graph: crate::graph::UiGraphNodeIdentity,
}

/// A mounted, published consumer whose conditions all hold.
fn world(policy: Policy) -> World {
    let role = fixture::role();
    let (mut session, host) = fixture::session_with_source(&role, source("f", policy));
    let (surface, graph) = super::super::mounting_fixture::mount(&mut session, 1_000);
    classify(&mut session, policy, "condition-operability-initial");
    session.advance_mounted_identity_frame().unwrap();
    let frame = prepare(&mut session);
    publish(&mut session, &host, frame, 1);
    World {
        session,
        host,
        surface,
        graph,
    }
}

/// Publishes the frame that paints `target` with `red` and `pointer`.
fn repaint(
    world: &mut World,
    policy: Policy,
    target: UiMountedInstanceIdentity,
    red: u8,
    pointer: UiPointerAffordanceFamily,
    now: u64,
) {
    classify(
        &mut world.session,
        policy,
        &format!("condition-repaint-{now}"),
    );
    let frame = project(
        &mut world.session,
        &[(target, red)],
        &[(world.surface, Some((target, pointer)))],
    );
    publish(&mut world.session, &world.host, frame, now);
}

fn reobservations(session: &crate::facade::WorthUiActiveApplicationSession) -> u64 {
    session
        .intent_admission_metrics()
        .operability_reobservations()
}

fn set(session: &mut crate::facade::WorthUiActiveApplicationSession, fact: &str, value: bool) {
    session
        .update_intent_boolean_fact(&fixture::fact(fact), value)
        .unwrap();
}

fn standing(
    session: &crate::facade::WorthUiActiveApplicationSession,
    graph: crate::graph::UiGraphNodeIdentity,
    target: UiMountedInstanceIdentity,
) -> crate::runtime::intent::UiIntentOperabilityStandingFact {
    session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap()
        .fact_for(graph, target, fixture::ROUTE)
        .unwrap()
        .clone()
}

#[test]
fn a_condition_flip_repaints_its_consumer_without_an_activation() {
    let mut world = world(Policy::Fact);
    let (surface, graph) = (world.surface, world.graph);
    let (target, operable) = activate(&mut world.session, surface, 1);
    assert_eq!(operable.primary_cause(), None);
    repaint(
        &mut world,
        Policy::Fact,
        target,
        10,
        UiPointerAffordanceFamily::Activation,
        2,
    );
    let before = world
        .session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap();
    let attempts = reobservations(&world.session);

    set(&mut world.session, fixture::MUTABLE, false);

    let after = world
        .session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap();
    let fact = after.fact_for(graph, target, fixture::ROUTE).unwrap();
    assert_eq!(
        fact.decision().mutability(),
        UiIntentMutabilityPosture::Readonly
    );
    assert_eq!(
        fact.decision().primary_cause(),
        Some(UiIntentInoperableCause::Readonly)
    );
    assert_eq!(fact.class(), Class::Denied);
    assert_eq!(after.changed_instances(&before).as_ref(), &[target]);
    assert_eq!(
        reobservations(&world.session),
        attempts + 1,
        "the one fact reading the changed condition is re-observed once"
    );
    repaint(
        &mut world,
        Policy::Fact,
        target,
        20,
        UiPointerAffordanceFamily::Default,
        3,
    );

    let pushed = standing(&world.session, graph, target);
    let (_, activated) = activate(&mut world.session, surface, 3);
    assert_eq!(
        &activated,
        pushed.decision(),
        "an activation derives the decision the push already recorded"
    );
    let _ = world.session.shutdown();
}

#[test]
fn only_changed_conditions_with_consumers_are_reobserved() {
    let World {
        mut session,
        surface,
        ..
    } = world(Policy::Fact);
    let _ = activate(&mut session, surface, 1);
    let attempts = reobservations(&session);
    let work = session.expression_work_counters();

    set(&mut session, fixture::POLICY, false);
    let changed = session.expression_work_counters();
    assert_eq!(changed.published_changes, work.published_changes + 1);
    assert_eq!(
        reobservations(&session),
        attempts,
        "a changed condition no declaration reads re-observes nothing"
    );

    set(&mut session, fixture::MUTABLE, true);
    let unchanged = session.expression_work_counters();
    assert_eq!(
        unchanged.suppressed_unchanged,
        changed.suppressed_unchanged + 1
    );
    assert_eq!(unchanged.published_changes, changed.published_changes);
    assert_eq!(
        reobservations(&session),
        attempts,
        "a condition that settles unchanged re-observes nothing"
    );
    let _ = session.shutdown();
}

#[test]
fn a_condition_flip_never_creates_a_standing_fact() {
    let World { mut session, .. } = world(Policy::Fact);
    let attempts = reobservations(&session);

    set(&mut session, fixture::MUTABLE, false);

    assert!(session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap()
        .facts()
        .is_empty());
    assert_eq!(reobservations(&session), attempts);
    let _ = session.shutdown();
}

#[test]
fn reobservation_keeps_the_affinity_the_fact_was_observed_with() {
    let World {
        mut session,
        surface,
        graph,
        ..
    } = world(Policy::Fact);
    let (target, _) = activate(&mut session, surface, 1);
    set(&mut session, fixture::MUTABLE, false);
    // A payload prepared before a successor records its fact under the
    // historical receipt, which re-observation refuses; the affinity is
    // therefore pinned on a fact whose target is still current.
    let rebind = standing(&session, graph, target)
        .with_affinity_for_test(UiIntentAffinityPosture::RebindRequired);
    let active = session.active_generation_identity();
    let prepared = session.application.prepared_authority();
    let owners = session.intent_read_owners(prepared, &active);

    let refreshed = crate::runtime::intent::reobserve_standing_fact(&rebind, owners).unwrap();

    assert_eq!(refreshed.mutability(), UiIntentMutabilityPosture::Readonly);
    assert_eq!(
        refreshed.affinity(),
        UiIntentAffinityPosture::RebindRequired,
        "a re-observed condition never upgrades the affinity it was observed with"
    );
    let _ = session.shutdown();
}

#[test]
fn reobservation_refreshes_only_the_route_the_fact_names() {
    let World {
        mut session,
        surface,
        graph,
        ..
    } = world(Policy::Fact);
    let (target, _) = activate(&mut session, surface, 1);
    let fact = standing(&session, graph, target);
    let active = session.active_generation_identity();
    let prepared = session.application.prepared_authority();
    let owners = session.intent_read_owners(prepared, &active);

    assert!(crate::runtime::intent::reobserve_standing_fact(&fact, owners).is_some());
    let renamed = fact.with_route_for_test("test.appearance.submit");
    assert!(
        crate::runtime::intent::reobserve_standing_fact(&renamed, owners).is_none(),
        "a fact whose route no longer resolves at its node is left to its lifecycle"
    );
    let _ = session.shutdown();
}
