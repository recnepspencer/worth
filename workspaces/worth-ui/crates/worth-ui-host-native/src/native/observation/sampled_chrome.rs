//! Read-only accepted sample geometry for joining external captures to attempts.
use super::UiNativeRetainedFrameObservation;
use worth_ui_host_contract::UiMountedScrollChromeIdentity;

impl UiNativeRetainedFrameObservation {
    pub(crate) fn with_sampled_chrome(
        mut self,
        chrome: Box<[(UiMountedScrollChromeIdentity, [i64; 4])]>,
    ) -> Self {
        if !chrome.is_empty() {
            self.accepted_qpc_100ns = qpc_100ns();
        }
        self.sampled_chrome = chrome;
        self
    }

    /// Host completion time on Windows' system-relative QPC clock. This is
    /// acceptance, not the compositor's independently observed visible time.
    pub const fn accepted_qpc_100ns(&self) -> Option<i64> {
        self.accepted_qpc_100ns
    }

    /// Changed chrome: diagnostic owner, inline axis, thumb part, and visible
    /// [x, y, width, height] in logical millipoints. Never live Scroll authority.
    /// The existing bounded observation history limits retained samples.
    pub fn sampled_chrome(&self) -> impl Iterator<Item = (u64, bool, bool, [i64; 4])> + '_ {
        self.sampled_chrome.iter().map(|(identity, bounds)| {
            (
                identity.owner_instance().diagnostic_value(),
                identity.axis() == worth_ui_host_contract::UiMountedScrollChromeAxis::Inline,
                identity.part() == worth_ui_host_contract::UiMountedScrollChromePart::Thumb,
                *bounds,
            )
        })
    }
}

#[cfg(windows)]
fn qpc_100ns() -> Option<i64> {
    let frequency = winsafe::QueryPerformanceFrequency().ok()?;
    let ticks = winsafe::QueryPerformanceCounter().ok()?;
    i64::try_from(
        i128::from(ticks)
            .checked_mul(10_000_000)?
            .checked_div(i128::from(frequency))?,
    )
    .ok()
}

#[cfg(not(windows))]
fn qpc_100ns() -> Option<i64> {
    None
}
