//! Observe only changed chrome identities carried by the accepted sample.
use super::UiNativeRetainedDrawList;
use worth_ui_host_contract::{UiMountedPaintCommandIdentity, UiMountedScrollChromeIdentity};

impl UiNativeRetainedDrawList {
    pub(in crate::native) fn sampled_chrome_observation(
        &self,
        identities: impl IntoIterator<Item = UiMountedPaintCommandIdentity>,
    ) -> Box<[(UiMountedScrollChromeIdentity, [i64; 4])]> {
        identities
            .into_iter()
            .filter_map(|identity| {
                let chrome = identity.scroll_chrome_identity()?;
                let change = self.sample_overrides.get(&identity).copied()?;
                let bounds = self.sampled_target_bounds(identity, Some(change)).ok()??;
                Some((
                    chrome,
                    [bounds.x(), bounds.y(), bounds.width(), bounds.height()]
                        .map(|value| (f64::from(value) * 1_000.0).round() as i64),
                ))
            })
            .collect()
    }
}
