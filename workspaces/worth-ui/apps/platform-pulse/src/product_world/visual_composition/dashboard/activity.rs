use super::{surface, text, DashboardElement};

pub(super) fn elements() -> Vec<DashboardElement> {
    let mut rows = vec![
        surface(
            "activity_card",
            [266, 636, 812, 346],
            "raised_surface",
            12,
            true,
            2,
        ),
        text(
            "activity_heading",
            "Recent activity",
            [291, 653, 360, 35],
            18,
            true,
            "primary_text",
        ),
        surface(
            "activity_rail",
            [299, 711, 1, 218],
            "structural_rule",
            0,
            false,
            3,
        ),
        surface(
            "activity_dot_0",
            [293, 704, 15, 15],
            "positive",
            32,
            false,
            4,
        ),
        text(
            "activity_title_0",
            "Deployment succeeded",
            [330, 695, 500, 24],
            14,
            false,
            "primary_text",
        ),
        text(
            "activity_body_0",
            "v2.4.1 deployed to production",
            [330, 716, 540, 23],
            13,
            false,
            "secondary_text",
        ),
        text(
            "activity_time_0",
            "Just now",
            [991, 696, 66, 24],
            12,
            false,
            "secondary_text",
        ),
        surface("activity_rule_0", [330, 739, 717, 1], "grid", 0, false, 3),
        surface(
            "activity_dot_1",
            [293, 759, 15, 15],
            "principal_accent",
            32,
            false,
            4,
        ),
        text(
            "activity_title_1",
            "Configuration updated",
            [330, 750, 500, 24],
            14,
            false,
            "primary_text",
        ),
        text(
            "activity_body_1",
            "Search relevance tuning",
            [330, 771, 540, 23],
            13,
            false,
            "secondary_text",
        ),
        text(
            "activity_time_1",
            "2m ago",
            [991, 751, 66, 24],
            12,
            false,
            "secondary_text",
        ),
        surface("activity_rule_1", [330, 794, 717, 1], "grid", 0, false, 3),
        surface(
            "activity_dot_2",
            [293, 814, 15, 15],
            "caution",
            32,
            false,
            4,
        ),
        text(
            "activity_title_2",
            "Increased traffic detected",
            [330, 805, 500, 24],
            14,
            false,
            "primary_text",
        ),
        text(
            "activity_body_2",
            "+18% compared to previous hour",
            [330, 826, 540, 23],
            13,
            false,
            "secondary_text",
        ),
        text(
            "activity_time_2",
            "28m ago",
            [991, 806, 66, 24],
            12,
            false,
            "secondary_text",
        ),
        surface("activity_rule_2", [330, 849, 717, 1], "grid", 0, false, 3),
        surface(
            "activity_dot_3",
            [293, 869, 15, 15],
            "positive",
            32,
            false,
            4,
        ),
        text(
            "activity_title_3",
            "New signal resolved",
            [330, 860, 500, 24],
            14,
            false,
            "primary_text",
        ),
        text(
            "activity_body_3",
            "Latency back to normal",
            [330, 881, 540, 23],
            13,
            false,
            "secondary_text",
        ),
        text(
            "activity_time_3",
            "1h ago",
            [991, 861, 66, 24],
            12,
            false,
            "secondary_text",
        ),
        surface("activity_rule_3", [330, 904, 717, 1], "grid", 0, false, 3),
        surface(
            "activity_dot_4",
            [293, 924, 15, 15],
            "principal_accent",
            32,
            false,
            4,
        ),
        text(
            "activity_title_4",
            "Scheduled deployment",
            [330, 915, 500, 24],
            14,
            false,
            "primary_text",
        ),
        text(
            "activity_body_4",
            "v2.5.0 in 3 hours",
            [330, 936, 540, 23],
            13,
            false,
            "secondary_text",
        ),
        text(
            "activity_time_4",
            "3h ago",
            [991, 916, 66, 24],
            12,
            false,
            "secondary_text",
        ),
    ];
    for row in &mut rows {
        if row.id.starts_with("activity_")
            && !matches!(row.id, "activity_card" | "activity_heading")
        {
            row.scroll_panel = Some(super::DashboardScrollPanel::RecentActivity);
            if let Some(index) = row
                .id
                .rsplit('_')
                .next()
                .and_then(|value| value.parse::<u16>().ok())
            {
                row.rect[1] += index * 17;
            }
        }
    }
    rows.iter_mut()
        .find(|row| row.id == "activity_rail")
        .unwrap()
        .rect[3] = 286;
    rows
}
