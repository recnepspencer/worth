//! A branch that admits few product observations makes room for its callers
//! from the observations nothing holds.

use super::*;

/// A caller that only writes cannot fill a branch for good. A performed write
/// keeps the observation its source was read at for its required outputs,
/// and no later write replaces it: every body here is a root of its own.
/// More such writes than the branch admits observations are all performed.
/// Every output is then demanded and settles on the model in one advance,
/// and a root is written and demanded once more.
fn a_write_only_caller_leaves_room(starts_outputs: bool) {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_observing(None, ring_world::seed::<3>, Retained::AMPLE, 16);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f00);
    let at = "a write-only caller";
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    for index in 3..8 {
        court.create_ring(index, at);
        rings.push(Ring::created(index));
    }
    take_all_decisions();
    let write = |body: &str, y| {
        if starts_outputs {
            court.write_y_and_abandon_its_outputs(body, y, at);
        } else {
            court.write_y(body, y, at);
        }
    };
    for ring in &mut rings {
        (ring.a_y, ring.successor_y, ring.far_y) = (2, 2, 11);
        write(&ring.key("a"), ring.a_y);
        write(&ring.successor, ring.successor_y);
        write(&ring.key("source-c"), ring.far_y);
    }
    let at = "after a write-only caller";
    for index in 0..rings.len() {
        court.demand_ring(&mut rings, index, at);
    }
    rings[1].a_y = 5;
    court.write_y(&rings[1].key("a"), 5, at);
    court.demand_ring(&mut rings, 1, at);
}

/// The caller drops each write: its required outputs never start.
#[test]
fn writes_whose_outputs_never_start_leave_room_for_a_demand_and_another_write() {
    a_write_only_caller_leaves_room(false);
}

/// The caller starts each write's required outputs and drops them before
/// they advance. Each output keeps its obligation and is produced from the
/// source its next demand discloses.
#[test]
fn writes_whose_started_outputs_never_advance_leave_room_for_a_demand_and_another_write() {
    a_write_only_caller_leaves_room(true);
}
