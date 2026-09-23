use worth_ui_host_contract::{
    UiHostScrollDeltaPrecision, UiHostScrollLineCountBasis, UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
};

use super::super::platform::{observe_wheel_notch_report, UiNativeWheelNotchReport};
use super::super::UiNativeInputObservationStop;

#[derive(Clone, Copy)]
pub(crate) struct UiNativeEventProfile {
    pub(super) scale_factor: f64,
    pub(super) scale_micros: u32,
    pub(super) physical_size: [u32; 2],
    /// The platform wheel notch setting, kept exactly as reported. The
    /// precision and the per-notch scale below are derived from it, never
    /// stored beside it.
    pub(super) wheel_notch: UiNativeWheelNotchReport,
}

impl UiNativeEventProfile {
    /// The precision a wheel notch delta observed under this profile carries.
    ///
    /// The host states the line count it applied and why it applied that count.
    /// It never converts notches into content distance: how tall a line is
    /// belongs to whoever owns the content, not to the host.
    pub(super) fn wheel_precision(self) -> UiHostScrollDeltaPrecision {
        match self.wheel_notch {
            UiNativeWheelNotchReport::Lines(0) => UiHostScrollDeltaPrecision::Line {
                platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                basis: UiHostScrollLineCountBasis::DefaultedAfterInvalid,
            },
            UiNativeWheelNotchReport::Lines(lines) => match u16::try_from(lines) {
                Ok(platform_lines_per_notch) => UiHostScrollDeltaPrecision::Line {
                    platform_lines_per_notch,
                    basis: UiHostScrollLineCountBasis::PlatformReported,
                },
                Err(_) => UiHostScrollDeltaPrecision::Line {
                    platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                    basis: UiHostScrollLineCountBasis::DefaultedAfterInvalid,
                },
            },
            UiNativeWheelNotchReport::Page => UiHostScrollDeltaPrecision::Page,
            UiNativeWheelNotchReport::Uninterpretable => UiHostScrollDeltaPrecision::Line {
                platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                basis: UiHostScrollLineCountBasis::DefaultedAfterInvalid,
            },
            UiNativeWheelNotchReport::Unavailable => UiHostScrollDeltaPrecision::Line {
                platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                basis: UiHostScrollLineCountBasis::DefaultedAfterMissing,
            },
        }
    }

    /// Units of the declared precision that one wheel notch turns.
    ///
    /// A line precision turns as many lines as it states. A precision that
    /// states no line count turns one whole unit of whatever it does name.
    pub(super) fn wheel_units_per_notch(self) -> f64 {
        match self.wheel_precision().lines_per_notch() {
            Some(lines_per_notch) => f64::from(lines_per_notch),
            None => 1.0,
        }
    }
}

pub(super) fn event_profile(
    scale_factor: f64,
    physical_size: [u32; 2],
) -> Result<UiNativeEventProfile, UiNativeInputObservationStop> {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return Err(UiNativeInputObservationStop::InvalidScale);
    }
    let micros = (scale_factor * 1_000_000.0).round();
    if !micros.is_finite() || !(1.0..=f64::from(u32::MAX)).contains(&micros) {
        return Err(UiNativeInputObservationStop::InvalidScale);
    }
    Ok(UiNativeEventProfile {
        scale_factor,
        scale_micros: micros as u32,
        physical_size,
        wheel_notch: observe_wheel_notch_report(),
    })
}

pub(super) fn logical_subpixels(value: u32, scale_factor: f64) -> i64 {
    (f64::from(value) / scale_factor * 1_000.0).round() as i64
}

#[cfg(test)]
mod tests {
    use super::{event_profile, UiNativeEventProfile, UiNativeWheelNotchReport};
    use worth_ui_host_contract::{
        UiHostScrollDeltaPrecision, UiHostScrollLineCountBasis,
        UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
    };

    fn profile_reporting(wheel_notch: UiNativeWheelNotchReport) -> UiNativeEventProfile {
        UiNativeEventProfile {
            wheel_notch,
            ..event_profile(2.0, [800, 600]).expect("a finite positive scale admits a profile")
        }
    }

    /// Every platform answer reaches the observation as a stated count and a
    /// stated reason, and the notch scale follows the count that was stated.
    #[test]
    fn each_platform_wheel_answer_states_its_count_and_why() {
        let default = u32::from(UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH);
        for (report, expected, units) in [
            (
                UiNativeWheelNotchReport::Lines(1),
                UiHostScrollDeltaPrecision::Line {
                    platform_lines_per_notch: 1,
                    basis: UiHostScrollLineCountBasis::PlatformReported,
                },
                1.0,
            ),
            (
                UiNativeWheelNotchReport::Lines(default),
                UiHostScrollDeltaPrecision::Line {
                    platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                    basis: UiHostScrollLineCountBasis::PlatformReported,
                },
                f64::from(default),
            ),
            (
                UiNativeWheelNotchReport::Lines(u32::from(u16::MAX)),
                UiHostScrollDeltaPrecision::Line {
                    platform_lines_per_notch: u16::MAX,
                    basis: UiHostScrollLineCountBasis::PlatformReported,
                },
                f64::from(u16::MAX),
            ),
            (
                UiNativeWheelNotchReport::Lines(0),
                UiHostScrollDeltaPrecision::Line {
                    platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                    basis: UiHostScrollLineCountBasis::DefaultedAfterInvalid,
                },
                f64::from(default),
            ),
            (
                UiNativeWheelNotchReport::Lines(u32::from(u16::MAX) + 1),
                UiHostScrollDeltaPrecision::Line {
                    platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                    basis: UiHostScrollLineCountBasis::DefaultedAfterInvalid,
                },
                f64::from(default),
            ),
            (
                UiNativeWheelNotchReport::Page,
                UiHostScrollDeltaPrecision::Page,
                1.0,
            ),
            (
                UiNativeWheelNotchReport::Uninterpretable,
                UiHostScrollDeltaPrecision::Line {
                    platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                    basis: UiHostScrollLineCountBasis::DefaultedAfterInvalid,
                },
                f64::from(default),
            ),
            (
                UiNativeWheelNotchReport::Unavailable,
                UiHostScrollDeltaPrecision::Line {
                    platform_lines_per_notch: UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                    basis: UiHostScrollLineCountBasis::DefaultedAfterMissing,
                },
                f64::from(default),
            ),
        ] {
            let profile = profile_reporting(report);
            assert_eq!(profile.wheel_precision(), expected, "{report:?}");
            assert_eq!(profile.wheel_units_per_notch(), units, "{report:?}");
        }
    }

    /// A page-scroll setting never launders itself into a line count.
    #[test]
    fn page_scrolling_is_never_reported_as_a_line_count() {
        let precision = profile_reporting(UiNativeWheelNotchReport::Page).wheel_precision();

        assert_eq!(precision.lines_per_notch(), None);
        assert_eq!(precision.line_count_basis(), None);
    }
}
