//! An authored content successor whose presentation the native loop holds
//! detached, completed later against the session: what it commits, and
//! whether what it commits is still current when an application fact moved
//! while it was detached.

use super::projection_world::{hovered_online, in_flight, observe, status, Mutability, Recovery};
use super::*;
use crate::facade::entry::active_application_session::succession_characterization::{
    assert_owners_follow, assert_pointer_is_fresh, assert_standing_is_fresh, pointer_rows,
    SuccessionWork,
};
use crate::runtime::rebind::{UiChangeProfile, UiRebindOutcome};

/// A recovery whose mutability reads the fixture fact, which never moves, so
/// the successor keeps its consumer operable and the hovered row activating.
const OPERABLE_RECOVERY: Recovery = Recovery {
    mutability: Mutability::Fact,
    ahead: false,
};

#[test]
fn a_detached_authored_successor_commits_its_owners_when_completed() {
    let (mut world, owner, target) = hovered_online(UiChangeProfile::platform_pulse());
    let predecessor = world.session.active_generation_identity();
    let work = SuccessionWork::read(&world.session);

    let plan = observe(
        &mut world,
        status(&owner, "OFFLINE", 2),
        Some(OPERABLE_RECOVERY),
        6,
    );
    let pending = in_flight(&mut world, plan, 6);
    assert_eq!(
        world.session.active_generation_identity(),
        predecessor,
        "nothing commits while the presentation is in flight"
    );
    assert!(matches!(
        pending.complete(&mut world.session, 7),
        UiRebindOutcome::Published(_)
    ));

    assert_ne!(world.session.active_generation_identity(), predecessor);
    assert_eq!(
        super::succession_tests::hovered_family(&world, target).0,
        crate::declaration::UiPointerAffordance::Activation
    );
    assert_pointer_is_fresh(&world.session, true);
    assert_standing_is_fresh(&world.session, 1);
    assert_owners_follow(&world.session, true);
    assert_eq!(
        work.since(&world.session),
        SuccessionWork {
            reobservations: 2,
            operand_probes: 6,
            index_hits: 1,
            evaluations: 2,
            appearance_batches: 1,
        },
        "the observation turn, the preparation, and the detached completion (W2)"
    );
    let _ = world.session.shutdown();
}

#[test]
#[ignore = "detached completion commits pre-drift successor owners; fixed by the typed reattach (cleanup step 6)"]
fn a_fact_updated_while_detached_is_current_in_the_committed_pointer() {
    let (mut world, owner, target) = hovered_online(UiChangeProfile::platform_pulse());
    let plan = observe(
        &mut world,
        status(&owner, "OFFLINE", 2),
        Some(OPERABLE_RECOVERY),
        6,
    );
    let pending = in_flight(&mut world, plan, 6);

    set(&mut world.session, fixture::POLICY, false);
    assert!(matches!(
        pending.complete(&mut world.session, 7),
        UiRebindOutcome::Published(_)
    ));

    assert_eq!(
        standing(&world.session, world.graph, target)
            .decision()
            .policy(),
        UiIntentPolicyPosture::Denied,
        "the update denies the policy the hovered consumer's intent reads"
    );
    let rows = pointer_rows(&world.session);
    assert_eq!(rows.len(), 1, "the pointer hovers the one consumer");
    let fresh = rows[0].1.as_ref().expect("the consumer is observable");
    assert!(!fresh.3, "a denied policy leaves nothing to activate");
    assert_pointer_is_fresh(&world.session, true);
    assert_standing_is_fresh(&world.session, 1);
    let _ = world.session.shutdown();
}
