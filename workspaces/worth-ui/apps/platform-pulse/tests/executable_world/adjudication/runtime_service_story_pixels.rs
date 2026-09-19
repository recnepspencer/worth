use crate::external_observation::NativeClientPixelCapture;

const LOGICAL_EXTENT: [u32; 2] = [960, 600];
const QUERY_POSTURE_STORY: [u32; 4] = [680, 176, 232, 104];
const COMMAND_STORY: [u32; 4] = [680, 392, 232, 48];
const QUERY_DENIAL_STORY: [u32; 4] = [680, 448, 232, 76];
const MUTABLE_STORY_REGIONS: [[u32; 4]; 3] =
    [QUERY_POSTURE_STORY, COMMAND_STORY, QUERY_DENIAL_STORY];
const MINIMUM_CHANGED_PIXELS: usize = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulseRuntimeServicePixelEvidence {
    query_posture_changed_pixels: usize,
    command_story_changed_pixels: usize,
    query_denial_story_changed_pixels: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlatformPulseRuntimeServicePixelFailure {
    CaptureMismatch,
    StoryDidNotChange {
        identity: &'static str,
        changed_pixels: usize,
    },
}

pub(crate) fn adjudicate_runtime_service_story_pixels(
    baseline: &NativeClientPixelCapture,
    observed: &NativeClientPixelCapture,
) -> Result<PlatformPulseRuntimeServicePixelEvidence, PlatformPulseRuntimeServicePixelFailure> {
    if baseline.process_id() != observed.process_id()
        || baseline.width() != observed.width()
        || baseline.height() != observed.height()
        || baseline.width() * LOGICAL_EXTENT[1] != baseline.height() * LOGICAL_EXTENT[0]
    {
        return Err(PlatformPulseRuntimeServicePixelFailure::CaptureMismatch);
    }
    let query_posture_changed_pixels = require_changed(
        baseline,
        observed,
        QUERY_POSTURE_STORY,
        "platform.pulse.component.projected_status",
    )?;
    let command_story_changed_pixels = require_changed(
        baseline,
        observed,
        COMMAND_STORY,
        "platform.pulse.text.native_body",
    )?;
    let query_denial_story_changed_pixels = require_changed(
        baseline,
        observed,
        QUERY_DENIAL_STORY,
        "platform.pulse.text.query_denial",
    )?;
    Ok(PlatformPulseRuntimeServicePixelEvidence {
        query_posture_changed_pixels,
        command_story_changed_pixels,
        query_denial_story_changed_pixels,
    })
}

fn require_changed(
    baseline: &NativeClientPixelCapture,
    observed: &NativeClientPixelCapture,
    logical: [u32; 4],
    identity: &'static str,
) -> Result<usize, PlatformPulseRuntimeServicePixelFailure> {
    let rect = scale_rect(observed, logical);
    let mut changed_pixels = 0;
    for y in rect[1]..rect[1] + rect[3] {
        for x in rect[0]..rect[0] + rect[2] {
            changed_pixels += usize::from(rgba_at(baseline, x, y) != rgba_at(observed, x, y));
        }
    }
    if changed_pixels < MINIMUM_CHANGED_PIXELS {
        Err(PlatformPulseRuntimeServicePixelFailure::StoryDidNotChange {
            identity,
            changed_pixels,
        })
    } else {
        Ok(changed_pixels)
    }
}

fn scale_rect(capture: &NativeClientPixelCapture, rect: [u32; 4]) -> [u32; 4] {
    [
        rect[0] * capture.width() / LOGICAL_EXTENT[0],
        rect[1] * capture.height() / LOGICAL_EXTENT[1],
        rect[2] * capture.width() / LOGICAL_EXTENT[0],
        rect[3] * capture.height() / LOGICAL_EXTENT[1],
    ]
}

pub(super) fn is_intentionally_mutable_story_pixel(
    capture: &NativeClientPixelCapture,
    x: u32,
    y: u32,
) -> bool {
    MUTABLE_STORY_REGIONS.into_iter().any(|logical| {
        let rect = scale_rect(capture, logical);
        x >= rect[0] && x < rect[0] + rect[2] && y >= rect[1] && y < rect[1] + rect[3]
    })
}

fn rgba_at(capture: &NativeClientPixelCapture, x: u32, y: u32) -> &[u8] {
    let offset = ((y as usize * capture.width() as usize) + x as usize) * 4;
    &capture.rgba()[offset..offset + 4]
}

impl PlatformPulseRuntimeServicePixelEvidence {
    pub(crate) const fn query_posture_changed_pixels(self) -> usize {
        self.query_posture_changed_pixels
    }

    pub(crate) const fn command_story_changed_pixels(self) -> usize {
        self.command_story_changed_pixels
    }

    pub(crate) const fn query_denial_story_changed_pixels(self) -> usize {
        self.query_denial_story_changed_pixels
    }
}

impl std::fmt::Display for PlatformPulseRuntimeServicePixelFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_independent_story_regions_must_change() {
        let baseline = capture(vec![0; 960 * 600 * 4]);
        let mut observed = baseline.rgba().to_vec();
        paint(&mut observed, QUERY_POSTURE_STORY, [210, 150, 40, 255]);
        paint(&mut observed, COMMAND_STORY, [160, 170, 180, 255]);
        let failure = adjudicate_runtime_service_story_pixels(&baseline, &capture(observed));
        assert!(matches!(
            failure,
            Err(PlatformPulseRuntimeServicePixelFailure::StoryDidNotChange {
                identity: "platform.pulse.text.query_denial",
                ..
            })
        ));
    }

    #[test]
    fn three_visible_story_changes_are_admitted() {
        let baseline = capture(vec![0; 960 * 600 * 4]);
        let mut observed = baseline.rgba().to_vec();
        paint(&mut observed, QUERY_POSTURE_STORY, [210, 150, 40, 255]);
        paint(&mut observed, COMMAND_STORY, [160, 170, 180, 255]);
        paint(&mut observed, QUERY_DENIAL_STORY, [190, 140, 40, 255]);
        let evidence = adjudicate_runtime_service_story_pixels(&baseline, &capture(observed))
            .expect("all three product stories changed visibly");
        assert!(evidence.query_posture_changed_pixels() >= MINIMUM_CHANGED_PIXELS);
        assert!(evidence.command_story_changed_pixels() >= MINIMUM_CHANGED_PIXELS);
        assert!(evidence.query_denial_story_changed_pixels() >= MINIMUM_CHANGED_PIXELS);
    }

    #[test]
    fn story_mask_owns_only_the_intentionally_mutable_regions() {
        let capture = capture(vec![0; 960 * 600 * 4]);
        assert!(is_intentionally_mutable_story_pixel(&capture, 680, 176));
        assert!(is_intentionally_mutable_story_pixel(&capture, 680, 392));
        assert!(is_intentionally_mutable_story_pixel(&capture, 911, 523));
        assert!(!is_intentionally_mutable_story_pixel(&capture, 679, 176));
        assert!(!is_intentionally_mutable_story_pixel(&capture, 912, 523));
    }

    fn capture(rgba: Vec<u8>) -> NativeClientPixelCapture {
        NativeClientPixelCapture::new(7, 960, 600, rgba).expect("fixture has exact RGBA extent")
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
