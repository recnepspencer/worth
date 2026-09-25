use crate::external_observation::NativeClientPixelCapture;

const DEFAULT_LOGICAL_EXTENT: [u32; 2] = [960, 600];
const RESIZED_LOGICAL_EXTENT: [u32; 2] = [1_120, 700];
const CHANNEL_TOLERANCE: u8 = 12;
const RIGHT_SAFETY: u32 = 2;
const LINE_HEIGHT: u32 = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlatformPulseWrappingTextFailure {
    UnexpectedExtent([u32; 2]),
    RequiredWrappedLineMissing {
        identity: &'static str,
        line: u32,
    },
    InkReachedClippingEdge {
        identity: &'static str,
        bottom_pixels: usize,
        right_pixels: usize,
    },
    InkReachedRowSeparation {
        identity: &'static str,
        separation_pixels: usize,
    },
}

pub(crate) fn adjudicate_default_wrapping_text(
    capture: &NativeClientPixelCapture,
) -> Result<(), PlatformPulseWrappingTextFailure> {
    adjudicate_wrapping_text(capture, DEFAULT_LOGICAL_EXTENT, contracts(704))
}

pub(crate) fn adjudicate_resized_wrapping_text(
    capture: &NativeClientPixelCapture,
) -> Result<(), PlatformPulseWrappingTextFailure> {
    adjudicate_open_wrapping_text(capture, RESIZED_LOGICAL_EXTENT, contracts(864))
}

fn adjudicate_open_wrapping_text(
    capture: &NativeClientPixelCapture,
    logical_extent: [u32; 2],
    mut expected: [WrappingTextContract; 3],
) -> Result<(), PlatformPulseWrappingTextFailure> {
    // This checkpoint has one authored black, alpha-128 modal Backdrop.
    // Expected sRGB backgrounds after linear-light compositing; text guards
    // remain unchanged and must still distinguish ink from the dimmed surface.
    expected[0].background = [9, 13, 17];
    expected[1].background = [9, 13, 17];
    expected[2].background = [14, 18, 24];
    adjudicate_wrapping_text(capture, logical_extent, expected)
}

fn adjudicate_wrapping_text(
    capture: &NativeClientPixelCapture,
    logical_extent: [u32; 2],
    contracts: [WrappingTextContract; 3],
) -> Result<(), PlatformPulseWrappingTextFailure> {
    if capture.width() * logical_extent[1] != capture.height() * logical_extent[0] {
        return Err(PlatformPulseWrappingTextFailure::UnexpectedExtent([
            capture.width(),
            capture.height(),
        ]));
    }
    for contract in contracts {
        let rect = scale_rect(capture, logical_extent, contract.rect);
        let bottom_pixels = changed_in_bottom_guard(capture, logical_extent, rect, contract);
        let right_pixels =
            changed_in_right_guard(capture, logical_extent, rect, contract.background);
        if bottom_pixels != 0 || right_pixels != 0 {
            return Err(PlatformPulseWrappingTextFailure::InkReachedClippingEdge {
                identity: contract.identity,
                bottom_pixels,
                right_pixels,
            });
        }
        let separation_pixels = changed_in_row_separation(capture, logical_extent, rect, contract);
        if separation_pixels != 0 {
            return Err(PlatformPulseWrappingTextFailure::InkReachedRowSeparation {
                identity: contract.identity,
                separation_pixels,
            });
        }
        require_inked_line_cells(capture, logical_extent, rect, contract)?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct WrappingTextContract {
    identity: &'static str,
    rect: [u32; 4],
    background: [u8; 3],
    required_lines: u32,
    line_stride: u32,
    bottom_safety: u32,
}

fn contracts(query_x: u32) -> [WrappingTextContract; 3] {
    [
        WrappingTextContract {
            identity: "platform.pulse.text.evidence_body",
            rect: [48, 176, 168, 64],
            background: [17, 22, 28],
            required_lines: 2,
            line_stride: LINE_HEIGHT,
            bottom_safety: 20,
        },
        WrappingTextContract {
            identity: "platform.pulse.text.source_query",
            rect: [48, 256, 160, 48],
            background: [17, 22, 28],
            required_lines: 2,
            line_stride: LINE_HEIGHT,
            bottom_safety: 8,
        },
        WrappingTextContract {
            identity: "platform.pulse.text.projected_status",
            rect: [query_x, 220, 184, 48],
            background: [23, 29, 37],
            required_lines: 2,
            line_stride: 24,
            bottom_safety: 4,
        },
    ]
}

fn scale_rect(
    capture: &NativeClientPixelCapture,
    logical_extent: [u32; 2],
    rect: [u32; 4],
) -> [u32; 4] {
    [
        rect[0] * capture.width() / logical_extent[0],
        rect[1] * capture.height() / logical_extent[1],
        rect[2] * capture.width() / logical_extent[0],
        rect[3] * capture.height() / logical_extent[1],
    ]
}

fn changed_in_bottom_guard(
    capture: &NativeClientPixelCapture,
    logical_extent: [u32; 2],
    rect: [u32; 4],
    contract: WrappingTextContract,
) -> usize {
    let logical_guard = contract.bottom_safety * capture.height() / logical_extent[1];
    let start = rect[1] + rect[3].saturating_sub(logical_guard.max(1));
    count_changed(
        capture,
        [rect[0], start, rect[2], rect[1] + rect[3] - start],
        contract.background,
    )
}

fn changed_in_right_guard(
    capture: &NativeClientPixelCapture,
    logical_extent: [u32; 2],
    rect: [u32; 4],
    background: [u8; 3],
) -> usize {
    let logical_guard = RIGHT_SAFETY * capture.width() / logical_extent[0];
    let start = rect[0] + rect[2].saturating_sub(logical_guard.max(1));
    count_changed(
        capture,
        [start, rect[1], rect[0] + rect[2] - start, rect[3]],
        background,
    )
}

fn changed_in_row_separation(
    capture: &NativeClientPixelCapture,
    logical_extent: [u32; 2],
    rect: [u32; 4],
    contract: WrappingTextContract,
) -> usize {
    let logical_gap = contract.line_stride.saturating_sub(LINE_HEIGHT);
    if logical_gap == 0 || contract.required_lines < 2 {
        return 0;
    }
    let line_height = LINE_HEIGHT * capture.height() / logical_extent[1];
    let gap_height = logical_gap * capture.height() / logical_extent[1];
    count_changed(
        capture,
        [rect[0], rect[1] + line_height, rect[2], gap_height],
        contract.background,
    )
}

fn require_inked_line_cells(
    capture: &NativeClientPixelCapture,
    logical_extent: [u32; 2],
    rect: [u32; 4],
    contract: WrappingTextContract,
) -> Result<(), PlatformPulseWrappingTextFailure> {
    let line_height = (LINE_HEIGHT * capture.height() / logical_extent[1]).max(1);
    let line_stride = (contract.line_stride * capture.height() / logical_extent[1]).max(1);
    for line in 0..contract.required_lines {
        let line_rect = [rect[0], rect[1] + line * line_stride, rect[2], line_height];
        if count_changed(capture, line_rect, contract.background) == 0 {
            return Err(
                PlatformPulseWrappingTextFailure::RequiredWrappedLineMissing {
                    identity: contract.identity,
                    line: line + 1,
                },
            );
        }
    }
    Ok(())
}

fn count_changed(capture: &NativeClientPixelCapture, rect: [u32; 4], background: [u8; 3]) -> usize {
    let width = capture.width() as usize;
    let mut changed = 0;
    for y in rect[1]..rect[1] + rect[3] {
        for x in rect[0]..rect[0] + rect[2] {
            let offset = ((y as usize * width) + x as usize) * 4;
            let pixel = &capture.rgba()[offset..offset + 3];
            if pixel
                .iter()
                .zip(background)
                .any(|(&observed, expected)| observed.abs_diff(expected) > CHANNEL_TOLERANCE)
            {
                changed += 1;
            }
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottom_edge_ink_is_rejected() {
        let mut rgba = valid_capture();
        let clipped = ((303 * 960 + 60) * 4) as usize;
        rgba[clipped..clipped + 4].copy_from_slice(&[161, 169, 180, 255]);
        let capture = NativeClientPixelCapture::new(1, 960, 600, rgba).unwrap();
        assert!(matches!(
            adjudicate_default_wrapping_text(&capture),
            Err(PlatformPulseWrappingTextFailure::InkReachedClippingEdge { .. })
        ));
    }

    #[test]
    fn an_empty_second_line_cell_is_rejected() {
        let mut rgba = valid_capture();
        paint(&mut rgba, [48, 256, 160, 48], [17, 22, 28, 255]);
        paint(&mut rgba, [60, 260, 20, 8], [161, 169, 180, 255]);
        let capture = NativeClientPixelCapture::new(1, 960, 600, rgba).unwrap();
        assert!(matches!(
            adjudicate_default_wrapping_text(&capture),
            Err(
                PlatformPulseWrappingTextFailure::RequiredWrappedLineMissing {
                    identity: "platform.pulse.text.source_query",
                    line: 2,
                }
            )
        ));
    }

    #[test]
    fn live_evidence_body_requires_its_second_line_and_bottom_gutter() {
        let mut rgba = valid_capture();
        paint(&mut rgba, [48, 176, 168, 64], [17, 22, 28, 255]);
        paint(&mut rgba, [60, 180, 20, 8], [242, 244, 247, 255]);
        let capture = NativeClientPixelCapture::new(1, 960, 600, rgba).unwrap();
        assert!(matches!(
            adjudicate_default_wrapping_text(&capture),
            Err(
                PlatformPulseWrappingTextFailure::RequiredWrappedLineMissing {
                    identity: "platform.pulse.text.evidence_body",
                    line: 2,
                }
            )
        ));
    }

    #[test]
    fn query_posture_requires_its_second_line_and_bottom_gutter() {
        let mut rgba = valid_capture();
        paint(&mut rgba, [704, 220, 184, 48], [23, 29, 37, 255]);
        paint(&mut rgba, [716, 224, 20, 8], [210, 153, 34, 255]);
        let capture = NativeClientPixelCapture::new(1, 960, 600, rgba).unwrap();
        assert!(matches!(
            adjudicate_default_wrapping_text(&capture),
            Err(
                PlatformPulseWrappingTextFailure::RequiredWrappedLineMissing {
                    identity: "platform.pulse.text.projected_status",
                    line: 2,
                }
            )
        ));
    }

    #[test]
    fn query_value_cannot_wrap_into_the_posture_row() {
        let mut rgba = valid_capture();
        paint(&mut rgba, [716, 240, 20, 4], [210, 153, 34, 255]);
        let capture = NativeClientPixelCapture::new(1, 960, 600, rgba).unwrap();
        assert!(matches!(
            adjudicate_default_wrapping_text(&capture),
            Err(PlatformPulseWrappingTextFailure::InkReachedRowSeparation {
                identity: "platform.pulse.text.projected_status",
                ..
            })
        ));
    }

    fn base_capture() -> Vec<u8> {
        let mut rgba = vec![0; 960 * 600 * 4];
        for pixel in rgba.as_chunks_mut::<4>().0 {
            pixel.copy_from_slice(&[11, 15, 20, 255]);
        }
        paint(&mut rgba, [48, 176, 168, 64], [17, 22, 28, 255]);
        paint(&mut rgba, [48, 256, 160, 48], [17, 22, 28, 255]);
        paint(&mut rgba, [704, 220, 184, 48], [23, 29, 37, 255]);
        rgba
    }

    fn valid_capture() -> Vec<u8> {
        let mut rgba = base_capture();
        for (x, y, color) in [
            (60, 180, [242, 244, 247, 255]),
            (60, 200, [242, 244, 247, 255]),
            (60, 260, [161, 169, 180, 255]),
            (60, 280, [161, 169, 180, 255]),
            (716, 224, [210, 153, 34, 255]),
            (716, 248, [210, 153, 34, 255]),
        ] {
            paint(&mut rgba, [x, y, 20, 8], color);
        }
        rgba
    }

    fn paint(rgba: &mut [u8], rect: [u32; 4], color: [u8; 4]) {
        for y in rect[1]..rect[1] + rect[3] {
            for x in rect[0]..rect[0] + rect[2] {
                let offset = ((y * 960 + x) * 4) as usize;
                rgba[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }
}
