//! The Service health card: its fixed chrome and the vertical list that travels
//! with the Service health scroll owner.
//!
//! Every row sits at its band index times the declared pitch, in coordinates
//! local to the scrolled content. Nothing here restates where the panel lands
//! on the host surface, and nothing is patched after the table is built.
use super::{surface, text, DashboardElement, DashboardScrollPanel};

/// The block extent one service health row occupies, band top to band top.
const ROW_PITCH_LOGICAL_POINTS: u16 = 52;
/// The number of rows the Service health list authors.
const ROW_COUNT: usize = 7;

/// Content-local block origin of a row band.
const fn row_band_top(band: u16) -> u16 {
    band * ROW_PITCH_LOGICAL_POINTS
}

/// Content-local inline offsets and extents of the cells in a row band.
const DOT_LOCAL_X: u16 = 9;
const DOT_EXTENT: u16 = 11;
const DOT_BLOCK_INSET: u16 = 11;
const NAME_LOCAL_X: u16 = 40;
const NAME_WIDTH: u16 = 180;
const NAME_BLOCK_INSET: u16 = 5;
const BADGE_LOCAL_X: u16 = 272;
const BADGE_WIDTH: u16 = 85;
const BADGE_HEIGHT: u16 = 25;
const BADGE_BLOCK_INSET: u16 = 4;
const RULE_LOCAL_X: u16 = 1;
const RULE_WIDTH: u16 = 357;
const RULE_BLOCK_INSET: u16 = 35;

/// Marks one element as travelling with the Service health content. The card
/// and its heading never carry this: they stay put while the list moves.
const fn scrolled(mut element: DashboardElement) -> DashboardElement {
    element.scroll_panel = Some(DashboardScrollPanel::ServiceHealth);
    element
}

/// One authored service row. Its element identities are bound to its band
/// index, so a row cannot be described by one index and painted at another.
struct HealthRow {
    band: u16,
    dot_id: &'static str,
    name_id: &'static str,
    badge_id: &'static str,
    status_id: &'static str,
    rule_id: &'static str,
    tone: &'static str,
    badge_tone: &'static str,
    name: &'static str,
    status: &'static str,
}

/// Binds one row's authored text to the element identities its band owns.
macro_rules! health_row {
    ($band:literal, $tone:literal, $badge_tone:literal, $name:literal, $status:literal) => {
        HealthRow {
            band: $band,
            dot_id: concat!("health_dot_", $band),
            name_id: concat!("health_name_", $band),
            badge_id: concat!("health_badge_", $band),
            status_id: concat!("health_status_", $band),
            rule_id: concat!("health_rule_", $band),
            tone: $tone,
            badge_tone: $badge_tone,
            name: $name,
            status: $status,
        }
    };
}

const ROWS: [HealthRow; ROW_COUNT] = [
    health_row!(0, "positive", "mint_pale", "API", "Operational"),
    health_row!(1, "positive", "mint_pale", "Web app", "Operational"),
    health_row!(2, "positive", "mint_pale", "Worker jobs", "Operational"),
    health_row!(3, "positive", "mint_pale", "Database", "Operational"),
    health_row!(4, "positive", "mint_pale", "CDN", "Operational"),
    health_row!(5, "caution", "amber_pale", "Payments", "Degraded"),
    health_row!(6, "positive", "mint_pale", "Search", "Operational"),
];

pub(super) fn elements() -> Vec<DashboardElement> {
    let mut elements = vec![card(), heading()];
    elements.extend(ROWS.iter().flat_map(row_elements));
    elements
}

/// The card behind the list. Fixed: the region clips the content against it.
fn card() -> DashboardElement {
    surface(
        "native_card",
        [1095, 269, 411, 348],
        "raised_surface",
        12,
        true,
        2,
    )
}

/// The panel header. Fixed, as neighbouring panel headers are.
fn heading() -> DashboardElement {
    text(
        "native_label",
        "Service health",
        [1120, 288, 290, 35],
        18,
        true,
        "primary_text",
    )
}

fn row_elements(row: &HealthRow) -> Vec<DashboardElement> {
    let top = row_band_top(row.band);
    let mut cells = vec![
        scrolled(surface(
            row.dot_id,
            [DOT_LOCAL_X, top + DOT_BLOCK_INSET, DOT_EXTENT, DOT_EXTENT],
            row.tone,
            32,
            false,
            3,
        )),
        scrolled(text(
            row.name_id,
            row.name,
            [NAME_LOCAL_X, top + NAME_BLOCK_INSET, NAME_WIDTH, 25],
            14,
            false,
            "primary_text",
        )),
        scrolled(surface(
            row.badge_id,
            [
                BADGE_LOCAL_X,
                top + BADGE_BLOCK_INSET,
                BADGE_WIDTH,
                BADGE_HEIGHT,
            ],
            row.badge_tone,
            8,
            false,
            3,
        )),
        scrolled(
            text(
                row.status_id,
                row.status,
                [
                    BADGE_LOCAL_X,
                    top + BADGE_BLOCK_INSET,
                    BADGE_WIDTH,
                    BADGE_HEIGHT,
                ],
                11,
                false,
                row.tone,
            )
            .centered(),
        ),
    ];
    // The last band has nothing after it to be separated from.
    if usize::from(row.band) + 1 < ROW_COUNT {
        cells.push(scrolled(surface(
            row.rule_id,
            [RULE_LOCAL_X, top + RULE_BLOCK_INSET, RULE_WIDTH, 1],
            "grid",
            0,
            false,
            3,
        )));
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scrolled content rectangle the rows are authored inside.
    const CONTENT_RECT: [u16; 4] = DashboardScrollPanel::ServiceHealth.content_rect();

    #[test]
    fn every_row_band_sits_on_the_declared_pitch() {
        for (ordinal, row) in ROWS.iter().enumerate() {
            assert_eq!(usize::from(row.band), ordinal, "row bands are dense");
            assert_eq!(
                row_band_top(row.band),
                row.band * ROW_PITCH_LOGICAL_POINTS,
                "a row band is its index times the declared pitch",
            );
        }
    }

    /// The rows are authored in content-local coordinates, so every cell must
    /// fall inside the authored content rectangle measured from its own origin.
    #[test]
    fn every_authored_cell_stays_inside_the_content_extent() {
        let [_, _, content_width, content_height] = CONTENT_RECT;
        for element in elements()
            .into_iter()
            .filter(|element| element.scroll_panel == Some(DashboardScrollPanel::ServiceHealth))
        {
            let [x, y, width, height] = element.rect;
            assert!(
                x + width <= content_width,
                "{} escapes the content inline extent",
                element.id,
            );
            assert!(
                y + height <= content_height,
                "{} escapes the content block extent",
                element.id,
            );
        }
    }

    /// Service health stays a vertical list. If the card and heading travelled
    /// with the rows the panel would scroll its own chrome away.
    #[test]
    fn the_card_and_heading_do_not_travel_with_the_list() {
        for element in elements() {
            let scrolls = element.scroll_panel.is_some();
            assert_eq!(
                scrolls,
                element.id.starts_with("health_"),
                "{} travels with the wrong owner",
                element.id,
            );
        }
    }
}
