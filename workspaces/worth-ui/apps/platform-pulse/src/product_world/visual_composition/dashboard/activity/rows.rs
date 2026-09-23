//! The declared Recent activity row table: twelve rows on one declared pitch.
//!
//! Every row's block position comes from its band index times the declared
//! pitch, in coordinates local to the scrolled content. Nothing is patched
//! after the table is built, and nothing here restates where the panel lands
//! on the host surface.
use super::super::{surface, text, DashboardElement};
use super::scrolled;

/// The block extent one activity row occupies, band top to band top.
pub const PLATFORM_PULSE_ACTIVITY_ROW_PITCH_LOGICAL_POINTS: u16 = 56;
/// The number of rows the Recent activity list authors.
pub const PLATFORM_PULSE_ACTIVITY_ROW_COUNT: usize = 12;

/// Content-local block origin of a row band.
pub(super) const fn row_band_top(band: u16) -> u16 {
    band * PLATFORM_PULSE_ACTIVITY_ROW_PITCH_LOGICAL_POINTS
}

/// Inline offset and extent of the leading cells, from the content origin.
const DOT_LOCAL_X: u16 = 3;
const LABEL_LOCAL_X: u16 = 40;
const LABEL_WIDTH: u16 = 420;
const AGE_LOCAL_X: u16 = 472;
const AGE_WIDTH: u16 = 96;
/// The separator spans every column, so it ends where the last column ends.
const SEPARATOR_WIDTH: u16 = 1_496;

/// One authored activity row. Its element identities are bound to its band
/// index, so a row cannot be described by one index and painted at another.
struct ActivityRow {
    band: u16,
    dot_id: &'static str,
    title_id: &'static str,
    body_id: &'static str,
    time_id: &'static str,
    rule_id: &'static str,
    tone: &'static str,
    title: &'static str,
    detail: &'static str,
    age: &'static str,
}

/// Binds one row's authored text to the element identities its band owns.
macro_rules! activity_row {
    ($band:literal, $tone:literal, $title:literal, $detail:literal, $age:literal) => {
        ActivityRow {
            band: $band,
            dot_id: concat!("activity_dot_", $band),
            title_id: concat!("activity_title_", $band),
            body_id: concat!("activity_body_", $band),
            time_id: concat!("activity_time_", $band),
            rule_id: concat!("activity_rule_", $band),
            tone: $tone,
            title: $title,
            detail: $detail,
            age: $age,
        }
    };
}

const ROWS: [ActivityRow; PLATFORM_PULSE_ACTIVITY_ROW_COUNT] = [
    activity_row!(
        0,
        "positive",
        "Deployment succeeded",
        "v2.4.1 deployed to production",
        "Just now"
    ),
    activity_row!(
        1,
        "principal_accent",
        "Configuration updated",
        "Search relevance tuning",
        "2m ago"
    ),
    activity_row!(
        2,
        "caution",
        "Increased traffic detected",
        "+18% compared to previous hour",
        "28m ago"
    ),
    activity_row!(
        3,
        "positive",
        "New signal resolved",
        "Latency back to normal",
        "1h ago"
    ),
    activity_row!(
        4,
        "principal_accent",
        "Scheduled deployment",
        "v2.5.0 in 3 hours",
        "3h ago"
    ),
    activity_row!(
        5,
        "positive",
        "Cache warm-up completed",
        "Edge nodes repopulated",
        "4h ago"
    ),
    activity_row!(
        6,
        "caution",
        "Certificate renewal pending",
        "Expires in nine days",
        "5h ago"
    ),
    activity_row!(
        7,
        "negative",
        "Checkout latency spiked",
        "p99 above 900 ms for four minutes",
        "7h ago"
    ),
    activity_row!(
        8,
        "positive",
        "Rollback verified",
        "v2.3.9 restored on two regions",
        "9h ago"
    ),
    activity_row!(
        9,
        "principal_accent",
        "Feature flag enabled",
        "Adaptive sampling at 25 percent",
        "11h ago"
    ),
    activity_row!(
        10,
        "caution",
        "Disk pressure warning",
        "Analytics volume at 82 percent",
        "14h ago"
    ),
    activity_row!(
        11,
        "positive",
        "Nightly backup completed",
        "Snapshot stored in three regions",
        "18h ago"
    ),
];

pub(super) fn activity_rows() -> Vec<DashboardElement> {
    ROWS.iter().flat_map(row_elements).collect()
}

fn row_elements(row: &ActivityRow) -> [DashboardElement; 5] {
    let top = row_band_top(row.band);
    [
        scrolled(surface(
            row.dot_id,
            [DOT_LOCAL_X, top + 10, 15, 15],
            row.tone,
            32,
            false,
            4,
        )),
        scrolled(text(
            row.title_id,
            row.title,
            [LABEL_LOCAL_X, top + 2, LABEL_WIDTH, 24],
            14,
            false,
            "primary_text",
        )),
        scrolled(text(
            row.body_id,
            row.detail,
            [LABEL_LOCAL_X, top + 25, LABEL_WIDTH, 23],
            13,
            false,
            "secondary_text",
        )),
        scrolled(text(
            row.time_id,
            row.age,
            [AGE_LOCAL_X, top + 3, AGE_WIDTH, 24],
            12,
            false,
            "secondary_text",
        )),
        scrolled(surface(
            row.rule_id,
            [LABEL_LOCAL_X, top + 55, SEPARATOR_WIDTH, 1],
            "grid",
            0,
            false,
            3,
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::super::super::DashboardScrollPanel;
    use super::*;

    /// The scrolled content rectangle the row table is authored inside.
    const CONTENT_RECT: [u16; 4] = DashboardScrollPanel::RecentActivity.content_rect();

    #[test]
    fn every_row_band_sits_on_the_declared_pitch() {
        for (ordinal, row) in ROWS.iter().enumerate() {
            assert_eq!(usize::from(row.band), ordinal, "row bands are dense");
            assert_eq!(
                row_band_top(row.band),
                row.band * PLATFORM_PULSE_ACTIVITY_ROW_PITCH_LOGICAL_POINTS,
            );
        }
    }

    /// The table is authored from the content origin, so the first band starts
    /// at zero and the last band ends exactly at the authored content height.
    #[test]
    fn the_row_table_fills_the_authored_content_block_extent() {
        let bands = PLATFORM_PULSE_ACTIVITY_ROW_COUNT as u16;
        assert_eq!(row_band_top(0), 0, "the first band is the content origin");
        assert_eq!(
            bands * PLATFORM_PULSE_ACTIVITY_ROW_PITCH_LOGICAL_POINTS,
            CONTENT_RECT[3],
            "twelve bands at the declared pitch are the authored content height",
        );
        assert_eq!(row_band_top(bands), CONTENT_RECT[3]);
    }
}
