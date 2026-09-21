//! Which owner in a resolved chain a gesture latches to.
//!
//! The latch decides who owns the *rest* of one physical gesture, so the
//! question it answers is about capacity rather than about what moved. An
//! inner region with room keeps the gesture that started on it; the same
//! region at its edge hands the gesture to the ancestor the reader will watch
//! move; a chain with nothing left anywhere names no owner at all.
//!
//! The last case is the one worth stating separately. A declared smooth wheel
//! routes a zero delta, and a route stops at the first owner that leaves
//! nothing over, so the route visits one owner and the chain has two. The
//! latch is judged from the chain, which is why that notch still finds the
//! ancestor.

use super::*;

/// How far one owner can travel on the block axis.
const INNER_EXTENT: i64 = 100;
const OUTER_EXTENT: i64 = 500;
/// One notch, already turned into the offset direction an observation routes.
const NOTCH: i64 = 35;
/// Somewhere in the middle of the inner content, with room either way.
const MID_CONTENT: i64 = 40;

/// A block-scrolling region nested inside the viewport behind it, resolved the
/// way an observation resolves one: the chain in order, the bounds the route
/// reconciles against, and the offsets the chain holds before anything moves.
struct Nested {
    state: UiScrollRuntimeState,
    entries: Vec<UiScrollChainEntry>,
    offsets: Vec<UiScrollOffset>,
    bounds: Vec<UiScrollBounds>,
}

fn surface() -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
    worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface identity")
}

fn incarnation(value: u64) -> UiScrollOwnerIncarnation {
    UiScrollOwnerIncarnation::new(value).expect("nonzero incarnation")
}

fn bounds(block: i64) -> UiScrollBounds {
    UiScrollBounds::new(0, block).expect("non-negative bounds")
}

fn offset(block: i64) -> UiScrollOffset {
    UiScrollOffset::new(0, block).expect("in-range offset")
}

fn host_cause() -> UiScrollDeltaCause {
    UiScrollDeltaCause::Host {
        source: worth_ui_host_contract::UiHostScrollDeltaSource::PointerWheel,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
        precision: worth_ui_host_contract::UiHostScrollDeltaPrecision::Pixel,
    }
}

fn nested(inner_offset: i64, outer_offset: i64) -> Nested {
    let surface = surface();
    let inner =
        UiScrollOwnerIdentity::region(surface, crate::graph::UiGraphNodeIdentity::new(31), 3);
    let outer = UiScrollOwnerIdentity::viewport(surface);
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    let plan = [
        (inner, 1, INNER_EXTENT, inner_offset),
        (outer, 2, OUTER_EXTENT, outer_offset),
    ];
    for (owner, generation, extent, held) in plan {
        state
            .register(UiScrollOwnerRegistration::new(
                owner,
                incarnation(generation),
                UiScrollAxes::Block,
                bounds(extent),
                offset(held),
            ))
            .expect("registration");
    }
    let entries = plan
        .iter()
        .map(|(owner, generation, _, _)| UiScrollChainEntry::new(*owner, incarnation(*generation)))
        .collect();
    let offsets = plan
        .iter()
        .map(|(owner, generation, _, _)| {
            state
                .offset(*owner, incarnation(*generation))
                .expect("a registered owner holds an offset")
        })
        .collect();
    Nested {
        state,
        entries,
        offsets,
        bounds: vec![bounds(INNER_EXTENT), bounds(OUTER_EXTENT)],
    }
}

/// The question the observation asks, with the block travel one notch names.
fn latch(chain: &Nested, block_delta: i64) -> Option<usize> {
    latching_chain_index(
        &chain.offsets,
        &chain.bounds,
        UiScrollDelta::new(0, block_delta),
    )
}

/// A region the reader aimed at, with content left to show, keeps its gesture.
#[test]
fn an_inner_owner_with_room_keeps_the_gesture_that_started_on_it() {
    assert_eq!(latch(&nested(MID_CONTENT, 0), NOTCH), Some(0));
}

/// The same region at the end of its content is not the owner that will move,
/// so the gesture belongs to the first ancestor that can take it.
#[test]
fn a_gesture_starting_at_an_inner_edge_latches_to_the_ancestor_that_can_move() {
    assert_eq!(latch(&nested(INNER_EXTENT, 0), NOTCH), Some(1));
}

/// An edge is an edge in one direction only. The same chain, pushed back the
/// way it came, is answered by the region under the pointer.
#[test]
fn the_same_edge_answers_the_other_direction_at_the_region_itself() {
    assert_eq!(latch(&nested(INNER_EXTENT, 0), -NOTCH), Some(0));
}

/// Nothing in the chain can show the reader anything further, so no owner is
/// named and the gesture latches to nothing.
#[test]
fn a_chain_with_no_travel_left_names_no_owner() {
    assert_eq!(latch(&nested(INNER_EXTENT, OUTER_EXTENT), NOTCH), None);
}

/// A declared smooth wheel routes no delta at all. The route is finished at
/// the first owner and never reaches the ancestor, so an answer taken from the
/// route would pin the notch to a region that cannot move. The chain answers.
#[test]
fn a_smooth_notch_routes_one_owner_and_still_latches_to_the_one_with_room() {
    let mut chain = nested(INNER_EXTENT, 0);
    let receipt = chain
        .state
        .route(
            UiScrollDeltaRequest::new(
                chain.entries.clone(),
                UiScrollDelta::new(0, 0),
                host_cause(),
            )
            .expect("a zero delta over a two-owner chain"),
        )
        .expect("a zero delta routes");

    assert_eq!(
        receipt.owners_visited(),
        1,
        "a delta with nothing left over is finished at the owner it reached"
    );
    assert_eq!(latch(&chain, NOTCH), Some(1));
}
