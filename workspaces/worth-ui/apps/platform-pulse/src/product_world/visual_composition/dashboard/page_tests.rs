//! An independent geometric oracle for the dashboard page.
//!
//! The spec fixes the page in closed form: a 236-point sidebar, a 24-point
//! gutter, 20-point gaps, panel columns sharing the width 2:1 and never
//! narrower than 480 and 320 points, and panel rows sharing the height left
//! below the greeting and the summary cards. Each test resolves what the
//! dashboard declares and checks it against that closed form, at the concept
//! extent and away from it.
use super::declared_layout::{assert_near, Declared, Rect, EXTENTS, PANELS};
use super::{dashboard_elements, DashboardScrollPanel};

/// Splits `available` between two tracks by weight, holding a share below
/// its minimum at the minimum and giving the other what remains.
fn split(available: f32, weights: [f32; 2], minimums: [f32; 2]) -> [f32; 2] {
    let shares = weights.map(|weight| available * weight / (weights[0] + weights[1]));
    if shares[0] < minimums[0] {
        [minimums[0], (available - minimums[0]).max(minimums[1])]
    } else if shares[1] < minimums[1] {
        [(available - minimums[1]).max(minimums[0]), minimums[1]]
    } else {
        shares
    }
}

/// The spec's page, card row, and four panels at one extent.
fn spec(width: f32, height: f32) -> (Rect, Rect, [Rect; 4]) {
    let page = [260.0, 81.0, width - 284.0, height - 105.0];
    let [left, right] = split(page[2] - 20.0, [2.0, 1.0], [480.0, 320.0]);
    let [upper, lower] = split(page[3] - 66.0 - 98.0 - 60.0, [1.0, 1.0], [348.0, 346.0]);
    let (second, upper_y) = (page[0] + left + 20.0, 285.0);
    let lower_y = upper_y + upper + 20.0;
    let cards = [page[0], 167.0, page[2], 98.0];
    (
        page,
        cards,
        [
            [page[0], upper_y, left, upper],
            [second, upper_y, right, upper],
            [page[0], lower_y, left, lower],
            [second, lower_y, right, lower],
        ],
    )
}

fn inside(inner: Rect, outer: Rect) -> bool {
    inner[0] >= outer[0] - 1e-3
        && inner[1] >= outer[1] - 1e-3
        && inner[0] + inner[2] <= outer[0] + outer[2] + 1e-3
        && inner[1] + inner[3] <= outer[1] + outer[3] + 1e-3
}

#[test]
fn the_page_and_its_panels_resolve_to_the_spec() {
    for (width, height) in EXTENTS {
        let declared = Declared::at(width, height);
        let (page, cards, panels) = spec(width, height);
        let at = format!("at {width}x{height}");
        assert_near(declared.container("page"), page, &format!("page {at}"));
        assert_near(
            declared.container("metric_row"),
            cards,
            &format!("cards {at}"),
        );
        for (id, expected) in PANELS.into_iter().zip(panels) {
            assert_near(declared.container(id), expected, &format!("{id} {at}"));
        }
        let greeting = declared.element("service_title");
        assert_near(
            greeting,
            [page[0], 104.0, page[2], 43.0],
            &format!("greeting {at}"),
        );
    }
}

#[test]
fn the_sidebar_and_masthead_keep_their_edges() {
    for (width, height) in EXTENTS {
        let declared = Declared::at(width, height);
        let at = format!("at {width}x{height}");
        let expect =
            |id: &str, rect: Rect| assert_near(declared.element(id), rect, &format!("{id} {at}"));
        expect("evidence_rail", [0.0, 0.0, 236.0, height]);
        expect("brand", [26.0, 26.0, 204.0, 40.0]);
        expect("source_signal_active", [26.0, height - 123.0, 9.0, 9.0]);
        expect("status_text", [26.0, height - 68.0, 186.0, 44.0]);
        expect("masthead_border", [236.0, 56.0, width - 236.0, 1.0]);
        expect("search_field", [width - 482.0, 12.0, 322.0, 40.0]);
        expect("portal_target", [width - 136.0, 12.0, 40.0, 40.0]);
        expect("avatar", [width - 70.0, 12.0, 40.0, 40.0]);
    }
}

/// Each time label stays centered under its vertical gridline, and each
/// volume label stays just above its horizontal one.
#[test]
fn axis_labels_stay_on_their_gridlines() {
    for (width, height) in EXTENTS {
        let declared = Declared::at(width, height);
        let panel = declared.container("chart_panel");
        let grid = declared.element("chart_grid");
        let at = format!("at {width}x{height}");
        let expected = [
            panel[0] + 65.0,
            panel[1] + 78.0,
            panel[2] - 91.0,
            panel[3] - 132.0,
        ];
        assert_near(grid, expected, &format!("chart grid {at}"));
        for k in 0..7 {
            let label = declared.element(&format!("chart_x_{k}"));
            let center = grid[0] + k as f32 * grid[2] / 6.0;
            let expected = [center - 22.0, panel[1] + panel[3] - 45.0, 44.0, 23.0];
            assert_near(label, expected, &format!("time label {k} {at}"));
        }
        for k in 0..4 {
            let label = declared.element(&format!("chart_y_{k}"));
            let top = grid[1] - 9.0 + k as f32 * grid[3] / 3.0;
            assert_near(
                label,
                [panel[0] + 24.0, top, 37.0, 21.0],
                &format!("volume label {k} {at}"),
            );
        }
    }
}

#[test]
fn each_list_region_fills_its_panel_below_the_heading() {
    for (width, height) in EXTENTS {
        let declared = Declared::at(width, height);
        let at = format!("at {width}x{height}");
        let health = declared.container("health_panel");
        let region = declared.region(DashboardScrollPanel::ServiceHealth);
        let expected = [
            health[0] + 25.0,
            health[1] + 55.0,
            health[2] - 51.0,
            health[3] - 79.0,
        ];
        assert_near(region, expected, &format!("service health region {at}"));
        let content = declared.container("health_content");
        assert_near(
            content,
            [region[0], region[1], region[2], 344.0],
            &format!("service health content {at}"),
        );
        let activity = declared.container("activity_panel");
        let region = declared.region(DashboardScrollPanel::RecentActivity);
        let expected = [
            activity[0] + 24.0,
            activity[1] + 57.0,
            activity[2] - 44.0,
            activity[3] - 77.0,
        ];
        assert_near(region, expected, &format!("recent activity region {at}"));
        let content = declared.container("activity_content");
        assert_near(
            content,
            [region[0], region[1], 1560.0, 672.0],
            &format!("recent activity content {at}"),
        );
        let dot = declared.element("health_dot_0");
        let content = declared.container("health_content");
        assert_near(
            dot,
            [content[0] + 9.0, content[1] + 11.0, 11.0, 11.0],
            &format!("health dot {at}"),
        );
    }
    let concept = Declared::at(1536.0, 1024.0);
    let region = concept.region(DashboardScrollPanel::RecentActivity);
    assert_near(
        region,
        [284.0, 710.0, 777.0 + 1.0 / 3.0, 270.0],
        "recent activity region",
    );
}

/// Everything a panel frames stays inside it; only the lists' content, which
/// their regions clip, may extend past.
#[test]
fn panel_content_stays_inside_its_panel() {
    for (width, height) in EXTENTS {
        let declared = Declared::at(width, height);
        for element in dashboard_elements() {
            let Some(panel) = declared.panel_of(element.placement) else {
                continue;
            };
            if element.scroll_panel.is_some() {
                continue;
            }
            assert!(
                inside(declared.element(element.id), declared.container(panel)),
                "{} leaves {panel} at {width}x{height}",
                element.id,
            );
        }
    }
}

#[test]
fn panel_headings_keep_clear_of_their_controls() {
    for (width, height) in EXTENTS {
        let declared = Declared::at(width, height);
        for (heading, control) in [
            ("service_body", "confirmation_target"),
            ("deployments_heading", "deployments_all"),
        ] {
            let heading_box = declared.element(heading);
            let control_box = declared.element(control);
            assert!(
                heading_box[0] + heading_box[2] <= control_box[0],
                "{control} crowds {heading} at {width}x{height}",
            );
        }
    }
}
