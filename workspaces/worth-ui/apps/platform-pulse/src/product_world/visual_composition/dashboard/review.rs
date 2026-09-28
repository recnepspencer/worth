//! The deployment review modal: a card that holds its concept height where
//! the viewport has room and gives way to its insets where it does not.
//!
//! The card lays its content out in three rows: the heading, the changes, and
//! the actions. The heading and the actions keep their concept heights, and
//! the changes take what is left, down to a minimum. The changes list is
//! authored at its concept extent and scrolls inside its row, so a card
//! shorter than the concept keeps every change reachable. The shadow keeps
//! its concept margin around the card at every height.
use worth_ui::facade::declaration::{
    ComponentViewportAxisPlacement, ComponentViewportRegion, MosaicLayoutCell,
    MosaicLayoutContract, MosaicTrack,
};

use super::frame::{Edge, Frame};
use super::{
    surface, text, ContainerTracks, DashboardElement, DashboardGraphic, DashboardPlacement,
    DashboardScrollOwner,
};

#[cfg(test)]
mod tests;

/// The Portal owner the modal opens from.
const OWNER: &str = "review_target";
/// The card, which lays out the modal's rows.
const CARD: &str = "review_card";
/// The scrolled changes list.
pub(super) const LIST: &str = "review_list";
/// Where the concept draws the card, and each of its rows.
const CARD_RECT: [u16; 4] = [36, 36, 521, 444];
const HEADING_ROW: [u16; 4] = [36, 36, 521, 141];
const CHANGES_ROW: [u16; 4] = [36, 177, 521, 206];
const ACTIONS_ROW: [u16; 4] = [36, 383, 521, 97];
/// Where the concept draws the changes list's content.
const LIST_RECT: [u16; 4] = [67, 177, 460, 206];
/// The fewest points the changes row keeps.
const CHANGES_MINIMUM: u16 = 48;
/// The shadow's margin around the card on every side.
const SHADOW_MARGIN: u16 = 36;

pub(super) fn elements() -> Vec<DashboardElement> {
    let heading = Frame::cell(CARD, MosaicLayoutCell::at(0, 0), HEADING_ROW);
    let actions = Frame::cell(CARD, MosaicLayoutCell::at(0, 2), ACTIONS_ROW);
    let list = Frame::cell(LIST, MosaicLayoutCell::at(0, 0), LIST_RECT);
    let card = Frame::cell(
        CARD,
        MosaicLayoutCell::spanning(0, 0, 1, 3).expect("the surface spans the card's rows"),
        CARD_RECT,
    );
    let mut shadow = surface("review_shadow", [0, 0, 593, 516], "shadow", 12, false, 1)
        .graphic(DashboardGraphic::Shadow);
    shadow.placement = DashboardPlacement::Viewport(shadow_region());
    let start = |frame: Frame, element| frame.place(element, Edge::Start, Edge::Start);
    vec![
        shadow,
        card.place(surface("review_surface", CARD_RECT, "raised_surface", 16, false, 2), Edge::Both, Edge::Both),
        start(heading, text("review_title", "Review deployment", [67, 66, 431, 43], 26, true, "primary_text")),
        start(heading, text("review_body", "You’re about to deploy a new version to production. Please review the changes below before proceeding.", [67, 114, 461, 51], 15, false, "secondary_text").wrapped()),
        start(heading, surface("review_close_target", [505, 58, 36, 36], "raised_surface", 8, false, 3).action("close")),
        start(heading, surface("review_close_icon", [516, 69, 14, 14], "secondary_text", 12, false, 4).graphic(DashboardGraphic::Close)),
        changes_row().place(surface("review_changes", LIST_RECT, "inset_fill", 12, false, 3), Edge::Start, Edge::Both),
        start(list, surface("change_bg_0", [83, 197, 32, 32], "mint_pale", 32, false, 4)),
        start(list, surface("change_icon_0", [91, 205, 16, 16], "positive", 12, false, 5).graphic(DashboardGraphic::Plus)),
        start(list, text("change_title_0", "Update pricing model", [133, 195, 318, 25], 13, false, "primary_text")),
        start(list, text("change_body_0", "Modifies subscription tiers and billing logic", [133, 216, 368, 23], 12, false, "secondary_text")),
        start(list, text("change_files_0", "3 files", [465, 205, 58, 22], 12, false, "secondary_text")),
        start(list, surface("change_rule_0", [133, 248, 378, 1], "grid", 0, false, 4)),
        start(list, surface("change_bg_1", [83, 264, 32, 32], "lavender_pale", 32, false, 4)),
        start(list, surface("change_icon_1", [91, 272, 16, 16], "principal_accent", 12, false, 5).graphic(DashboardGraphic::Pencil)),
        start(list, text("change_title_1", "Improve error handling", [133, 262, 318, 25], 13, false, "primary_text")),
        start(list, text("change_body_1", "Adds retries and better logging", [133, 283, 368, 23], 12, false, "secondary_text")),
        start(list, text("change_files_1", "5 files", [465, 272, 58, 22], 12, false, "secondary_text")),
        start(list, surface("change_rule_1", [133, 315, 378, 1], "grid", 0, false, 4)),
        start(list, surface("change_bg_2", [83, 331, 32, 32], "coral_pale", 32, false, 4)),
        start(list, surface("change_icon_2", [91, 339, 16, 16], "negative", 12, false, 5).graphic(DashboardGraphic::Minus)),
        start(list, text("change_title_2", "Remove legacy feature", [133, 329, 318, 25], 13, false, "primary_text")),
        start(list, text("change_body_2", "Deprecates the old recommendations endpoint", [133, 350, 368, 23], 12, false, "secondary_text")),
        start(list, text("change_files_2", "2 files", [465, 339, 58, 22], 12, false, "secondary_text")),
        start(actions, surface("review_cancel_target", [195, 403, 109, 48], "raised_surface", 10, true, 3).action("close")),
        start(actions, text("review_cancel_label", "Cancel", [195, 403, 109, 48], 13, false, "primary_text").centered()),
        start(actions, surface("review_primary_target", [316, 403, 211, 48], "action_fill", 10, false, 3).action("approve")),
        start(actions, text("review_primary_label", "Approve deployment", [316, 403, 211, 48], 13, false, "action_text").centered()),
    ]
    .into_iter()
    .map(|element| element.portal(OWNER))
    .collect()
}

/// The card and the changes list it scrolls.
pub(super) fn containers() -> Vec<ContainerTracks> {
    let [x, y, width, height] = CARD_RECT;
    let track = |track: Result<MosaicTrack, _>| track.expect("the card declares valid tracks");
    let card = ContainerTracks {
        id: CARD,
        placement: DashboardPlacement::Viewport(ComponentViewportRegion::new(
            ComponentViewportAxisPlacement::fixed_from_start(x, width).expect("the card has width"),
            ComponentViewportAxisPlacement::bounded_stretch(y, y, height)
                .expect("the card has height"),
        )),
        tracks: MosaicLayoutContract::rows([
            track(MosaicTrack::fixed(HEADING_ROW[3])),
            track(MosaicTrack::flex(1, CHANGES_MINIMUM)),
            track(MosaicTrack::fixed(ACTIONS_ROW[3])),
        ])
        .expect("the card declares its rows"),
        stacked: None,
        scroll_owner: None,
        portal_owner: Some(OWNER),
    };
    let list = ContainerTracks {
        id: LIST,
        placement: changes_row().placement(LIST_RECT, Edge::Start, Edge::Start),
        tracks: MosaicLayoutContract::frame().expect("scrolled content frames its members"),
        stacked: None,
        scroll_owner: Some(DashboardScrollOwner::Review),
        portal_owner: Some(OWNER),
    };
    vec![card, list]
}

/// The changes list's region: its concept rectangle within the changes row,
/// as tall as the row lets it be.
pub(super) fn list_region() -> ComponentViewportRegion {
    changes_row().region(LIST_RECT, Edge::Start, Edge::Both)
}

fn changes_row() -> Frame {
    Frame::cell(CARD, MosaicLayoutCell::at(0, 1), CHANGES_ROW)
}

/// The shadow: the card and its margin on every side, as tall as the card
/// lets it be.
fn shadow_region() -> ComponentViewportRegion {
    let [x, y, width, height] = CARD_RECT;
    let margin = 2 * SHADOW_MARGIN;
    ComponentViewportRegion::new(
        ComponentViewportAxisPlacement::fixed_from_start(x - SHADOW_MARGIN, width + margin)
            .expect("the shadow has width"),
        ComponentViewportAxisPlacement::bounded_stretch(
            y - SHADOW_MARGIN,
            y - SHADOW_MARGIN,
            height + margin,
        )
        .expect("the shadow has height"),
    )
}
