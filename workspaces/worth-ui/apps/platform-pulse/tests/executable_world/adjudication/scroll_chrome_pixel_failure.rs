use std::fmt;

/// Why the Recent activity scroll chrome or its content failed the pixel oracle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScrollChromePixelFailure {
    WheelLinesUninterpretable,
    ThumbAbsent {
        column_px: u32,
    },
    ThumbTopMismatch {
        expected_px: i64,
        observed_px: u32,
    },
    ThumbLengthMismatch {
        expected_px: i64,
        observed_px: u32,
    },
    ContentInkAbsent,
    ContentUnchanged {
        column_px: u32,
    },
    ContentShiftMismatch {
        column_px: u32,
        shift_px: i64,
        compared_rows: usize,
        mismatched_rows: usize,
    },
}

impl fmt::Display for ScrollChromePixelFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WheelLinesUninterpretable => {
                formatter.write_str("the platform wheel-lines setting is neither a count nor absent")
            }
            Self::ThumbAbsent { column_px } => {
                write!(formatter, "no thumb-toned run on gutter column x={column_px}")
            }
            Self::ThumbTopMismatch {
                expected_px,
                observed_px,
            } => write!(
                formatter,
                "thumb top expected at y={expected_px} px, observed y={observed_px} px"
            ),
            Self::ThumbLengthMismatch {
                expected_px,
                observed_px,
            } => write!(
                formatter,
                "thumb length expected {expected_px} px, observed {observed_px} px"
            ),
            Self::ContentInkAbsent => {
                formatter.write_str("no content column carries ink inside the viewport")
            }
            Self::ContentUnchanged { column_px } => {
                write!(formatter, "content column x={column_px} did not change at all")
            }
            Self::ContentShiftMismatch {
                column_px,
                shift_px,
                compared_rows,
                mismatched_rows,
            } => write!(
                formatter,
                "content column x={column_px} is not the baseline moved up {shift_px} px: {mismatched_rows} of {compared_rows} rows disagree"
            ),
        }
    }
}
