//! A source edit that changes executable meaning while the consumer is
//! mounted and hovered cuts the mounted application over: the successor
//! starts with no pointer snapshot and no standing facts, and its retained
//! owners name the successor generation.

use super::*;
use crate::facade::entry::active_application_session::succession_characterization::{
    assert_owners_follow, assert_pointer_is_fresh, assert_standing_is_fresh, SuccessionWork,
};
use crate::runtime::rebind::{UiRebindOutcome, UiRebindSemanticProof};

/// Plans the rebind of a turn that admits the launch source with its policy
/// read from `POLICY_WHEN` instead of the fact, which changes meaning.
fn changed_plan(world: &mut World) -> crate::runtime::rebind::UiRebindPlan {
    let candidate = submission(&world.session, "f", Policy::Condition, "condition-cutover");
    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::Changed(changed) =
        world.session.classify_observations(observations).unwrap()
    else {
        panic!("a different policy source changes executable meaning");
    };
    let lifecycle = world
        .session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = world
        .session
        .compile_rebind_plan(
            lifecycle,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    assert!(matches!(
        plan.semantic_proof(),
        UiRebindSemanticProof::Changed(_)
    ));
    plan
}

#[test]
fn a_mounted_cutover_starts_its_successor_with_no_pointer_or_standing_facts() {
    let (mut world, _) = super::succession_tests::hovered();
    let predecessor = world.session.active_generation_identity();
    assert_standing_is_fresh(&world.session, 1);
    let work = SuccessionWork::read(&world.session);

    let plan = changed_plan(&mut world);
    let host = world.host.clone();
    let prepared = world
        .session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(3),
        )
        .unwrap();
    for _ in prepared
        .prepared_frame()
        .into_iter()
        .flat_map(|frame| frame.surfaces())
    {
        host.push_native_display_settled_without_effects();
    }
    assert!(matches!(prepared.execute(3), UiRebindOutcome::Published(_)));

    assert_ne!(world.session.active_generation_identity(), predecessor);
    assert_pointer_is_fresh(&world.session, false);
    assert_standing_is_fresh(&world.session, 0);
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
        "the observation turn, the preparation, and the mounted cutover (W3)"
    );
    let _ = world.session.shutdown();
}
