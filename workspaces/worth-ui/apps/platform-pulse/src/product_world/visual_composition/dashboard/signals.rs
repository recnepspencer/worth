use super::{surface, text, DashboardElement, DashboardGraphic};

pub(super) fn elements() -> Vec<DashboardElement> {
    vec![
        surface("signals_shadow", [0, 0, 379, 325], "shadow", 12, false, 1)
            .graphic(DashboardGraphic::Shadow)
            .portal("portal_target"),
        surface(
            "portal_surface",
            [36, 36, 307, 253],
            "raised_surface",
            12,
            true,
            2,
        )
        .portal("portal_target"),
        // The pointer fill joins over the panel's top border; its two sloping
        // edges paint above that fill. There is intentionally no base stroke.
        surface(
            "signals_pointer",
            [242, 28, 18, 10],
            "raised_surface",
            0,
            false,
            3,
        )
        .graphic(DashboardGraphic::PopoverPointer)
        .portal("portal_target"),
        surface(
            "signals_pointer_edge",
            [242, 28, 18, 9],
            "grid",
            0,
            false,
            4,
        )
        .graphic(DashboardGraphic::PopoverPointerEdge)
        .portal("portal_target"),
        text(
            "portal_title",
            "Recent signals",
            [55, 49, 206, 32],
            18,
            true,
            "primary_text",
        )
        .portal("portal_target"),
        text(
            "portal_primary_label",
            "View all",
            [269, 45, 70, 32],
            13,
            false,
            "principal_accent",
        )
        .centered()
        .portal("portal_target"),
        surface(
            "portal_primary_target",
            [269, 45, 70, 32],
            "raised_surface",
            8,
            false,
            3,
        )
        .portal("portal_target")
        .action("all_signals"),
        surface("portal_accent", [36, 84, 307, 1], "grid", 0, false, 3).portal("portal_target"),
        surface(
            "signal_icon_bg_0",
            [55, 99, 40, 40],
            "coral_pale",
            32,
            false,
            3,
        )
        .portal("portal_target"),
        surface("signal_icon_0", [67, 111, 16, 16], "negative", 12, false, 4)
            .graphic(DashboardGraphic::Warning)
            .portal("portal_target"),
        text(
            "signal_title_0",
            "Error rate spike",
            [109, 100, 180, 26],
            13,
            false,
            "primary_text",
        )
        .portal("portal_target"),
        text(
            "signal_body_0",
            "API errors above 1%",
            [109, 122, 180, 24],
            12,
            false,
            "secondary_text",
        )
        .portal("portal_target"),
        text(
            "signal_time_0",
            "12m ago",
            [275, 100, 67, 22],
            11,
            false,
            "secondary_text",
        )
        .portal("portal_target"),
        surface(
            "signal_icon_bg_1",
            [55, 162, 40, 40],
            "amber_pale",
            32,
            false,
            3,
        )
        .portal("portal_target"),
        surface("signal_icon_1", [67, 174, 16, 16], "caution", 12, false, 4)
            .graphic(DashboardGraphic::Bars)
            .portal("portal_target"),
        text(
            "signal_title_1",
            "Latency increase",
            [109, 163, 180, 26],
            13,
            false,
            "primary_text",
        )
        .portal("portal_target"),
        text(
            "signal_body_1",
            "Checkout flow > 800ms",
            [109, 185, 180, 24],
            12,
            false,
            "secondary_text",
        )
        .portal("portal_target"),
        text(
            "signal_time_1",
            "47m ago",
            [275, 163, 67, 22],
            11,
            false,
            "secondary_text",
        )
        .portal("portal_target"),
        surface(
            "signal_icon_bg_2",
            [55, 225, 40, 40],
            "mint_pale",
            32,
            false,
            3,
        )
        .portal("portal_target"),
        surface("signal_icon_2", [67, 237, 16, 16], "positive", 12, false, 4)
            .graphic(DashboardGraphic::Check)
            .portal("portal_target"),
        text(
            "signal_title_2",
            "Recovery confirmed",
            [109, 226, 180, 26],
            13,
            false,
            "primary_text",
        )
        .portal("portal_target"),
        text(
            "signal_body_2",
            "Search latency normal",
            [109, 248, 180, 24],
            12,
            false,
            "secondary_text",
        )
        .portal("portal_target"),
        text(
            "signal_time_2",
            "2h ago",
            [275, 226, 67, 22],
            11,
            false,
            "secondary_text",
        )
        .portal("portal_target"),
    ]
}
