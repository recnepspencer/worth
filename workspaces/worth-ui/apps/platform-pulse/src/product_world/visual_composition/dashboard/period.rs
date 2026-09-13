use super::{surface, text, DashboardElement, DashboardGraphic};

pub(super) fn elements() -> Vec<DashboardElement> {
    vec![
        surface("period_shadow", [0, 0, 272, 236], "shadow", 12, false, 1)
            .graphic(DashboardGraphic::Shadow)
            .portal("confirmation_target"),
        surface(
            "period_surface",
            [36, 36, 200, 164],
            "raised_surface",
            12,
            true,
            2,
        )
        .portal("confirmation_target"),
        surface(
            "period_day",
            [44, 44, 184, 48],
            "raised_surface",
            8,
            false,
            3,
        )
        .portal("confirmation_target")
        .action("close"),
        text(
            "period_day_label",
            "Last 24 hours",
            [58, 44, 150, 48],
            14,
            false,
            "primary_text",
        )
        .portal("confirmation_target"),
        surface(
            "period_week",
            [44, 94, 184, 48],
            "raised_surface",
            8,
            false,
            3,
        )
        .portal("confirmation_target")
        .action("close"),
        text(
            "period_week_label",
            "Last 7 days",
            [58, 94, 150, 48],
            14,
            false,
            "primary_text",
        )
        .portal("confirmation_target"),
        surface(
            "period_month",
            [44, 144, 184, 48],
            "raised_surface",
            8,
            false,
            3,
        )
        .portal("confirmation_target")
        .action("close"),
        text(
            "period_month_label",
            "Last 30 days",
            [58, 144, 150, 48],
            14,
            false,
            "primary_text",
        )
        .portal("confirmation_target"),
    ]
}
