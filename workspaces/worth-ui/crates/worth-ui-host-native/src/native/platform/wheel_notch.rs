use super::wheel_scroll_lines_store::{
    read_wheel_scroll_lines_setting, UiNativeWheelScrollLinesSetting,
};

/// The spelling the platform uses for "one page per notch".
///
/// Windows documents `WHEEL_PAGESCROLL` as `u32::MAX`, and its backing registry
/// value spells the same answer `-1`. Both are the page answer, and neither is
/// a line count.
const WHEEL_PAGE_SCROLL_TEXT: &str = "-1";
const WHEEL_PAGE_SCROLL_COUNT: u32 = u32::MAX;

/// What the platform says one turn of the wheel notch should scroll.
///
/// This is the platform's own answer, not a normalized one: `Lines(0)` and a
/// count no line-denominated reader can carry are both reported exactly as the
/// platform gave them, and the event profile decides what to do about it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeWheelNotchReport {
    /// The platform reported this many lines per notch.
    Lines(u32),
    /// The platform asked for one page per notch (Windows `WHEEL_PAGESCROLL`).
    Page,
    /// The platform stated an answer that is neither a count nor the page
    /// answer, so this host read a setting it cannot interpret.
    Uninterpretable,
    /// No wheel notch setting reached this host at all.
    Unavailable,
}

/// Observe the platform's wheel notch setting.
///
/// The read and the interpretation are separate on purpose: the store answers
/// with raw text, and every rule for turning that text into a notch answer
/// lives in [`interpret_wheel_scroll_lines`], which a test can exercise without
/// touching a registry.
pub(crate) fn observe_wheel_notch_report() -> UiNativeWheelNotchReport {
    interpret_wheel_scroll_lines(&read_wheel_scroll_lines_setting())
}

/// Turn the platform's raw wheel-lines answer into a notch report.
pub(crate) fn interpret_wheel_scroll_lines(
    setting: &UiNativeWheelScrollLinesSetting,
) -> UiNativeWheelNotchReport {
    let stated = match setting {
        UiNativeWheelScrollLinesSetting::Absent => return UiNativeWheelNotchReport::Unavailable,
        UiNativeWheelScrollLinesSetting::Unreadable => {
            return UiNativeWheelNotchReport::Uninterpretable
        }
        UiNativeWheelScrollLinesSetting::Stated(text) => text.trim(),
    };
    if stated == WHEEL_PAGE_SCROLL_TEXT {
        return UiNativeWheelNotchReport::Page;
    }
    match stated.parse::<u32>() {
        Ok(WHEEL_PAGE_SCROLL_COUNT) => UiNativeWheelNotchReport::Page,
        Ok(count) => UiNativeWheelNotchReport::Lines(count),
        Err(_) => UiNativeWheelNotchReport::Uninterpretable,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        interpret_wheel_scroll_lines, UiNativeWheelNotchReport, UiNativeWheelScrollLinesSetting,
    };

    fn stated(text: &str) -> UiNativeWheelScrollLinesSetting {
        UiNativeWheelScrollLinesSetting::Stated(text.to_owned())
    }

    /// Every answer the store can give reaches exactly one notch report, and a
    /// count the host cannot read never launders itself into a number.
    #[test]
    fn each_stored_wheel_setting_reaches_one_notch_report() {
        for (setting, expected) in [
            (stated("3"), UiNativeWheelNotchReport::Lines(3)),
            (stated("1"), UiNativeWheelNotchReport::Lines(1)),
            (stated(" 7 "), UiNativeWheelNotchReport::Lines(7)),
            (stated("0"), UiNativeWheelNotchReport::Lines(0)),
            (stated("-1"), UiNativeWheelNotchReport::Page),
            (stated("4294967295"), UiNativeWheelNotchReport::Page),
            (stated("65536"), UiNativeWheelNotchReport::Lines(65_536)),
            (stated(""), UiNativeWheelNotchReport::Uninterpretable),
            (stated("three"), UiNativeWheelNotchReport::Uninterpretable),
            (stated("-2"), UiNativeWheelNotchReport::Uninterpretable),
            (stated("3.5"), UiNativeWheelNotchReport::Uninterpretable),
            (
                UiNativeWheelScrollLinesSetting::Unreadable,
                UiNativeWheelNotchReport::Uninterpretable,
            ),
            (
                UiNativeWheelScrollLinesSetting::Absent,
                UiNativeWheelNotchReport::Unavailable,
            ),
        ] {
            assert_eq!(
                interpret_wheel_scroll_lines(&setting),
                expected,
                "{setting:?}"
            );
        }
    }

    /// An absent setting and an unreadable one are two different facts, because
    /// one means the host never learned the answer and the other means the host
    /// read an answer it must refuse.
    #[test]
    fn an_absent_setting_is_not_the_same_fact_as_an_unreadable_one() {
        assert_ne!(
            interpret_wheel_scroll_lines(&UiNativeWheelScrollLinesSetting::Absent),
            interpret_wheel_scroll_lines(&UiNativeWheelScrollLinesSetting::Unreadable)
        );
    }
}
