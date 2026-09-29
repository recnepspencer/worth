//! A generation succession reads conditions through the prepared expression
//! succession: a hovered consumer stays current across an unchanged
//! condition, a changed outcome publishes its new pointer family, and an
//! uncommitted succession leaves the predecessor as it was.

use super::*;
use crate::facade::entry::active_application_session::succession_characterization::{
    assert_owners_follow, assert_pointer_is_fresh, assert_standing_is_fresh, SuccessionWork,
};

/// Runs a live evidence-only rebind whose mutability condition body is
/// `mutable_when`, and returns whether it prepared a mounted frame.
fn evidence_only_rebind(world: &mut World, mutable_when: &str, request: u64) -> bool {
    let prepared = prepare_rebind(&mut world.session, mutable_when, request);
    let framed = prepared.prepared_frame().is_some();
    if framed {
        world.host.push_native_display_settled_without_effects();
    }
    assert!(matches!(
        prepared.execute(request),
        crate::runtime::rebind::UiRebindOutcome::Published(_)
    ));
    framed
}

/// Classifies a source whose mutability condition body is `mutable_when` and
/// prepares its evidence-only rebind, without executing it.
fn prepare_rebind<'session>(
    session: &'session mut crate::facade::WorthUiActiveApplicationSession,
    mutable_when: &str,
    request: u64,
) -> crate::runtime::rebind::UiPreparedRebind<'session> {
    let name = format!("condition-follow-{request}");
    let candidate = submission(session, mutable_when, Policy::Fact, &name);
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) =
        session.classify_observations(observations).unwrap()
    else {
        panic!("a condition body is not part of the runtime artifact");
    };
    let plan = session
        .compile_preservation_rebind(
            evidence,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(request),
        )
        .unwrap()
}

/// A world whose pointer hovers its one consumer, painted operable.
pub(super) fn hovered() -> (World, UiMountedInstanceIdentity) {
    let mut world = world(Policy::Fact);
    let (target, _) = activate(&mut world.session, world.surface, 1);
    repaint(
        &mut world,
        Policy::Fact,
        target,
        10,
        UiPointerAffordanceFamily::Activation,
        2,
    );
    (world, target)
}

/// The pointer family the active snapshot holds for the hovered consumer,
/// and whether the displayed frame still owes the pointer a presentation.
pub(super) fn hovered_family(
    world: &World,
    target: UiMountedInstanceIdentity,
) -> (crate::declaration::UiPointerAffordance, bool) {
    let snapshot = world.session.pointer_affordance_snapshot.as_ref().unwrap();
    assert_eq!(
        snapshot.generation(),
        &world.session.active_generation_identity()
    );
    let family = snapshot
        .projections()
        .iter()
        .find(|row| row.target() == Some(target))
        .expect("the pointer still hovers the consumer")
        .family();
    let displayed = world
        .session
        .mounted
        .current_presentation_for_surface(world.surface)
        .unwrap();
    let pending = world.session.mounted.pointer_presentation_pending(
        world.surface,
        displayed.basis().binding(),
        Some(snapshot),
    );
    (family, pending)
}

#[test]
fn a_successor_keeps_the_hovered_affordance_current_for_a_current_condition() {
    let (mut world, target) = hovered();
    let predecessor = world.session.active_generation_identity();
    let work = SuccessionWork::read(&world.session);

    assert!(
        !evidence_only_rebind(&mut world, "f", 3),
        "an unchanged condition changes no pointer family, so the rebind \
         prepares no pointer publication and no frame"
    );

    let (family, pending) = hovered_family(&world, target);
    assert_eq!(
        family,
        crate::declaration::UiPointerAffordance::Activation,
        "a current condition is read through the prepared succession, not as stale"
    );
    assert!(
        !pending,
        "the displayed pointer already shows the current affordance"
    );
    assert_ne!(world.session.active_generation_identity(), predecessor);
    assert_pointer_is_fresh(&world.session, true);
    assert_standing_is_fresh(&world.session, 1);
    assert_owners_follow(&world.session, true);
    assert_eq!(
        work.since(&world.session),
        SuccessionWork {
            reobservations: 99,
            operand_probes: 99,
            index_hits: 99,
            evaluations: 99,
            appearance_batches: 99,
        },
        "an evidence-only successor (W1) re-stamps its unchanged conditions"
    );
    let _ = world.session.shutdown();
}

#[test]
fn a_successor_that_changes_a_hovered_condition_publishes_its_new_family() {
    let (mut world, target) = hovered();
    assert_eq!(
        hovered_family(&world, target).0,
        crate::declaration::UiPointerAffordance::Activation
    );
    let predecessor = world.session.active_generation_identity();
    let work = SuccessionWork::read(&world.session);

    assert!(
        evidence_only_rebind(&mut world, "!f", 3),
        "a changed pointer family is published with the successor's frame"
    );

    let (family, pending) = hovered_family(&world, target);
    assert_eq!(
        family,
        crate::declaration::UiPointerAffordance::Default,
        "the readonly consumer no longer offers activation"
    );
    assert!(!pending, "the published frame shows the new family");
    assert_ne!(world.session.active_generation_identity(), predecessor);
    assert_pointer_is_fresh(&world.session, true);
    assert_standing_is_fresh(&world.session, 1);
    assert_owners_follow(&world.session, true);
    assert_eq!(
        work.since(&world.session),
        SuccessionWork {
            reobservations: 99,
            operand_probes: 99,
            index_hits: 99,
            evaluations: 99,
            appearance_batches: 99,
        },
        "a changed pointer family upgrades the rebind to an authored \r
         successor (W2) published while attached"
    );
    let _ = world.session.shutdown();
}

#[test]
fn an_uncommitted_succession_leaves_the_predecessor_records_untouched() {
    let (mut world, target) = hovered();
    let predecessor = world.session.active_generation_identity();
    let before = world
        .session
        .expression_record(MUTABLE_WHEN)
        .unwrap()
        .clone();
    let (family, _) = hovered_family(&world, target);
    let work = world.session.expression_work_counters();

    let prepared = prepare_rebind(&mut world.session, "!f", 3);
    assert!(
        prepared.prepared_frame().is_some(),
        "the successor would change the hovered family"
    );
    drop(prepared);

    assert_eq!(world.session.active_generation_identity(), predecessor);
    let record = world.session.expression_record(MUTABLE_WHEN).unwrap();
    assert_eq!(record.generation(), &predecessor);
    assert_eq!(record.outcome(), before.outcome());
    assert_eq!(record.outcome_revision(), before.outcome_revision());
    assert_eq!(record.program_identity(), before.program_identity());
    let reference = world.session.expression_result(MUTABLE_WHEN).unwrap();
    assert!(world.session.is_current_expression_result(&reference));
    assert_eq!(hovered_family(&world, target), (family, false));
    assert_eq!(
        world.session.expression_work_counters().evaluations,
        work.evaluations + 1,
        "preparing rebuilt the changed condition once; the work counts \
         although the succession never committed"
    );
    let _ = world.session.shutdown();
}

#[test]
fn a_committed_succession_evaluates_each_rebuilt_condition_once() {
    let (mut world, _) = hovered();
    let work = world.session.expression_work_counters();

    evidence_only_rebind(&mut world, "!f", 3);

    let followed = world.session.expression_work_counters();
    assert_eq!(
        followed.evaluations,
        work.evaluations + 1,
        "the commit installs the prepared record; it does not evaluate again"
    );
    assert_eq!(followed.published_changes, work.published_changes + 1);
    let _ = world.session.shutdown();
}

#[test]
fn a_successor_reobserves_only_when_a_condition_outcome_changes() {
    let (mut world, target) = hovered();
    let graph = world.graph;
    let attempts = reobservations(&world.session);
    let operable = standing(&world.session, graph, target);
    let work = world.session.expression_work_counters();

    evidence_only_rebind(&mut world, "f", 3);
    let restamped = world.session.expression_work_counters();
    assert_eq!(
        restamped.published_changes, work.published_changes,
        "an unchanged body is re-stamped, not rebuilt"
    );
    assert_eq!(
        reobservations(&world.session),
        attempts,
        "an unchanged condition body re-observes nothing"
    );
    assert_eq!(
        standing(&world.session, graph, target).decision(),
        operable.decision()
    );

    evidence_only_rebind(&mut world, "f && f", 4);
    assert_eq!(
        world.session.expression_work_counters().published_changes,
        restamped.published_changes + 1,
        "a different program is rebuilt and settles one fresh record"
    );
    assert_eq!(
        reobservations(&world.session),
        attempts,
        "a rebuilt condition that settles to its prior outcome re-observes nothing"
    );
    assert_eq!(
        standing(&world.session, graph, target).decision(),
        operable.decision()
    );

    evidence_only_rebind(&mut world, "!f", 5);
    let followed = standing(&world.session, graph, target);
    assert_eq!(
        followed.decision().mutability(),
        UiIntentMutabilityPosture::Readonly
    );
    assert_eq!(
        reobservations(&world.session),
        attempts + 1,
        "a changed condition outcome re-observes its one consumer once"
    );
    let _ = world.session.shutdown();
}
