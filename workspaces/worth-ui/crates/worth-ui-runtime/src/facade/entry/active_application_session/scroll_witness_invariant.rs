//! The invariant every commit leaves true, asserted in debug builds.
//!
//! Every publication settles its Scroll extent through
//! `settle_presented_scroll_extent`, every committed Motion tick settles its
//! accepted samples, and every Motion commit installs its track. Each then
//! asserts that displayed geometry stands on each surface's latest witness:
//!
//! - Each Scroll content sample Motion shows is displayed by that witness.
//!   Unless a deferral owes its settle, the owner's offset stands where the
//!   witness displayed it, and so does the mounted pose, unless geometry no
//!   witness has shown is staged over it: a layout, or a direct placement
//!   awaiting its publication.
//! - Every Scroll region's offset and mounted pose hold one pose, under the
//!   same staging exception.
//! - On every presented surface, both hit-test lanes hold the same rows at
//!   the same rects: the retained frame's rows moved by the on-screen Motion
//!   samples and the committed poses, and the presented hit index. A
//!   publication that displays an entrance holds it in the hit index before
//!   its Motion commit installs the sample, so that surface's lanes are
//!   asserted once the commit installs.
//! - On every presented surface, interaction reads each row where the host
//!   shows what its instance paints, and no part of it outside the clip the
//!   host shows that paint through.
//! - Each hovering pointer's target is what its position resolves to now.
//!
//! A surface whose last settle was refused reports that in its disposition.
//! Its Scroll poses are not asserted until a settle is paid there.

use std::collections::BTreeSet;

use super::scroll_accepted_sample_settlement::UiScrollSettlementReading;
use super::scroll_settle_retry::UiOwedScrollSettles;
use crate::mounting::WorthUiMountedSessionState;
use crate::runtime::interaction::UiInteractionRuntimeState;
use crate::runtime::scroll::{UiScrollOwnerIdentity, UiScrollRuntimeState};
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

/// Assert, in debug builds, that displayed Scroll, hit-test, and cursor
/// geometry stand on the latest witness. Called where a witness commits.
pub(in crate::facade::entry) fn debug_assert_scroll_geometry_witnessed(
    reading: UiScrollSettlementReading<'_>,
    interaction: &UiInteractionRuntimeState,
    settles: &UiOwedScrollSettles,
) {
    if !cfg!(debug_assertions) {
        return;
    }
    assert_samples_displayed(reading, settles);
    if let Some(scroll) = reading.scroll {
        assert_regions_hold_one_pose(reading.mounted, scroll, settles);
    }
    for surface in reading.mounted.current_surfaces() {
        assert_hit_lanes_agree(reading.mounted, surface);
        assert_paint_and_hit_agree(reading.mounted, surface);
    }
    let stale = interaction.stale_pointer_targets(reading.mounted);
    assert!(
        stale.is_empty(),
        "a pointer hovers what the latest witness no longer shows under it (recorded, resolved): {stale:?}"
    );
}

/// Geometry no witness has shown is staged over `surface`.
fn staged(scroll: &UiScrollRuntimeState, surface: UiSemanticSurfaceIdentity) -> bool {
    scroll.has_unpresented_layout(surface) || scroll.has_pending_direct(surface)
}

fn assert_samples_displayed(reading: UiScrollSettlementReading<'_>, settles: &UiOwedScrollSettles) {
    for (target, _) in reading.mounted.accepted_scroll_group_samples() {
        let surface = target.semantic_surface();
        if reading
            .mounted
            .current_presentation_for_surface(surface)
            .is_none()
        {
            continue;
        }
        assert!(
            reading.displayed_sample(target, surface).is_some(),
            "Motion shows a Scroll sample the latest witness of {surface:?} does not display"
        );
        if settles.owes(surface) || settles.refuses(surface) {
            continue;
        }
        let (Some(displayed), Ok(Some(owner)), Some(scroll)) = (
            reading.displayed_offset(target, surface),
            reading.owner(target, surface),
            reading.scroll,
        ) else {
            continue;
        };
        let offset = scroll.offset(owner.entry.owner(), owner.entry.incarnation());
        assert!(
            offset.is_ok_and(|offset| displayed.stands_at(offset)),
            "Scroll holds {offset:?} while the latest witness of {surface:?} displays {displayed:?}"
        );
        if staged(scroll, surface) {
            continue;
        }
        let mounted = reading
            .mounted
            .mounted_scroll_pose(owner.region_instance, owner.owner_instance);
        assert!(
            mounted.is_some_and(|mounted| displayed.stands_at(mounted)),
            "mounted geometry holds {mounted:?} while the latest witness of {surface:?} displays {displayed:?}"
        );
    }
}

/// Assert that each region owner on a presented surface holds one pose in
/// Scroll and mounted geometry.
fn assert_regions_hold_one_pose(
    mounted: &WorthUiMountedSessionState,
    scroll: &UiScrollRuntimeState,
    settles: &UiOwedScrollSettles,
) {
    let mut owners = BTreeSet::new();
    for instance in scroll.ownership_instances() {
        let Ok(chain) = scroll.ownership_chain(instance) else {
            continue;
        };
        for (slot, owner) in chain.owners().iter().copied().enumerate() {
            let UiScrollOwnerIdentity::Region { surface, .. } = owner else {
                continue;
            };
            if !owners.insert(owner) || mounted.current_presentation_for_surface(surface).is_none()
            {
                continue;
            }
            if settles.refuses(surface) || staged(scroll, surface) {
                continue;
            }
            let (Some((owner_instance, _, _)), Some(incarnation)) = (
                mounted.scroll_region_geometry(instance, slot),
                mounted.scroll_region_incarnation(instance, slot),
            ) else {
                continue;
            };
            let (Ok(offset), Some(pose)) = (
                scroll.offset(owner, incarnation),
                mounted.mounted_scroll_pose(instance, owner_instance),
            ) else {
                continue;
            };
            assert_eq!(
                offset, pose,
                "Scroll and mounted geometry hold one pose for {owner:?}"
            );
        }
    }
}

/// Assert that interaction reads every row the presented hit index holds,
/// and only those, where the index holds it.
fn assert_hit_lanes_agree(
    mounted: &WorthUiMountedSessionState,
    surface: UiSemanticSurfaceIdentity,
) {
    let Some(displayed) = mounted.current_presentation_for_surface(surface) else {
        return;
    };
    let parted = mounted.parted_hit_lanes(displayed.basis());
    assert!(
        parted.is_empty(),
        "the hit-test lanes of {surface:?} part (instance, interaction, index): {parted:?}"
    );
}

/// Assert that interaction reads each row of `surface` where the host shows
/// what its instance paints.
fn assert_paint_and_hit_agree(
    mounted: &WorthUiMountedSessionState,
    surface: UiSemanticSurfaceIdentity,
) {
    let parted = mounted.parted_paint_and_hit(surface);
    assert!(
        parted.is_empty(),
        "paint and hit testing part on {surface:?}: {parted:?}"
    );
}
