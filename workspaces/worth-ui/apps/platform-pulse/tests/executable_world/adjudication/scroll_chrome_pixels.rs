//! Independent pixel oracle for the Recent activity scroll chrome and the
//! content that travels under it.
//!
//! Every expectation is rectangle arithmetic from declared constants and the
//! authored content extent. Nothing here reads a thumb position, an offset or
//! a track length back from runtime chrome facts.
use crate::external_observation::NativeClientPixelCapture;

use super::scroll_chrome_pixel_failure::ScrollChromePixelFailure;

/// Recent activity viewport, host-surface logical points: [x, y, width, height].
const REGION: [u32; 4] = [290, 693, 768, 269];
/// Authored extent of the Recent activity content, logical points.
const CONTENT_EXTENT: [u32; 2] = [1_560, 672];
/// The product's declared line extent: one wheel line moves the list this far.
const LINE_EXTENT_POINTS: u32 = 20;
/// Runtime scroll chrome constants the spec declares: gutter, bar, minimum thumb.
const GUTTER_POINTS: f64 = 12.0;
const MINIMUM_THUMB_POINTS: f64 = 24.0;
/// The line count Windows assumes when its wheel setting is absent.
const DEFAULT_LINES_PER_NOTCH: u32 = 3;
/// Light theme scroll thumb tones: rest, hover and drag. The pointer sits over
/// the chrome during much of the journey, so any of the three is the thumb.
const THUMB_TONES: [[u8; 3]; 3] = [[176, 181, 196], [146, 152, 172], [112, 119, 142]];
const CHANNEL_TOLERANCE: u8 = 8;
/// Slack for one snapped edge, physical pixels.
const EDGE_TOLERANCE_PX: i64 = 3;
/// Content columns searched for ink, logical points from the viewport's left edge.
const INK_SEARCH_SPAN_POINTS: u32 = 40;
/// Anti-aliased edges may disagree after a fractional-point move.
const MAXIMUM_SHIFT_MISMATCH_PERMILLE: usize = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VerticalThumbEvidence {
    top_px: u32,
    length_px: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContentShiftEvidence {
    column_px: u32,
    shift_px: i64,
    compared_rows: usize,
    mismatched_rows: usize,
}

/// The platform's own answer for lines per wheel notch, read from the same
/// documented store the product reads, but by this oracle's own hand.
pub(crate) fn platform_wheel_lines_per_notch() -> Result<u32, ScrollChromePixelFailure> {
    let read = winsafe::HKEY::CURRENT_USER.RegGetValue(
        Some(r"Control Panel\Desktop"),
        Some("WheelScrollLines"),
        winsafe::co::RRF::RT_ANY,
    );
    match read {
        Err(_) | Ok(winsafe::RegistryValue::None) => Ok(DEFAULT_LINES_PER_NOTCH),
        Ok(winsafe::RegistryValue::Sz(text) | winsafe::RegistryValue::ExpandSz(text)) => text
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|count| (1..u32::MAX).contains(count))
            .ok_or(ScrollChromePixelFailure::WheelLinesUninterpretable),
        Ok(winsafe::RegistryValue::Dword(count)) if (1..u32::MAX).contains(&count) => Ok(count),
        Ok(_) => Err(ScrollChromePixelFailure::WheelLinesUninterpretable),
    }
}

/// How far one coarse wheel notch must move the list, logical points.
pub(crate) fn notch_points(lines_per_notch: u32) -> f64 {
    f64::from(lines_per_notch) * f64::from(LINE_EXTENT_POINTS)
}

/// Rectangle arithmetic for the vertical scroll chrome of the Recent activity
/// region, in logical points.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RecentActivityScrollGeometry;

impl RecentActivityScrollGeometry {
    pub(crate) fn max_offset_points(self) -> f64 {
        f64::from(CONTENT_EXTENT[1] - REGION[3])
    }

    /// The vertical track: the viewport height less the corner the block track
    /// surrenders because the content also overflows inline.
    fn track_length_points(self) -> f64 {
        let inline_overflows = CONTENT_EXTENT[0] > REGION[2];
        f64::from(REGION[3]) - if inline_overflows { GUTTER_POINTS } else { 0.0 }
    }

    pub(crate) fn thumb_length_points(self) -> f64 {
        let track = self.track_length_points();
        (track * f64::from(REGION[3]) / f64::from(CONTENT_EXTENT[1]))
            .max(MINIMUM_THUMB_POINTS)
            .min(track)
    }

    fn thumb_travel_points(self) -> f64 {
        self.track_length_points() - self.thumb_length_points()
    }

    pub(crate) fn thumb_top_points(self, offset_points: f64) -> f64 {
        f64::from(REGION[1]) + self.thumb_travel_points() * offset_points / self.max_offset_points()
    }

    pub(crate) fn thumb_center_points(self, offset_points: f64) -> [f64; 2] {
        [
            f64::from(REGION[0] + REGION[2]) - GUTTER_POINTS / 2.0,
            self.thumb_top_points(offset_points) + self.thumb_length_points() / 2.0,
        ]
    }

    /// The offset a thumb dragged `delta_points` along the track lands on.
    pub(crate) fn offset_after_thumb_drag(self, from_offset: f64, delta_points: f64) -> f64 {
        from_offset + delta_points * self.max_offset_points() / self.thumb_travel_points()
    }

    /// The scrolled viewport itself, logical points: [x, y, width, height].
    pub(crate) fn viewport_points(self) -> [f64; 4] {
        REGION.map(f64::from)
    }

    /// A point inside the scrolled content, clear of the chrome.
    pub(crate) fn content_interior_points(self) -> [f64; 2] {
        [
            f64::from(REGION[0]) + f64::from(REGION[2]) / 2.0,
            f64::from(REGION[1]) + f64::from(REGION[3]) / 4.0,
        ]
    }
}

/// Logical points to physical pixels at `dpi`, rounded to the nearest pixel.
pub(crate) fn physical_px(points: f64, dpi: u32) -> i64 {
    (points * f64::from(dpi) / 96.0).round() as i64
}

pub(crate) fn adjudicate_vertical_thumb(
    capture: &NativeClientPixelCapture,
    dpi: u32,
    offset_points: f64,
) -> Result<VerticalThumbEvidence, ScrollChromePixelFailure> {
    let geometry = RecentActivityScrollGeometry;
    let column_px = physical_px(geometry.thumb_center_points(0.0)[0], dpi) as u32;
    let track_top = physical_px(f64::from(REGION[1]), dpi).max(0) as u32;
    let track_bottom =
        physical_px(f64::from(REGION[1]) + geometry.track_length_points(), dpi).max(0) as u32;
    let scan_bottom = track_bottom.min(capture.height());
    let mut best: Option<(u32, u32)> = None;
    let mut run_start = None;
    for y in track_top..=scan_bottom {
        let is_thumb =
            y < scan_bottom && physical_pixel(capture, [column_px, y]).is_some_and(is_thumb_tone);
        match (is_thumb, run_start) {
            (true, None) => run_start = Some(y),
            (false, Some(start)) => {
                if best.is_none_or(|(_, length)| y - start > length) {
                    best = Some((start, y - start));
                }
                run_start = None;
            }
            _ => {}
        }
    }
    let (top_px, length_px) = best.ok_or(ScrollChromePixelFailure::ThumbAbsent { column_px })?;
    let expected_top = physical_px(geometry.thumb_top_points(offset_points), dpi);
    if (i64::from(top_px) - expected_top).abs() > EDGE_TOLERANCE_PX {
        return Err(ScrollChromePixelFailure::ThumbTopMismatch {
            expected_px: expected_top,
            observed_px: top_px,
        });
    }
    let expected_length = physical_px(geometry.thumb_length_points(), dpi);
    let expected_bottom = expected_top + expected_length;
    let bottom_visible = expected_bottom + EDGE_TOLERANCE_PX < i64::from(scan_bottom);
    if bottom_visible && (i64::from(length_px) - expected_length).abs() > EDGE_TOLERANCE_PX {
        return Err(ScrollChromePixelFailure::ThumbLengthMismatch {
            expected_px: expected_length,
            observed_px: length_px,
        });
    }
    Ok(VerticalThumbEvidence { top_px, length_px })
}

/// Prove the content under the viewport moved up by `shift_points` between
/// `before` and `after`: a column of row ink in `after` must equal the same
/// column of `before` read `shift` pixels lower, and must differ from `before`
/// read in place.
pub(crate) fn adjudicate_content_shift(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
    dpi: u32,
    shift_points: f64,
) -> Result<ContentShiftEvidence, ScrollChromePixelFailure> {
    let top = physical_px(f64::from(REGION[1]), dpi).max(0) as u32;
    let bottom = physical_px(
        f64::from(REGION[1]) + RecentActivityScrollGeometry.track_length_points(),
        dpi,
    )
    .max(0) as u32;
    let bottom = bottom.min(before.height()).min(after.height());
    let column_px = ink_column(before, top, bottom, dpi)?;
    let nominal_shift = physical_px(shift_points, dpi);
    let unshifted = mismatches(before, after, column_px, top, bottom, 0);
    if unshifted.1 == 0 {
        return Err(ScrollChromePixelFailure::ContentUnchanged { column_px });
    }
    let (shift_px, (compared_rows, mismatched_rows)) = (nominal_shift - 1..=nominal_shift + 1)
        .map(|shift| {
            (
                shift,
                mismatches(before, after, column_px, top, bottom, shift),
            )
        })
        .min_by_key(|(_, (_, mismatched))| *mismatched)
        .ok_or(ScrollChromePixelFailure::ContentInkAbsent)?;
    if compared_rows == 0
        || mismatched_rows * 1_000 > compared_rows * MAXIMUM_SHIFT_MISMATCH_PERMILLE
        || mismatched_rows >= unshifted.1
    {
        return Err(ScrollChromePixelFailure::ContentShiftMismatch {
            column_px,
            shift_px,
            compared_rows,
            mismatched_rows,
        });
    }
    Ok(ContentShiftEvidence {
        column_px,
        shift_px,
        compared_rows,
        mismatched_rows,
    })
}

/// The content column carrying the most ink against its own top pixel.
fn ink_column(
    capture: &NativeClientPixelCapture,
    top: u32,
    bottom: u32,
    dpi: u32,
) -> Result<u32, ScrollChromePixelFailure> {
    let first = physical_px(f64::from(REGION[0]) + 1.0, dpi).max(0) as u32;
    let last = physical_px(f64::from(REGION[0] + INK_SEARCH_SPAN_POINTS), dpi).max(0) as u32;
    (first..last)
        .filter_map(|x| {
            let reference = physical_pixel(capture, [x, top])?;
            let ink = (top..bottom)
                .filter_map(|y| physical_pixel(capture, [x, y]))
                .filter(|pixel| !matches_rgb(*pixel, reference))
                .count();
            (ink > 0).then_some((ink, x))
        })
        .max()
        .map(|(_, x)| x)
        .ok_or(ScrollChromePixelFailure::ContentInkAbsent)
}

/// Rows compared and rows that disagreed when `after[y]` is read against
/// `before[y + shift]` down one column.
fn mismatches(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
    column_px: u32,
    top: u32,
    bottom: u32,
    shift: i64,
) -> (usize, usize) {
    let mut compared = 0;
    let mut mismatched = 0;
    for y in top..bottom {
        let source_y = i64::from(y) + shift;
        if source_y < i64::from(top) || source_y >= i64::from(bottom) {
            continue;
        }
        let (Some(moved), Some(origin)) = (
            physical_pixel(after, [column_px, y]),
            physical_pixel(before, [column_px, source_y as u32]),
        ) else {
            continue;
        };
        compared += 1;
        if !matches_rgb(moved, origin) {
            mismatched += 1;
        }
    }
    (compared, mismatched)
}

fn is_thumb_tone(pixel: [u8; 4]) -> bool {
    THUMB_TONES
        .iter()
        .any(|tone| matches_rgb(pixel, [tone[0], tone[1], tone[2], 255]))
}

fn matches_rgb(observed: [u8; 4], expected: [u8; 4]) -> bool {
    observed[..3]
        .iter()
        .zip(expected[..3].iter())
        .all(|(left, right)| left.abs_diff(*right) <= CHANNEL_TOLERANCE)
}

fn physical_pixel(pixels: &NativeClientPixelCapture, point: [u32; 2]) -> Option<[u8; 4]> {
    if point[0] >= pixels.width() || point[1] >= pixels.height() {
        return None;
    }
    let offset = (point[1] as usize)
        .checked_mul(pixels.width() as usize)?
        .checked_add(point[0] as usize)?
        .checked_mul(4)?;
    let pixel = pixels.rgba().get(offset..offset + 4)?;
    Some([pixel[0], pixel[1], pixel[2], pixel[3]])
}

impl VerticalThumbEvidence {
    pub(crate) const fn top_px(self) -> u32 {
        self.top_px
    }

    pub(crate) const fn length_px(self) -> u32 {
        self.length_px
    }
}

impl ContentShiftEvidence {
    pub(crate) const fn shift_px(self) -> i64 {
        self.shift_px
    }

    pub(crate) const fn compared_rows(self) -> usize {
        self.compared_rows
    }

    pub(crate) const fn column_px(self) -> u32 {
        self.column_px
    }

    pub(crate) const fn mismatched_rows(self) -> usize {
        self.mismatched_rows
    }
}
