use crate::external_observation::NativeClientPixelCapture;
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationEnvelope,
    PlatformPulseVisualOverlayCleared, PlatformPulseVisualOverlayPublished,
};

use super::{
    visual_contract_manifest::{
        checked_in_adjudication_contract, PlatformPulseVisualAdjudicationContract,
    },
    ExecutableVisualIdentityFailure, ExecutableVisualSnapshotEvidence,
    ExecutableVisualTraceEvidence,
};

#[derive(Debug)]
pub(crate) struct ExecutableVisualOverlayEvidence {
    sequence: u64,
    overlay: PlatformPulseVisualOverlayPublished,
    pixels: NativeClientPixelCapture,
    matching_border_pixels: usize,
    sampled_border_pixels: usize,
    physical_extent: [u32; 2],
    target_point: [u32; 2],
    background_point: [u32; 2],
}

#[derive(Debug)]
pub(crate) struct ExecutableVisualClearEvidence {
    sequence: u64,
    clear: PlatformPulseVisualOverlayCleared,
    pixels: NativeClientPixelCapture,
}

pub(crate) fn adjudicate_overlay_pixels(
    envelope: PlatformPulseLifecycleObservationEnvelope,
    snapshot: &ExecutableVisualSnapshotEvidence,
    trace: &ExecutableVisualTraceEvidence,
    process_id: u32,
    pixels: NativeClientPixelCapture,
) -> Result<ExecutableVisualOverlayEvidence, ExecutableVisualIdentityFailure> {
    let manifest = checked_in_adjudication_contract()
        .map_err(ExecutableVisualIdentityFailure::VisualContract)?;
    let sequence = trace.sequence().saturating_add(1);
    require_sequence(&envelope, sequence)?;
    let PlatformPulseLifecycleObservation::VisualOverlayPublished(overlay) = envelope.outcome()
    else {
        return Err(ExecutableVisualIdentityFailure::WrongEvent(
            "visual overlay published",
        ));
    };
    if overlay.base_snapshot() != snapshot.snapshot().affinity().snapshot()
        || overlay.base_frame() != snapshot.snapshot().affinity().frame()
        || overlay.target_region() != trace.trace().target().visible_region()
        || overlay.published_frame() == overlay.base_frame()
    {
        return Err(ExecutableVisualIdentityFailure::OverlayAffinity);
    }
    require_native_affinity(process_id, snapshot.physical_extent(), &pixels)?;
    let samples = border_samples(&pixels, overlay.target_region())?;
    let matching = samples
        .iter()
        .filter(|&&pixel| matches_rgb(pixel, manifest.overlay_rgba(), &manifest))
        .count();
    if matching * 4 < samples.len() * 3 {
        return Err(ExecutableVisualIdentityFailure::BorderNotVisible {
            matching,
            sampled: samples.len(),
        });
    }
    let target_point = snapshot.project_logical_point(manifest.target_logical_point())?;
    let background_point = snapshot.project_logical_point(manifest.background_logical_point())?;
    require_control_pixels(&pixels, target_point, background_point, &manifest)?;
    Ok(ExecutableVisualOverlayEvidence {
        sequence,
        overlay: *overlay,
        pixels,
        matching_border_pixels: matching,
        sampled_border_pixels: samples.len(),
        physical_extent: snapshot.physical_extent(),
        target_point,
        background_point,
    })
}

pub(crate) fn adjudicate_restored_pixels(
    envelope: PlatformPulseLifecycleObservationEnvelope,
    overlay: &ExecutableVisualOverlayEvidence,
    process_id: u32,
    pixels: NativeClientPixelCapture,
) -> Result<ExecutableVisualClearEvidence, ExecutableVisualIdentityFailure> {
    let manifest = checked_in_adjudication_contract()
        .map_err(ExecutableVisualIdentityFailure::VisualContract)?;
    let sequence = overlay.sequence.saturating_add(1);
    require_sequence(&envelope, sequence)?;
    let PlatformPulseLifecycleObservation::VisualOverlayCleared(clear) = envelope.outcome() else {
        return Err(ExecutableVisualIdentityFailure::WrongEvent(
            "visual overlay cleared",
        ));
    };
    if clear.overlay() != overlay.overlay.overlay()
        || clear.published_frame() != overlay.overlay.published_frame()
        || clear.cleared_frame() == clear.published_frame()
    {
        return Err(ExecutableVisualIdentityFailure::ClearAffinity);
    }
    require_native_affinity(process_id, overlay.physical_extent, &pixels)?;
    let samples = border_samples(&pixels, overlay.overlay.target_region())?;
    let matching = samples
        .iter()
        .filter(|&&pixel| matches_rgb(pixel, manifest.overlay_rgba(), &manifest))
        .count();
    if matching != 0 {
        return Err(ExecutableVisualIdentityFailure::BorderStillVisible {
            matching,
            sampled: samples.len(),
        });
    }
    require_control_pixels(
        &pixels,
        overlay.target_point,
        overlay.background_point,
        &manifest,
    )?;
    Ok(ExecutableVisualClearEvidence {
        sequence,
        clear: *clear,
        pixels,
    })
}

fn require_sequence(
    envelope: &PlatformPulseLifecycleObservationEnvelope,
    expected: u64,
) -> Result<(), ExecutableVisualIdentityFailure> {
    let observed = envelope.sequence().value();
    if observed == expected {
        Ok(())
    } else {
        Err(ExecutableVisualIdentityFailure::WrongSequence { expected, observed })
    }
}

fn require_native_affinity(
    process_id: u32,
    expected_extent: [u32; 2],
    pixels: &NativeClientPixelCapture,
) -> Result<(), ExecutableVisualIdentityFailure> {
    if pixels.process_id() != process_id {
        return Err(ExecutableVisualIdentityFailure::NativeProcessIdentity);
    }
    if [pixels.width(), pixels.height()] != expected_extent {
        return Err(ExecutableVisualIdentityFailure::NativeCaptureExtent);
    }
    Ok(())
}

fn border_samples(
    pixels: &NativeClientPixelCapture,
    region: [u32; 4],
) -> Result<Vec<[u8; 4]>, ExecutableVisualIdentityFailure> {
    let [left, top, right, bottom] = region;
    if right > pixels.width() || bottom > pixels.height() || right - left < 4 || bottom - top < 4 {
        return Err(ExecutableVisualIdentityFailure::NativeCaptureExtent);
    }
    let mut samples = Vec::new();
    for x in left..right {
        samples.push(pixel_at(pixels, [x, top]).unwrap());
        samples.push(pixel_at(pixels, [x, top + 1]).unwrap());
        samples.push(pixel_at(pixels, [x, bottom - 2]).unwrap());
        samples.push(pixel_at(pixels, [x, bottom - 1]).unwrap());
    }
    for y in top + 2..bottom - 2 {
        samples.push(pixel_at(pixels, [left, y]).unwrap());
        samples.push(pixel_at(pixels, [left + 1, y]).unwrap());
        samples.push(pixel_at(pixels, [right - 2, y]).unwrap());
        samples.push(pixel_at(pixels, [right - 1, y]).unwrap());
    }
    Ok(samples)
}

fn require_control_pixels(
    pixels: &NativeClientPixelCapture,
    target_point: [u32; 2],
    background_point: [u32; 2],
    manifest: &PlatformPulseVisualAdjudicationContract,
) -> Result<(), ExecutableVisualIdentityFailure> {
    if !pixel_at(pixels, target_point)
        .is_some_and(|pixel| matches_rgb(pixel, manifest.target_rgba(), manifest))
    {
        return Err(ExecutableVisualIdentityFailure::TargetPixelChanged);
    }
    if !pixel_at(pixels, background_point)
        .is_some_and(|pixel| matches_rgb(pixel, manifest.canvas_rgba(), manifest))
    {
        return Err(ExecutableVisualIdentityFailure::BackgroundPixelChanged);
    }
    Ok(())
}

fn pixel_at(pixels: &NativeClientPixelCapture, point: [u32; 2]) -> Option<[u8; 4]> {
    let offset = usize::try_from(point[1])
        .ok()?
        .checked_mul(usize::try_from(pixels.width()).ok()?)?
        .checked_add(usize::try_from(point[0]).ok()?)?
        .checked_mul(4)?;
    let pixel = pixels.rgba().get(offset..offset + 4)?;
    Some([pixel[0], pixel[1], pixel[2], pixel[3]])
}

fn matches_rgb(
    observed: [u8; 4],
    expected: [u8; 4],
    manifest: &PlatformPulseVisualAdjudicationContract,
) -> bool {
    observed[..3]
        .iter()
        .zip(expected)
        .all(|(&observed, expected)| observed.abs_diff(expected) <= manifest.channel_tolerance())
}

impl ExecutableVisualOverlayEvidence {
    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(crate) fn capture_count(&self) -> u32 {
        self.pixels.capture_count()
    }

    pub(crate) fn border_ratio(&self) -> (usize, usize) {
        (self.matching_border_pixels, self.sampled_border_pixels)
    }
}

impl ExecutableVisualClearEvidence {
    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(crate) fn clear(&self) -> PlatformPulseVisualOverlayCleared {
        self.clear
    }

    pub(crate) fn capture_count(&self) -> u32 {
        self.pixels.capture_count()
    }
}
