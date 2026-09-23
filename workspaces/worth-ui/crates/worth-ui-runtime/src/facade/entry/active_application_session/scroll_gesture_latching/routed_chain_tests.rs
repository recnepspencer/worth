//! Which owner a position in a routed chain names.
//!
//! A gesture latches to a position, and the position is spent looking an owner
//! up. There are two lists it could be spent on and they are not the same
//! length. The chain names every owner the gesture could reach. A route
//! receipt names only the owners that had travel left to visit, and a route
//! stops at the first owner that leaves nothing over -- so a zero delta
//! produces one transition whatever the chain's depth, and a declared smooth
//! wheel routes exactly a zero delta.
//!
//! That is the case the latch exists for: an inner region at its edge hands
//! the gesture to the ancestor that can move, which is position one of a chain
//! the receipt stops at position zero. Reading the receipt there finds nothing
//! and refuses a notch that should have scrolled. So the chain answers, and
//! the answer carries the owner rather than the position, which is what leaves
//! no second list for a later caller to reach for.

use super::UiScrollRoutedChain;
use crate::runtime::scroll::{
    UiScrollAxes, UiScrollBounds, UiScrollChainEntry, UiScrollDelta, UiScrollDeltaCause,
    UiScrollDeltaRequest, UiScrollOffset, UiScrollOwnerIdentity, UiScrollOwnerIncarnation,
    UiScrollOwnerRegistration, UiScrollRuntimeState,
};

/// The inner region sits at the end of its content, so a notch downward
/// belongs to the ancestor behind it.
const INNER_EXTENT: i64 = 100;
const OUTER_EXTENT: i64 = 500;
/// The geometry slots the two owners read their boxes from. They are distinct
/// so a test cannot pass by reading the wrong one, and neither equals the
/// chain position that carries it, so one cannot pass by reading the position
/// either.
const INNER_SLOT: usize = 2;
const OUTER_SLOT: usize = 5;

fn incarnation(value: u64) -> UiScrollOwnerIncarnation {
    UiScrollOwnerIncarnation::new(value).expect("nonzero incarnation")
}

fn bounds(block: i64) -> UiScrollBounds {
    UiScrollBounds::new(0, block).expect("non-negative bounds")
}

/// A block-scrolling region nested inside the viewport behind it, resolved the
/// way an observation resolves one, together with the Scroll state that holds
/// the same two owners.
fn nested() -> (UiScrollRoutedChain, UiScrollRuntimeState) {
    let surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface");
    let inner =
        UiScrollOwnerIdentity::region(surface, crate::graph::UiGraphNodeIdentity::new(31), 3);
    let outer = UiScrollOwnerIdentity::viewport(surface);
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    for (owner, generation, extent, held) in [
        (inner, 1_u64, INNER_EXTENT, INNER_EXTENT),
        (outer, 2, OUTER_EXTENT, 0),
    ] {
        state
            .register(UiScrollOwnerRegistration::new(
                owner,
                incarnation(generation),
                UiScrollAxes::Block,
                bounds(extent),
                UiScrollOffset::new(0, held).expect("in-range offset"),
            ))
            .expect("registration");
    }
    let chain = UiScrollRoutedChain {
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound()
            .expect("mounted instance"),
        graph_node: crate::graph::UiGraphNodeIdentity::new(7),
        entries: vec![
            UiScrollChainEntry::new(inner, incarnation(1)),
            UiScrollChainEntry::new(outer, incarnation(2)),
        ],
        slots: vec![INNER_SLOT, OUTER_SLOT],
    };
    (chain, state)
}

/// Each position names the owner occurrence the chain holds there, and the
/// geometry slot that owner reads its boxes from.
///
/// The two lists this pairs are held separately, so the answer is checked
/// against what the World independently is rather than against either list:
/// the inner owner is a region and the outer one is the viewport behind it,
/// and the slots are neither equal to each other nor to the positions holding
/// them. A pairing that transposed the lists, or that answered a slot from the
/// position it was asked about, would satisfy neither.
#[test]
fn a_position_names_the_owner_the_chain_holds_there() {
    let (chain, _) = nested();

    let inner = chain
        .region(0)
        .expect("a chain of two reaches position zero");
    let outer = chain
        .region(1)
        .expect("a chain of two reaches position one");
    assert!(
        matches!(inner.entry().owner(), UiScrollOwnerIdentity::Region { .. }),
        "position zero is the nested region: {:?}",
        inner.entry().owner()
    );
    assert_eq!(inner.slot(), INNER_SLOT);
    assert!(
        matches!(outer.entry().owner(), UiScrollOwnerIdentity::Viewport(_)),
        "position one is the viewport the region is nested in: {:?}",
        outer.entry().owner()
    );
    assert_eq!(outer.slot(), OUTER_SLOT);
    assert_ne!(
        inner.entry().incarnation(),
        outer.entry().incarnation(),
        "the two positions name different owner occurrences"
    );
}

/// A position past the end of the chain names nothing, rather than the last
/// owner or the first.
#[test]
fn a_position_the_chain_does_not_reach_names_nothing() {
    let (chain, _) = nested();
    assert!(chain.region(2).is_none());
}

/// The regression this is here for. The zero delta a declared smooth wheel
/// routes produces one transition across a chain of two, and the owner the
/// gesture belongs to is the one at position one. The chain still answers for
/// it; the receipt has no position one to answer with.
#[test]
fn the_outer_owner_a_smooth_notch_latches_to_is_absent_from_its_receipt() {
    let (chain, mut state) = nested();

    let receipt = state
        .route(
            UiScrollDeltaRequest::new(
                chain.entries().to_vec(),
                UiScrollDelta::new(0, 0),
                UiScrollDeltaCause::Host {
                    source: worth_ui_host_contract::UiHostScrollDeltaSource::PointerWheel,
                    phase: worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
                    precision: worth_ui_host_contract::UiHostScrollDeltaPrecision::Line {
                        platform_lines_per_notch: 1,
                        basis: worth_ui_host_contract::UiHostScrollLineCountBasis::PlatformReported,
                    },
                },
            )
            .expect("a two-owner chain is a routable request"),
        )
        .expect("a zero delta routes");

    assert_eq!(
        receipt.owners_visited(),
        1,
        "a zero delta leaves nothing over, so the route stops at the first owner"
    );
    assert_eq!(receipt.transitions().len(), 1);
    let latched = chain
        .region(1)
        .expect("the chain reaches the ancestor with room");
    assert_eq!(latched.entry(), chain.entries()[1]);
    assert!(
        state
            .offset(latched.entry().owner(), latched.entry().incarnation())
            .is_ok(),
        "the routed state holds the latched owner, so its settle has an offset to start from"
    );
}
