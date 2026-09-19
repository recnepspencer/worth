use super::{surface, text, DashboardElement, DashboardGraphic};

pub(super) fn elements() -> Vec<DashboardElement> {
    vec![
        surface("review_shadow", [0, 0, 593, 516], "shadow", 12, false, 1).graphic(DashboardGraphic::Shadow).portal("review_target"),
        surface("review_surface", [36, 36, 521, 444], "raised_surface", 16, false, 2).portal("review_target"),
        text("review_title", "Review deployment", [67, 66, 431, 43], 26, true, "primary_text").portal("review_target"),
        text("review_body", "You’re about to deploy a new version to production. Please review the changes below before proceeding.", [67, 114, 461, 51], 15, false, "secondary_text").wrapped().portal("review_target"),
        surface("review_close_target", [505, 58, 36, 36], "raised_surface", 8, false, 3).portal("review_target").action("close"),
        surface("review_close_icon", [516, 69, 14, 14], "secondary_text", 12, false, 4).graphic(DashboardGraphic::Close).portal("review_target"),
        surface("review_changes", [67, 177, 460, 206], "inset_fill", 12, false, 3).portal("review_target"),
        surface("change_bg_0", [83, 197, 32, 32], "mint_pale", 32, false, 4).portal("review_target"),
        surface("change_icon_0", [91, 205, 16, 16], "positive", 12, false, 5).graphic(DashboardGraphic::Plus).portal("review_target"),
        text("change_title_0", "Update pricing model", [133, 195, 318, 25], 13, false, "primary_text").portal("review_target"),
        text("change_body_0", "Modifies subscription tiers and billing logic", [133, 216, 368, 23], 12, false, "secondary_text").portal("review_target"),
        text("change_files_0", "3 files", [465, 205, 58, 22], 12, false, "secondary_text").portal("review_target"),
        surface("change_rule_0", [133, 248, 378, 1], "grid", 0, false, 4).portal("review_target"),
        surface("change_bg_1", [83, 264, 32, 32], "lavender_pale", 32, false, 4).portal("review_target"),
        surface("change_icon_1", [91, 272, 16, 16], "principal_accent", 12, false, 5).graphic(DashboardGraphic::Pencil).portal("review_target"),
        text("change_title_1", "Improve error handling", [133, 262, 318, 25], 13, false, "primary_text").portal("review_target"),
        text("change_body_1", "Adds retries and better logging", [133, 283, 368, 23], 12, false, "secondary_text").portal("review_target"),
        text("change_files_1", "5 files", [465, 272, 58, 22], 12, false, "secondary_text").portal("review_target"),
        surface("change_rule_1", [133, 315, 378, 1], "grid", 0, false, 4).portal("review_target"),
        surface("change_bg_2", [83, 331, 32, 32], "coral_pale", 32, false, 4).portal("review_target"),
        surface("change_icon_2", [91, 339, 16, 16], "negative", 12, false, 5).graphic(DashboardGraphic::Minus).portal("review_target"),
        text("change_title_2", "Remove legacy feature", [133, 329, 318, 25], 13, false, "primary_text").portal("review_target"),
        text("change_body_2", "Deprecates the old recommendations endpoint", [133, 350, 368, 23], 12, false, "secondary_text").portal("review_target"),
        text("change_files_2", "2 files", [465, 339, 58, 22], 12, false, "secondary_text").portal("review_target"),
        surface("review_cancel_target", [195, 403, 109, 48], "raised_surface", 10, true, 3).portal("review_target").action("close"),
        text("review_cancel_label", "Cancel", [195, 403, 109, 48], 13, false, "primary_text").centered().portal("review_target"),
        surface("review_primary_target", [316, 403, 211, 48], "action_fill", 10, false, 3).portal("review_target").action("approve"),
        text("review_primary_label", "Approve deployment", [316, 403, 211, 48], 13, false, "action_text").centered().portal("review_target"),
    ]
}
