//! Facade-level proof that scrolling one region reaches that region's content
//! and stops there.
//!
//! The reader is never scrolling the whole application. They are scrolling one
//! region, and what moves is what that region carries. A runtime that answered
//! a notch by re-selecting every mounted occurrence, or by walking every
//! surface it has published, would still put the right pixels on screen -- and
//! would pay for a wheel in proportion to how big the application is rather
//! than how much of it moved. That is the cost a reader feels as a wheel that
//! gets slower the more there is on screen.
//!
//! The World here is the one the scrolling scenarios share, and it is laid out
//! so that every kind of neighbour exists at once. Two occurrences sit inside
//! the first component's scrollable region and travel with it. One sits beside
//! it on the same surface and does not. One sits on a second surface entirely.
//! The region's own occurrence is the fifth: it is what the content moves
//! inside of, and it does not move itself.
//!
//! So a single notch has four chances to reach somewhere it should not, and
//! what it reaches is read off the frame the session prepares afterwards
//! rather than off a counter written for this. The appearance invalidation
//! batch names the occurrences a frame has to re-resolve; the selection cost
//! report counts them; and the second surface is asked separately whether it
//! has anything to publish at all.

use super::scroll_pose_authority::ScrollWorld;
use super::session::World;
use crate::runtime::scroll::UiHostScrollObservationOutcome;
use worth_ui_host_contract::*;

/// How far the notch carries the content. Enough that every occupant of the
/// region moves, so an occurrence left behind is a narrowing that went too far
/// rather than one that had nothing to do.
const TRAVEL_POINTS: i64 = 10;

/// The region's own occurrence. The content travels inside it; it stays where
/// the layout put it.
const REGION_OWNER: usize = 0;
/// A component laid out against the surface beside the scrollable one. Nothing
/// about it moved.
const UNRELATED_NEIGHBOUR: usize = 1;
/// A component laid out against the second surface, which this notch never
/// touched.
const OTHER_SURFACE: usize = 3;
/// The two occurrences laid out inside the region: the content this World
/// nests there for the scrolling scenarios, and the child occurrence it nests
/// there by default.
const CARRIED: [usize; 2] = [2, 4];

fn scrolled() -> ScrollWorld {
    ScrollWorld::publish_with_nested_content(World::launch())
}

/// One notch over the scrollable region, as a pointer resting on it produces.
fn notch(scroll: &mut ScrollWorld) -> UiHostScrollObservationOutcome {
    let target = scroll.pointer_target();
    scroll.targeted_wheel(
        UiHostScrollDeltaPhase::Updated,
        target,
        UiHostScrollDeltaPrecision::Pixel,
        -TRAVEL_POINTS * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
        5,
    )
}

/// A local scroll re-resolves the occurrences it moved and no others, and the
/// ones it moved are the ones the region carries.
#[test]
fn a_notch_re_resolves_the_content_its_region_carries_and_nothing_beside_it() {
    let mut scroll = scrolled();
    let mounted = scroll.world.instances.len();
    let owners = scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("the World installs Scroll from policy")
        .owner_count();
    assert_eq!(
        owners, mounted,
        "every occurrence in this World registers a Scroll owner, so a route \
         that walked them all would have somewhere to go"
    );

    let at_rest = scroll.world.prepare_surface(scroll.surface());
    assert_eq!(
        at_rest
            .appearance_selection_cost_report()
            .selected_instance_count(),
        0,
        "nothing is owed before the notch, so what is owed after it is the notch"
    );
    drop(at_rest);

    let UiHostScrollObservationOutcome::Applied(receipt) = notch(&mut scroll) else {
        panic!("a wheel over the scrollable region applies")
    };
    assert_eq!(
        receipt.owners_visited(),
        1,
        "the notch was spent where it landed, so nothing was handed outward to \
         an owner further up the chain"
    );

    let prepared = scroll.world.prepare_surface(scroll.surface());
    let cost = prepared.appearance_selection_cost_report();
    assert_eq!(
        cost.selected_instance_count(),
        CARRIED.len(),
        "two occurrences travelled, so two are re-resolved -- not the {mounted} \
         this session has mounted"
    );
    assert_eq!(
        cost.index_entries_touched(),
        CARRIED.len(),
        "and the index was read for those two alone"
    );

    let batch = prepared
        .appearance_invalidation_batch()
        .expect("a moved occurrence leaves the surface something to re-resolve");
    let reached = |position: usize| {
        batch
            .mounted_consumers()
            .iter()
            .any(|(_, instance)| *instance == scroll.world.instances[position])
    };
    for position in CARRIED {
        assert!(
            reached(position),
            "the occurrence at {position} travelled with the region, so the frame owes it"
        );
    }
    assert!(
        !reached(REGION_OWNER),
        "the region's own occurrence did not move, so its content moving is not \
         a reason to re-resolve it"
    );
    assert!(
        !reached(UNRELATED_NEIGHBOUR),
        "the component beside the region is not in it"
    );
    assert!(
        !reached(OTHER_SURFACE),
        "and the component on the other surface is further away still"
    );
    assert!(
        batch.is_physical_input_only() && batch.graph_consumers().is_empty(),
        "a notch moves content; it does not make anything mean something else, \
         so nothing is asked to resolve semantically"
    );
    drop(prepared);

    let work = scroll.world.session.last_scroll_hit_index_work();
    assert_eq!(
        work.scroll_rows_displaced(),
        0,
        "preparing direct geometry must not displace the accepted hit index"
    );
    let _ = scroll.world.session.shutdown();
}

/// How many occurrences a freshly prepared frame for `surface` has to
/// re-resolve.
fn selected(scroll: &mut ScrollWorld, surface: UiSemanticSurfaceIdentity) -> usize {
    let prepared = scroll.world.prepare_surface(surface);
    let count = prepared
        .appearance_selection_cost_report()
        .selected_instance_count();
    drop(prepared);
    count
}

/// A pose is prepared against one surface's geometry and says nothing about
/// any other. The second surface is asked before and after the notch and
/// answers the same both times, while the surface that scrolled answers
/// differently -- so the question is one the notch could have changed.
#[test]
fn a_notch_on_one_surface_owes_the_other_surface_nothing() {
    let mut scroll = scrolled();
    let scrolled_surface = scroll.surface();
    let other = scroll.world.surfaces[1];
    assert_ne!(
        other, scrolled_surface,
        "the World publishes two surfaces, so there is another one to leave alone"
    );
    assert_eq!(
        selected(&mut scroll, other),
        0,
        "the second surface owes nothing before the notch"
    );

    assert!(matches!(
        notch(&mut scroll),
        UiHostScrollObservationOutcome::Applied(_)
    ));

    assert_eq!(
        selected(&mut scroll, other),
        0,
        "and nothing after it: a pose is prepared against the geometry of the \
         surface it scrolled and says nothing about any other"
    );
    assert_eq!(
        selected(&mut scroll, scrolled_surface),
        CARRIED.len(),
        "while the surface that did scroll owes the occurrences that moved, so \
         the question the second surface answered was one this notch could \
         have changed"
    );
    let _ = scroll.world.session.shutdown();
}

#[test]
fn accepted_samples_visit_only_indexed_content_and_its_owned_regions() {
    fn present_and_inspect(
        scroll: &mut super::scroll_pose_authority::ScrollWorld,
        tick: u64,
    ) -> super::super::super::UiScrollSettleDisposition {
        let basis = scroll.presentation();
        let prepared = scroll
            .world
            .session
            .prepare_motion_tick(tick, basis)
            .expect("armed settle prepares its tick");
        if !prepared.receipt().samples().is_empty() {
            scroll.world.host.push_native_display_presented();
        }
        scroll
            .world
            .session
            .present_prepared_motion_tick(prepared, basis);
        // Completion now reconciles accepted Scroll geometry before returning.
        // A second explicit settle would overwrite this frame's locality work
        // with the equal-pose (zero-work) retry.
        scroll.world.session.last_scroll_settle_disposition()
    }

    let mut scroll = super::scroll_settle_commit::smooth_world(true);
    let declaration = scroll
        .world
        .session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings()
        .surface_named("workspace.surface.overlay")
        .unwrap();
    let carried_regions: usize = CARRIED
        .iter()
        .map(|position| {
            let node = scroll
                .world
                .session
                .mounted
                .current_mounted_identity_basis(scroll.world.instances[*position])
                .unwrap()
                .graph_node_identity();
            scroll
                .world
                .session
                .application
                .mounted_region_declarations(declaration, node)
                .0
                .len()
        })
        .sum();
    assert!(carried_regions > 0);
    let owner_bounds = scroll
        .world
        .session
        .mounted
        .interaction_hit_test_basis(scroll.presentation())
        .unwrap()
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == scroll.target())
        .unwrap()
        .bounds();
    assert!(matches!(
        scroll.wheel(
            super::scroll_settle_commit::ONE_NOTCH,
            super::scroll_settle_commit::one_notch_up(),
            5
        ),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    // The first accepted sample establishes Motion's unchanged rest pose;
    // locality must not manufacture descendant work when nothing moved.
    present_and_inspect(&mut scroll, 6);
    let rest_work = scroll.world.session.last_scroll_hit_index_work();
    assert_eq!(rest_work.scroll_geometry_members_visited(), 0);
    assert_eq!(rest_work.scroll_geometry_regions_visited(), 0);
    for tick in [7, 8] {
        assert_eq!(
            present_and_inspect(&mut scroll, tick),
            super::super::super::UiScrollSettleDisposition::Applied
        );
        let work = scroll.world.session.last_scroll_hit_index_work();
        assert_eq!(work.scroll_geometry_members_visited(), CARRIED.len());
        assert_eq!(work.scroll_geometry_regions_visited(), carried_regions,
            "only carried occurrences' declared regions are visited; siblings and stationary owners are not scanned");
        assert_eq!(
            scroll
                .world
                .session
                .mounted
                .interaction_hit_test_basis(scroll.presentation())
                .unwrap()
                .rows()
                .iter()
                .find(|row| row.mounted_instance() == scroll.target())
                .unwrap()
                .bounds(),
            owner_bounds
        );
    }
    let _ = scroll.world.session.shutdown();
}
