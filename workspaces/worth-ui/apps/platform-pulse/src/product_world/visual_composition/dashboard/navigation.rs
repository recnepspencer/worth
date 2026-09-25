//! The sidebar, the masthead, and the greeting. The sidebar keeps its width
//! and its footer keeps to the bottom; the masthead fills the width beside
//! the sidebar, its controls keeping to the right.
use super::frame::{Edge, VIEWPORT};
use super::page;
use super::{surface, text, DashboardElement, DashboardGraphic};

pub(super) fn elements() -> Vec<DashboardElement> {
    let greeting = page::greeting();
    authored()
        .into_iter()
        .map(|element| match element.id {
            "service_eyebrow" | "service_title" => greeting.place(element, Edge::Both, Edge::Start),
            id => match viewport_edges(id) {
                Some((horizontal, vertical)) => VIEWPORT.place(element, horizontal, vertical),
                None => element,
            },
        })
        .collect()
}

/// Which viewport edges the chrome that moves keeps its insets from. The
/// brand and navigation keep to the top left, where they are authored.
fn viewport_edges(id: &str) -> Option<(Edge, Edge)> {
    match id {
        "evidence_rail" => Some((Edge::Start, Edge::Both)),
        "source_signal_active" | "runtime_badge" | "lower_shelf_divider" | "status_text" => {
            Some((Edge::Start, Edge::End))
        }
        "masthead_border" => Some((Edge::Both, Edge::Start)),
        "search_field" | "search_label" | "search_icon" | "portal_target" | "portal_bell"
        | "notification_dot" | "avatar" | "avatar_label" => Some((Edge::End, Edge::Start)),
        _ => None,
    }
}

fn authored() -> Vec<DashboardElement> {
    vec![
        surface("seed", [0, 0, 1536, 1024], "canvas", 0, false, 0),
        surface(
            "evidence_rail",
            [0, 0, page::PLATFORM_PULSE_SIDEBAR_WIDTH, 1024],
            "navigation_surface",
            0,
            false,
            1,
        ),
        surface(
            "evidence_border",
            [12, 91, 211, 51],
            "nav_selected",
            12,
            false,
            2,
        ),
        text(
            "brand",
            "Platform Pulse",
            [26, 26, 204, 40],
            24,
            true,
            "action_text",
        ),
        text(
            "evidence_title",
            "Overview",
            [72, 106, 145, 28],
            16,
            false,
            "action_text",
        ),
        surface("nav_icon_0", [30, 106, 23, 23], "action_text", 12, false, 3)
            .graphic(DashboardGraphic::Home),
        text(
            "evidence_body",
            "Activity",
            [72, 166, 145, 28],
            16,
            false,
            "nav_text",
        ),
        surface("nav_icon_1", [30, 166, 23, 23], "nav_text", 12, false, 3)
            .graphic(DashboardGraphic::Activity),
        text(
            "source_signal_title",
            "Signals",
            [72, 226, 145, 28],
            16,
            false,
            "nav_text",
        ),
        surface("nav_icon_2", [30, 226, 23, 23], "nav_text", 12, false, 3)
            .graphic(DashboardGraphic::Signals),
        text(
            "evidence_service_label",
            "Deployments",
            [72, 286, 145, 28],
            16,
            false,
            "nav_text",
        ),
        surface("nav_icon_3", [30, 286, 23, 23], "nav_text", 12, false, 3)
            .graphic(DashboardGraphic::Cube),
        text(
            "evidence_service_body",
            "Settings",
            [72, 346, 145, 28],
            16,
            false,
            "nav_text",
        ),
        surface("nav_icon_4", [30, 346, 23, 23], "nav_text", 12, false, 3)
            .graphic(DashboardGraphic::Settings),
        surface(
            "source_signal_active",
            [26, 901, 9, 9],
            "positive",
            32,
            false,
            3,
        ),
        text(
            "runtime_badge",
            "All systems operational",
            [50, 897, 166, 22],
            12,
            false,
            "nav_text",
        ),
        surface(
            "lower_shelf_divider",
            [26, 931, 183, 1],
            "nav_selected",
            0,
            false,
            2,
        ),
        text(
            "status_text",
            "Build. Ship. Observe.\nA healthier internet.",
            [26, 956, 186, 44],
            13,
            false,
            "secondary_text",
        )
        .wrapped(),
        surface(
            "masthead_border",
            [
                page::PLATFORM_PULSE_SIDEBAR_WIDTH,
                page::PLATFORM_PULSE_MASTHEAD_HEIGHT - 1,
                1300,
                1,
            ],
            "structural_rule",
            0,
            false,
            1,
        ),
        surface(
            "search_field",
            [1054, 12, 322, 40],
            "search_fill",
            12,
            true,
            2,
        ),
        text(
            "search_label",
            "Search anything...",
            [1099, 12, 230, 40],
            14,
            false,
            "secondary_text",
        ),
        surface(
            "search_icon",
            [1069, 24, 15, 15],
            "secondary_text",
            12,
            false,
            3,
        )
        .graphic(DashboardGraphic::Search),
        surface(
            "portal_target",
            [1400, 12, 40, 40],
            "search_fill",
            32,
            false,
            3,
        )
        .action("open_signals"),
        surface(
            "portal_bell",
            [1410, 22, 20, 20],
            "primary_text",
            12,
            false,
            4,
        )
        .graphic(DashboardGraphic::Bell),
        surface(
            "notification_dot",
            [1429, 16, 10, 10],
            "negative",
            32,
            false,
            5,
        ),
        surface("avatar", [1466, 12, 40, 40], "lavender_pale", 32, false, 2),
        text(
            "avatar_label",
            "JD",
            [1466, 12, 40, 40],
            14,
            false,
            "primary_text",
        )
        .centered(),
        text(
            "service_eyebrow",
            "Good afternoon",
            [266, 81, 1240, 24],
            15,
            false,
            "secondary_text",
        ),
        text(
            "service_title",
            "Here’s what’s happening on Platform Pulse.",
            [266, 104, 1240, 43],
            25,
            true,
            "primary_text",
        ),
    ]
}
