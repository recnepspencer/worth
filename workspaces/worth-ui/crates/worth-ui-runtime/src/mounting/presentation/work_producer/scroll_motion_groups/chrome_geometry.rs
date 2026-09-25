//! Read chrome's retained frame geometry through its physically accepted track.
use super::*;
use worth_ui_host_contract::{UiMountedScrollChromeAxis, UiMountedScrollChromePart};

impl UiMountedPresentationState {
    pub(in crate::mounting) fn retained_scroll_chrome_geometry(
        &self,
        target: UiMotionTargetIdentity,
    ) -> Option<(
        UiMountedCanonicalBox,
        UiMountedCanonicalBox,
        UiMountedCanonicalBox,
    )> {
        let input = &self.scroll_motion_groups.groups.get(&target)?.input;
        let identity = [
            UiMountedScrollChromeAxis::Block,
            UiMountedScrollChromeAxis::Inline,
        ]
        .into_iter()
        .map(|axis| {
            UiMountedScrollChromeIdentity::from_runtime_mounting(
                input.owner,
                axis,
                UiMountedScrollChromePart::Track,
            )
        })
        .find(|identity| self.scroll_motion_groups.chrome.contains_key(identity))?;
        let track = self.scroll_motion_groups.chrome.get(&identity)?;
        let accepted =
            self.accepted_motion_layers(UiMountedPaintCommandIdentity::scroll_chrome(identity));
        // A region's own offset never moves its track. Its accepted Scroll
        // translation is therefore exactly the ancestor displacement of
        // viewport/content.
        let delta = accepted
            .scroll_transform()
            .map(|transform| {
                [
                    transform.sampled().x() - transform.source().x(),
                    transform.sampled().y() - transform.source().y(),
                ]
            })
            .unwrap_or([0.0; 2]);
        let translated = |bounds: UiMountedCanonicalBox| {
            UiMountedCanonicalBox::canonicalize(
                worth_ui_host_contract::UiMountedCanonicalBoxInput {
                    x: bounds.x() + delta[0],
                    y: bounds.y() + delta[1],
                    width: bounds.width(),
                    height: bounds.height(),
                    coordinate_space: bounds.coordinate_space(),
                },
            )
            .ok()
        };
        Some((
            translated(input.content)?,
            translated(input.viewport)?,
            accepted.scroll_clip().unwrap_or(track.clip),
        ))
    }
}
