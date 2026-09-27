//! An independent geometric oracle for the deployment review modal.
//!
//! The spec fixes the modal in closed form from the Portal owner's origin:
//! the card stands 36 points in, 521 points wide, and as tall as the concept's
//! 444 points where the viewport leaves a 36-point inset below it, and no
//! taller than that inset allows where it does not. The shadow keeps a
//! 36-point margin around the card. Inside the card, the heading keeps 141
//! points and the actions 97, and the changes row takes what is left; the
//! change list scrolls its 206-point content through that row.
use super::super::declared_layout::{assert_near, Declared, Rect, EXTENTS};
use super::super::DashboardScrollOwner;

/// The concept extents, and a viewport too short for the concept card.
fn extents() -> impl Iterator<Item = (f32, f32)> {
    EXTENTS.into_iter().chain([(800.0, 400.0), (1536.0, 500.0)])
}

/// The card's height at a viewport height: the concept's, down to what the
/// insets above and below leave.
fn card_height(height: f32) -> f32 {
    (height - 72.0).min(444.0)
}

#[test]
fn the_card_and_its_shadow_hold_the_concept_where_the_viewport_has_room() {
    for (width, height) in extents() {
        let declared = Declared::at(width, height);
        let at = format!("at {width}x{height}");
        let [x, y, _, _] = declared.element("review_target");
        let card: Rect = [x + 36.0, y + 36.0, 521.0, card_height(height)];
        assert_near(
            declared.container("review_card"),
            card,
            &format!("card {at}"),
        );
        assert_near(
            declared.element("review_surface"),
            card,
            &format!("surface {at}"),
        );
        assert_near(
            declared.element("review_shadow"),
            [x, y, 593.0, card[3] + 72.0],
            &format!("shadow {at}"),
        );
    }
    let concept = Declared::at(1536.0, 1024.0);
    let [x, y, _, _] = concept.element("review_target");
    assert_near(
        concept.container("review_card"),
        [x + 36.0, y + 36.0, 521.0, 444.0],
        "concept card",
    );
}

#[test]
fn the_changes_take_what_the_heading_and_actions_leave_and_scroll_their_list() {
    for (width, height) in extents() {
        let declared = Declared::at(width, height);
        let at = format!("at {width}x{height}");
        let [x, y, _, h] = declared.container("review_card");
        let region = declared.region(DashboardScrollOwner::Review);
        assert_near(
            region,
            [x + 31.0, y + 141.0, 460.0, h - 238.0],
            &format!("list region {at}"),
        );
        assert_near(
            declared.container("review_list"),
            [region[0], region[1], 460.0, 206.0],
            &format!("list content {at}"),
        );
        assert_near(
            declared.element("review_changes"),
            region,
            &format!("changes fill {at}"),
        );
        assert_near(
            declared.element("change_title_0"),
            [region[0] + 66.0, region[1] + 18.0, 318.0, 25.0],
            &format!("first change {at}"),
        );
        assert_near(
            declared.element("review_title"),
            [x + 31.0, y + 30.0, 431.0, 43.0],
            &format!("title {at}"),
        );
        assert_near(
            declared.element("review_primary_target"),
            [x + 280.0, y + h - 77.0, 211.0, 48.0],
            &format!("approve {at}"),
        );
    }
    // A viewport too short for the concept card shortens the changes row,
    // not the heading or the actions; the list keeps its content and scrolls.
    let short = Declared::at(800.0, 400.0);
    assert_near(
        short.region(DashboardScrollOwner::Review),
        {
            let [x, y, _, _] = short.container("review_card");
            [x + 31.0, y + 141.0, 460.0, 90.0]
        },
        "short list region",
    );
}
