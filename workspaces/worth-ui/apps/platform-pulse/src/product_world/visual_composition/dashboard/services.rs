use super::{surface, text, DashboardElement};

pub(super) fn elements() -> Vec<DashboardElement> {
    let mut rows = vec![
        surface(
            "native_card",
            [1095, 269, 411, 348],
            "raised_surface",
            12,
            true,
            2,
        ),
        text(
            "native_label",
            "Service health",
            [1120, 288, 290, 35],
            18,
            true,
            "primary_text",
        ),
        surface(
            "health_dot_0",
            [1129, 335, 11, 11],
            "positive",
            32,
            false,
            3,
        ),
        text(
            "health_name_0",
            "API",
            [1160, 329, 180, 25],
            14,
            false,
            "primary_text",
        ),
        surface(
            "health_badge_0",
            [1392, 328, 85, 25],
            "mint_pale",
            8,
            false,
            3,
        ),
        text(
            "health_status_0",
            "Operational",
            [1392, 328, 85, 25],
            11,
            false,
            "positive",
        )
        .centered(),
        surface("health_rule_0", [1121, 359, 357, 1], "grid", 0, false, 3),
        surface(
            "health_dot_1",
            [1129, 374, 11, 11],
            "positive",
            32,
            false,
            3,
        ),
        text(
            "health_name_1",
            "Web app",
            [1160, 368, 180, 25],
            14,
            false,
            "primary_text",
        ),
        surface(
            "health_badge_1",
            [1392, 367, 85, 25],
            "mint_pale",
            8,
            false,
            3,
        ),
        text(
            "health_status_1",
            "Operational",
            [1392, 367, 85, 25],
            11,
            false,
            "positive",
        )
        .centered(),
        surface("health_rule_1", [1121, 398, 357, 1], "grid", 0, false, 3),
        surface(
            "health_dot_2",
            [1129, 413, 11, 11],
            "positive",
            32,
            false,
            3,
        ),
        text(
            "health_name_2",
            "Worker jobs",
            [1160, 407, 180, 25],
            14,
            false,
            "primary_text",
        ),
        surface(
            "health_badge_2",
            [1392, 406, 85, 25],
            "mint_pale",
            8,
            false,
            3,
        ),
        text(
            "health_status_2",
            "Operational",
            [1392, 406, 85, 25],
            11,
            false,
            "positive",
        )
        .centered(),
        surface("health_rule_2", [1121, 437, 357, 1], "grid", 0, false, 3),
        surface(
            "health_dot_3",
            [1129, 452, 11, 11],
            "positive",
            32,
            false,
            3,
        ),
        text(
            "health_name_3",
            "Database",
            [1160, 446, 180, 25],
            14,
            false,
            "primary_text",
        ),
        surface(
            "health_badge_3",
            [1392, 445, 85, 25],
            "mint_pale",
            8,
            false,
            3,
        ),
        text(
            "health_status_3",
            "Operational",
            [1392, 445, 85, 25],
            11,
            false,
            "positive",
        )
        .centered(),
        surface("health_rule_3", [1121, 476, 357, 1], "grid", 0, false, 3),
        surface(
            "health_dot_4",
            [1129, 491, 11, 11],
            "positive",
            32,
            false,
            3,
        ),
        text(
            "health_name_4",
            "CDN",
            [1160, 485, 180, 25],
            14,
            false,
            "primary_text",
        ),
        surface(
            "health_badge_4",
            [1392, 484, 85, 25],
            "mint_pale",
            8,
            false,
            3,
        ),
        text(
            "health_status_4",
            "Operational",
            [1392, 484, 85, 25],
            11,
            false,
            "positive",
        )
        .centered(),
        surface("health_rule_4", [1121, 515, 357, 1], "grid", 0, false, 3),
        surface("health_dot_5", [1129, 530, 11, 11], "caution", 32, false, 3),
        text(
            "health_name_5",
            "Payments",
            [1160, 524, 180, 25],
            14,
            false,
            "primary_text",
        ),
        surface(
            "health_badge_5",
            [1392, 523, 85, 25],
            "amber_pale",
            8,
            false,
            3,
        ),
        text(
            "health_status_5",
            "Degraded",
            [1392, 523, 85, 25],
            11,
            false,
            "caution",
        )
        .centered(),
        surface("health_rule_5", [1121, 554, 357, 1], "grid", 0, false, 3),
        surface(
            "health_dot_6",
            [1129, 569, 11, 11],
            "positive",
            32,
            false,
            3,
        ),
        text(
            "health_name_6",
            "Search",
            [1160, 563, 180, 25],
            14,
            false,
            "primary_text",
        ),
        surface(
            "health_badge_6",
            [1392, 562, 85, 25],
            "mint_pale",
            8,
            false,
            3,
        ),
        text(
            "health_status_6",
            "Operational",
            [1392, 562, 85, 25],
            11,
            false,
            "positive",
        )
        .centered(),
    ];
    for row in &mut rows {
        if row.id.starts_with("health_") {
            row.scroll_panel = Some(super::DashboardScrollPanel::ServiceHealth);
            if let Some(index) = row
                .id
                .rsplit('_')
                .next()
                .and_then(|value| value.parse::<u16>().ok())
            {
                row.rect[1] += index * 13;
            }
        }
    }
    rows
}
