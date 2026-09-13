use crate::external_observation::NativeClientPixelCapture;
use crate::native_platform::NativePlatformFailure;

use super::super::{
    capture, export_capture, NativeBoundExecutableWorld, PlatformPulsePortalJourneyFailure,
    PIXEL_POLL_SLICE, TRANSITION_DEADLINE,
};

pub(super) fn await_pixels(
    world: &mut NativeBoundExecutableWorld,
    nested: bool,
    parent: &NativeClientPixelCapture,
    logical_extent: [u32; 2],
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    let deadline = std::time::Instant::now() + TRANSITION_DEADLINE;
    loop {
        let current = match capture(world) {
            Ok(current) => current,
            Err(PlatformPulsePortalJourneyFailure::Native(
                NativePlatformFailure::ClientCapture(_),
            )) if std::time::Instant::now() < deadline => {
                std::thread::sleep(PIXEL_POLL_SLICE);
                continue;
            }
            Err(failure) => return Err(failure),
        };
        let result = check_pixels(&current, nested, logical_extent).and_then(|()| {
            if current.width() != parent.width() || current.height() != parent.height() {
                return Err(PlatformPulsePortalJourneyFailure::ModalStack(
                    "stacked modal capture changed client dimensions",
                ));
            }
            require_parent_review_label(&current, parent, nested, logical_extent)?;
            // This patch is below the first dialog and inside the second.
            // Unlike their shared fill color it distinguishes a stale child.
            let same = restoration_patch_matches(&current, parent, logical_extent);
            if same == nested {
                Err(PlatformPulsePortalJourneyFailure::ModalStack(
                    "child-covered application pixels did not change and restore",
                ))
            } else {
                Ok(())
            }
        });
        if result.is_ok() && nested {
            let [width, height] = logical_extent;
            export_capture(&format!("04-stacked-modal-{width}x{height}.png"), &current)
                .map_err(PlatformPulsePortalJourneyFailure::CaptureExport)?;
        }
        if result.is_ok() || std::time::Instant::now() >= deadline {
            return result;
        }
        std::thread::sleep(PIXEL_POLL_SLICE);
    }
}

fn restoration_patch_matches(
    current: &NativeClientPixelCapture,
    parent: &NativeClientPixelCapture,
    [width, height]: [u32; 2],
) -> bool {
    if current.width() != parent.width() || current.height() != parent.height() {
        return false;
    }
    for y in 480 * current.height() / height..488 * current.height() / height {
        for x in 560 * current.width() / width..568 * current.width() / width {
            let offset = ((y * current.width() + x) * 4) as usize;
            if current.rgba()[offset..offset + 3]
                .iter()
                .zip(&parent.rgba()[offset..offset + 3])
                .any(|(&left, &right)| left.abs_diff(right) > 2)
            {
                return false;
            }
        }
    }
    true
}

fn require_parent_review_label(
    current: &NativeClientPixelCapture,
    parent: &NativeClientPixelCapture,
    nested: bool,
    [width, height]: [u32; 2],
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    // The Review label is in the exposed parent, above the child's y296 edge.
    // Independently select opaque glyph interiors in the pre-open native image.
    // One black alpha128 scrim maps #A1A9B4 to #757B83 in linear light.
    let expected = if nested {
        [117, 123, 131]
    } else {
        [161, 169, 180]
    };
    let mut glyph_pixels = 0;
    for y in 258 * current.height() / height..278 * current.height() / height {
        for x in 576 * current.width() / width..760 * current.width() / width {
            let offset = ((y * current.width() + x) * 4) as usize;
            if parent.rgba()[offset..offset + 3]
                .iter()
                .zip([161, 169, 180])
                .any(|(&value, channel)| value.abs_diff(channel) > 2)
            {
                continue;
            }
            glyph_pixels += 1;
            if current.rgba()[offset..offset + 3]
                .iter()
                .zip(expected)
                .any(|(&value, channel)| value.abs_diff(channel) > 2)
            {
                return Err(PlatformPulsePortalJourneyFailure::ModalStack(
                    "nested Portal covered or incorrectly dimmed its parent's Review label",
                ));
            }
        }
    }
    if glyph_pixels < 32 {
        return Err(PlatformPulsePortalJourneyFailure::ModalStack(
            "parent capture did not contain the required Review label",
        ));
    }
    Ok(())
}

fn check_pixels(
    capture: &NativeClientPixelCapture,
    nested: bool,
    logical_extent: [u32; 2],
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    // Independent product oracle: black alpha128 in linear light dims
    // elevated #171D25 to #0E1218, then #080A0E. The exposed first dialog
    // receives only the second scrim; the top dialog receives neither.
    let application = if nested { [8, 10, 14] } else { [14, 18, 24] };
    let parent = if nested { [14, 18, 24] } else { [23, 29, 37] };
    // The child is clamped to a 24-point viewport margin. These independently
    // chosen samples begin four points below the largest legal dialog edge at
    // each exercised extent, so neither may contain the undimmed surface fill.
    let bottom_gutter_sample = match logical_extent {
        [960, 600] => [560, 580, 8, 8],
        [1120, 700] => [640, 680, 8, 8],
        _ => {
            return Err(PlatformPulsePortalJourneyFailure::ModalStack(
                "stacked modal oracle received an unsupported client extent",
            ));
        }
    };
    if nested && patch_matches(capture, bottom_gutter_sample, [23, 29, 37], logical_extent) {
        return Err(PlatformPulsePortalJourneyFailure::ModalStack(
            "nested dialog extends into the required bottom gutter",
        ));
    }
    if nested {
        require_nested_content(capture, logical_extent)?;
    }
    for (rect, expected, failure) in [
        (
            [280, 360, 8, 8],
            application,
            "application did not receive cumulative Backdrops",
        ),
        (
            [536, 140, 8, 8],
            parent,
            "exposed first dialog has incorrect Backdrop depth",
        ),
        (
            [560, 252, 8, 8],
            parent,
            "nested Portal promoted its ordinary Review anchor above the Backdrop",
        ),
        // The nested dialog begins at [552,296], below the Review action.
        (
            [560, 304, 8, 8],
            [23, 29, 37],
            "top dialog is missing or self-dimmed",
        ),
    ] {
        if !patch_matches(capture, rect, expected, logical_extent) {
            return Err(PlatformPulsePortalJourneyFailure::ModalStack(failure));
        }
    }
    Ok(())
}

fn require_nested_content(
    capture: &NativeClientPixelCapture,
    logical_extent: [u32; 2],
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    // Independent global rectangles derived from the nested dialog origin
    // [552,296] plus the authored child layout. Exact foreground colors must
    // contribute real glyph pixels; broad target interiors must retain fills.
    for (rect, color, minimum, failure) in [
        (
            [576, 320, 232, 28],
            [242, 244, 247],
            12,
            "nested title text is missing",
        ),
        (
            [576, 372, 232, 80],
            [161, 169, 180],
            12,
            "nested body text is missing",
        ),
        (
            [600, 522, 64, 20],
            [161, 169, 180],
            12,
            "nested Cancel text is missing",
        ),
        (
            [712, 522, 88, 20],
            [250, 251, 252],
            12,
            "nested primary text is missing",
        ),
        (
            [576, 512, 104, 40],
            [17, 22, 28],
            2_400,
            "nested Cancel surface is missing",
        ),
        (
            [696, 512, 112, 40],
            [148, 64, 212],
            2_600,
            "nested primary surface is missing",
        ),
    ] {
        if matching_pixels(capture, rect, color, logical_extent) < minimum {
            return Err(PlatformPulsePortalJourneyFailure::ModalStack(failure));
        }
    }
    Ok(())
}

fn matching_pixels(
    capture: &NativeClientPixelCapture,
    rect: [u32; 4],
    expected: [u8; 3],
    [logical_width, logical_height]: [u32; 2],
) -> usize {
    let [x, y, width, height] = rect;
    let mut matching = 0;
    for py in
        y * capture.height() / logical_height..(y + height) * capture.height() / logical_height
    {
        for px in x * capture.width() / logical_width..(x + width) * capture.width() / logical_width
        {
            let offset = ((py * capture.width() + px) * 4) as usize;
            matching += usize::from(
                capture.rgba()[offset..offset + 3]
                    .iter()
                    .zip(expected)
                    .all(|(&observed, expected)| observed.abs_diff(expected) <= 4),
            );
        }
    }
    matching
}

fn patch_matches(
    capture: &NativeClientPixelCapture,
    rect: [u32; 4],
    expected: [u8; 3],
    [logical_width, logical_height]: [u32; 2],
) -> bool {
    let [x, y, width, height] = rect;
    for py in
        y * capture.height() / logical_height..(y + height) * capture.height() / logical_height
    {
        for px in x * capture.width() / logical_width..(x + width) * capture.width() / logical_width
        {
            let offset = ((py * capture.width() + px) * 4) as usize;
            if capture.rgba()[offset..offset + 3]
                .iter()
                .zip(expected)
                .any(|(&observed, expected)| observed.abs_diff(expected) > 2)
            {
                return false;
            }
        }
    }
    true
}
