use std::collections::HashSet;

use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedPaintCommandIdentity};
use worth_ui_host_headless::UiHeadlessMountedFrameTranscript;

pub(super) fn exact_portal_child_commands(
    transcript: &UiHeadlessMountedFrameTranscript,
    portal_child: UiMountedInstanceIdentity,
    portal: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
) -> HashSet<UiMountedPaintCommandIdentity> {
    let fills = transcript
        .filled_rects()
        .iter()
        .filter(|mechanic| mechanic.mounted_instance() == portal_child)
        .inspect(|mechanic| assert_child_clip(mechanic, portal))
        .map(|mechanic| mechanic.command_identity())
        .collect::<Vec<_>>();
    let texts = transcript
        .semantic_text()
        .iter()
        .filter(|mechanic| mechanic.mounted_instance() == portal_child)
        .collect::<Vec<_>>();

    assert_eq!(fills.len(), 1, "the Portal child emits its authored fill");
    assert_eq!(
        texts.len(),
        2,
        "the real Portal child must emit semantic text into the sampled command group; observed semantic text: {:?}",
        transcript
            .semantic_text()
            .iter()
            .map(|mechanic| (mechanic.mounted_instance(), mechanic.text()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        texts
            .iter()
            .map(|mechanic| mechanic.text())
            .collect::<HashSet<_>>(),
        HashSet::from(["Portal motion content", " "]),
        "the child emits its authored value and retained posture rows"
    );

    fills
        .into_iter()
        .chain(
            texts
                .into_iter()
                .map(|mechanic| mechanic.command_identity()),
        )
        .collect()
}

fn assert_child_clip(
    child: &worth_ui_host_headless::UiHeadlessFilledRectMechanic,
    portal: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
) {
    // The authored child's own clip starts inside the Portal. Its effective
    // clip is the intersection, not the entire Portal allocation.
    let child_bounds = child.bounds();
    let portal_bounds = portal.bounds();
    let x = child_bounds.x().max(portal_bounds.x());
    let y = child_bounds.y().max(portal_bounds.y());
    let right =
        (child_bounds.x() + child_bounds.width()).min(portal_bounds.x() + portal_bounds.width());
    let bottom =
        (child_bounds.y() + child_bounds.height()).min(portal_bounds.y() + portal_bounds.height());
    assert!(
        right > x && bottom > y,
        "the authored child remains visible"
    );
    let clip = child.clip_bounds();
    assert_eq!(clip.coordinate_space(), portal_bounds.coordinate_space());
    assert_eq!(
        [clip.x(), clip.y(), clip.width(), clip.height()],
        [x, y, right - x, bottom - y],
    );
}
