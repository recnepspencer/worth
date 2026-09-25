//! The Deployments panel: a heading, a "View all" link, and four entries,
//! each keeping its concept padding from the panel as the panel resizes.
use super::frame::Edge;
use super::page::Panel;
use super::{surface, text, DashboardElement, DashboardGraphic};

pub(super) fn elements() -> Vec<DashboardElement> {
    let frame = Panel::Deployments.frame();
    authored()
        .into_iter()
        .map(|element| {
            let horizontal = horizontal_edge(element.id);
            let vertical = if element.id == "deployments_card" {
                Edge::Both
            } else {
                Edge::Start
            };
            frame.place(element, horizontal, vertical)
        })
        .collect()
}

/// Which panel edge each element keeps its inset from: the icons lead, the
/// status badges and the link trail, and the rules and entry text take the
/// width between.
fn horizontal_edge(id: &str) -> Edge {
    match id {
        "deployments_card" => Edge::Both,
        "deployments_heading" => Edge::Start,
        "deployments_all" | "review_target" | "projected_status" => Edge::End,
        _ => match id.rsplit_once('_').map(|(stem, _)| stem) {
            Some("deploy_icon_bg" | "deploy_icon") => Edge::Start,
            Some("deploy_rule" | "deploy_version" | "deploy_detail") => Edge::Both,
            Some("deploy_badge" | "deploy_status") => Edge::End,
            _ => unreachable!("deployment element {id} declares no edge"),
        },
    }
}

fn authored() -> Vec<DashboardElement> {
    vec![
        surface(
            "deployments_card",
            [1095, 636, 411, 346],
            "raised_surface",
            12,
            true,
            2,
        ),
        text(
            "deployments_heading",
            "Deployments",
            [1120, 653, 215, 35],
            18,
            true,
            "primary_text",
        ),
        text(
            "deployments_all",
            "View all",
            [1432, 656, 64, 25],
            13,
            false,
            "principal_accent",
        ),
        surface("deploy_rule_0", [1120, 690, 361, 1], "grid", 0, false, 3),
        surface(
            "deploy_icon_bg_0",
            [1127, 710, 25, 25],
            "positive",
            32,
            false,
            3,
        ),
        surface(
            "deploy_icon_0",
            [1132, 715, 15, 15],
            "action_text",
            12,
            false,
            4,
        )
        .graphic(DashboardGraphic::Check),
        text(
            "deploy_version_0",
            "v2.4.1",
            [1175, 705, 177, 25],
            14,
            false,
            "primary_text",
        ),
        text(
            "deploy_detail_0",
            "Production · 2m ago",
            [1175, 727, 211, 25],
            13,
            false,
            "secondary_text",
        ),
        surface(
            "deploy_badge_0",
            [1392, 712, 85, 27],
            "mint_pale",
            9,
            false,
            3,
        ),
        text(
            "deploy_status_0",
            "Deployed",
            [1392, 712, 85, 27],
            11,
            false,
            "positive",
        )
        .centered(),
        surface("deploy_rule_1", [1120, 757, 361, 1], "grid", 0, false, 3),
        surface(
            "deploy_icon_bg_1",
            [1127, 777, 25, 25],
            "lavender_pale",
            32,
            false,
            3,
        ),
        surface(
            "deploy_icon_1",
            [1132, 782, 15, 15],
            "principal_accent",
            12,
            false,
            4,
        )
        .graphic(DashboardGraphic::Clock),
        text(
            "deploy_version_1",
            "v2.5.0",
            [1175, 772, 177, 25],
            14,
            false,
            "primary_text",
        ),
        text(
            "deploy_detail_1",
            "Staging · in 3 hours",
            [1175, 794, 211, 25],
            13,
            false,
            "secondary_text",
        ),
        surface(
            "review_target",
            [1392, 779, 85, 27],
            "lavender_pale",
            9,
            false,
            3,
        )
        .action("open_review"),
        text(
            "projected_status",
            "Scheduled",
            [1392, 779, 85, 27],
            11,
            false,
            "principal_accent",
        )
        .centered(),
        surface("deploy_rule_2", [1120, 824, 361, 1], "grid", 0, false, 3),
        surface(
            "deploy_icon_bg_2",
            [1127, 844, 25, 25],
            "positive",
            32,
            false,
            3,
        ),
        surface(
            "deploy_icon_2",
            [1132, 849, 15, 15],
            "action_text",
            12,
            false,
            4,
        )
        .graphic(DashboardGraphic::Check),
        text(
            "deploy_version_2",
            "v2.4.0",
            [1175, 839, 177, 25],
            14,
            false,
            "primary_text",
        ),
        text(
            "deploy_detail_2",
            "Production · 1 day ago",
            [1175, 861, 211, 25],
            13,
            false,
            "secondary_text",
        ),
        surface(
            "deploy_badge_2",
            [1392, 846, 85, 27],
            "mint_pale",
            9,
            false,
            3,
        ),
        text(
            "deploy_status_2",
            "Deployed",
            [1392, 846, 85, 27],
            11,
            false,
            "positive",
        )
        .centered(),
        surface("deploy_rule_3", [1120, 891, 361, 1], "grid", 0, false, 3),
        surface(
            "deploy_icon_bg_3",
            [1127, 911, 25, 25],
            "negative",
            32,
            false,
            3,
        ),
        surface(
            "deploy_icon_3",
            [1132, 916, 15, 15],
            "action_text",
            12,
            false,
            4,
        )
        .graphic(DashboardGraphic::Close),
        text(
            "deploy_version_3",
            "v2.3.8",
            [1175, 906, 177, 25],
            14,
            false,
            "primary_text",
        ),
        text(
            "deploy_detail_3",
            "Production · 3 days ago",
            [1175, 928, 211, 25],
            13,
            false,
            "secondary_text",
        ),
        surface(
            "deploy_badge_3",
            [1392, 913, 85, 27],
            "coral_pale",
            9,
            false,
            3,
        ),
        text(
            "deploy_status_3",
            "Failed",
            [1392, 913, 85, 27],
            11,
            false,
            "negative",
        )
        .centered(),
    ]
}
