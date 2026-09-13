use super::ReconstructionFrameEvidence;
use worth_ui_native_platform::{
    UiNativeClientShutdownObservation, UiNativeClientTextPresentationWorkObservation,
    UiNativePlatformCloseReceipt, UiNativePresentationWorkKind,
};

pub(super) fn reconstruction_frames(
    receipt: &UiNativePlatformCloseReceipt,
) -> ReconstructionFrameEvidence {
    let reconstruction_index = receipt
        .retained_frames()
        .iter()
        .position(|frame| frame.kind() == UiNativePresentationWorkKind::Reconstruction);
    let reconstructed = reconstruction_index.and_then(|index| receipt.retained_frames().get(index));
    let predecessor = reconstruction_index
        .and_then(|index| index.checked_sub(1))
        .and_then(|index| receipt.retained_frames().get(index));
    let next_delta =
        reconstruction_index.and_then(|index| receipt.retained_frames().get(index + 1));
    let pixels_exact = predecessor.zip(reconstructed).is_some_and(|(left, right)| {
        left.retained_baseline_rgba8() == right.retained_baseline_rgba8()
            && left.retained_center_rgba8() == right.retained_center_rgba8()
    });
    let native_transcript_exact = predecessor
        .and_then(|frame| frame.presentation())
        .zip(reconstructed.and_then(|frame| frame.presentation()))
        .is_some_and(|(left, right)| native_presentations_match(left, right));
    ReconstructionFrameEvidence {
        reconstructed_frames: receipt
            .retained_frames()
            .iter()
            .filter(|frame| frame.kind() == UiNativePresentationWorkKind::Reconstruction)
            .count(),
        pixels_exact,
        native_transcript_exact,
        predecessor_request: predecessor.and_then(frame_request),
        reconstructed_request: reconstructed.and_then(frame_request),
        next_delta_request: next_delta.and_then(frame_request),
        predecessor_pixels: predecessor.map(frame_pixels),
        reconstructed_pixels: reconstructed.map(frame_pixels),
        retained_frames: receipt
            .retained_frames()
            .iter()
            .map(|frame| {
                serde_json::json!({
                    "frame": frame.frame(),
                    "kind": format!("{:?}", frame.kind()),
                    "baseline": frame.retained_baseline_rgba8(),
                    "center": frame.retained_center_rgba8(),
                })
            })
            .collect(),
    }
}

pub(super) fn headless_reconstruction_exact(
    shutdown: &UiNativeClientShutdownObservation,
    frames: &ReconstructionFrameEvidence,
) -> bool {
    let Some((predecessor, reconstructed)) = exact_text_work_pair(shutdown, frames) else {
        return false;
    };
    reconstructed.demand_records() == predecessor.demand_records()
        && reconstructed.binding_pins() == predecessor.binding_pins()
        && reconstructed.layout_set_digest() == predecessor.layout_set_digest()
        && reconstructed.raster_key_set_digest() == predecessor.raster_key_set_digest()
        && reconstructed.glyph_run_transcript_digest() == predecessor.glyph_run_transcript_digest()
        && reconstructed.intrinsic_glyph_transcript_digest()
            == predecessor.intrinsic_glyph_transcript_digest()
        && reconstructed.intrinsic_glyph_runs() == predecessor.intrinsic_glyph_runs()
}

pub(super) fn atlas_reconstruction_exact(
    receipt: &UiNativePlatformCloseReceipt,
    shutdown: &UiNativeClientShutdownObservation,
    frames: &ReconstructionFrameEvidence,
) -> bool {
    let Some((predecessor_index, reconstructed_index)) = exact_text_work_indices(shutdown, frames)
    else {
        return false;
    };
    receipt.text_pin_frame_counts().get(predecessor_index)
        == receipt.text_pin_frame_counts().get(reconstructed_index)
        && receipt
            .text_pin_frame_observations()
            .get(predecessor_index)
            .zip(
                receipt
                    .text_pin_frame_observations()
                    .get(reconstructed_index),
            )
            .is_some_and(|(left, right)| {
                // Reconstruction reissues pin generations. Its semantic model is
                // the exact nonempty layout/raster membership, not old live handles.
                !left.is_empty()
                    && left.len() == right.len()
                    && left.iter().zip(right.iter()).all(|(a, b)| {
                        a.layout_digest() == b.layout_digest()
                            && a.raster_key_digest() == b.raster_key_digest()
                    })
            })
        && receipt
            .text_atlas_model_frame_digests()
            .get(predecessor_index)
            == receipt
                .text_atlas_model_frame_digests()
                .get(reconstructed_index)
}

pub(super) fn next_delta_is_local(
    shutdown: &UiNativeClientShutdownObservation,
    frames: &ReconstructionFrameEvidence,
) -> bool {
    frames
        .next_delta_request
        .is_some_and(|request| next_delta_retry_pair_is_local(shutdown, request))
}

fn next_delta_retry_pair_is_local(
    shutdown: &UiNativeClientShutdownObservation,
    [current_attempt, binding, mounted_frame]: [u64; 3],
) -> bool {
    let work = shutdown
        .text_presentation_work()
        .iter()
        .filter(|work| work.binding() == binding && work.mounted_frame() == mounted_frame)
        .collect::<Vec<_>>();
    // "RECONSTRUCT-CURRENT!" has 20 non-space ASCII glyphs, followed by
    // one intrinsic emoji. The separating space creates no raster demand.
    const EXPECTED_NEXT_RASTER_DEMAND: u64 = 21;
    work.len() == 2
        && work
            .iter()
            .any(|work| work.attempt() == current_attempt && work.rasterized_glyphs() == 0)
        && work
            .iter()
            .map(|work| work.rasterized_glyphs())
            .sum::<u64>()
            == 2
        && work.iter().map(|work| work.pin_additions()).sum::<u64>() == 21
        && work.iter().all(|work| {
            work.demand_records() == EXPECTED_NEXT_RASTER_DEMAND
                && work.binding_pins() == EXPECTED_NEXT_RASTER_DEMAND
        })
        && work.windows(2).all(|pair| {
            pair[0].layout_set_digest() == pair[1].layout_set_digest()
                && pair[0].raster_key_set_digest() == pair[1].raster_key_set_digest()
        })
}

fn native_presentations_match(
    left: &worth_ui_native_platform::UiNativePresentationObservation,
    right: &worth_ui_native_platform::UiNativePresentationObservation,
) -> bool {
    left.client_physical_size() == right.client_physical_size()
        && left.scale_factor_milli() == right.scale_factor_milli()
        && left.source_rgba8() == right.source_rgba8()
        && left.semantic_surface() == right.semantic_surface()
        && left.host_surface() == right.host_surface()
        && left.mounted_instance() == right.mounted_instance()
        && left.logical_bounds_milli() == right.logical_bounds_milli()
        && left.order_ordinal() == right.order_ordinal()
        && left.alpha_glyphs() == right.alpha_glyphs()
        && left.intrinsic_glyphs() == right.intrinsic_glyphs()
        && left.glyph_transcript_digest() == right.glyph_transcript_digest()
}

fn frame_request(
    frame: &worth_ui_native_platform::UiNativeRetainedFrameObservation,
) -> Option<[u64; 3]> {
    frame.presentation().map(|value| {
        [
            value.presentation_attempt(),
            value.binding_generation(),
            value.presented_frame(),
        ]
    })
}

fn frame_pixels(
    frame: &worth_ui_native_platform::UiNativeRetainedFrameObservation,
) -> [[u8; 4]; 2] {
    [
        frame.retained_baseline_rgba8(),
        frame.retained_center_rgba8(),
    ]
}

fn exact_text_work_pair<'a>(
    shutdown: &'a UiNativeClientShutdownObservation,
    frames: &ReconstructionFrameEvidence,
) -> Option<(
    &'a UiNativeClientTextPresentationWorkObservation,
    &'a UiNativeClientTextPresentationWorkObservation,
)> {
    Some((
        find_text_work(shutdown, frames.predecessor_request?)?,
        find_text_work(shutdown, frames.reconstructed_request?)?,
    ))
}

fn exact_text_work_indices(
    shutdown: &UiNativeClientShutdownObservation,
    frames: &ReconstructionFrameEvidence,
) -> Option<(usize, usize)> {
    let find = |request| unique_text_work_index(shutdown, request);
    Some((
        find(frames.predecessor_request?)?,
        find(frames.reconstructed_request?)?,
    ))
}

fn find_text_work(
    shutdown: &UiNativeClientShutdownObservation,
    request: [u64; 3],
) -> Option<&UiNativeClientTextPresentationWorkObservation> {
    let mut matches = shutdown
        .text_presentation_work()
        .iter()
        .filter(|work| text_work_matches(work, request));
    let matched = matches.next()?;
    matches.next().is_none().then_some(matched)
}

fn unique_text_work_index(
    shutdown: &UiNativeClientShutdownObservation,
    request: [u64; 3],
) -> Option<usize> {
    let mut matches = shutdown
        .text_presentation_work()
        .iter()
        .enumerate()
        .filter(|(_, work)| text_work_matches(work, request));
    let (index, _) = matches.next()?;
    matches.next().is_none().then_some(index)
}

fn text_work_matches(
    work: &UiNativeClientTextPresentationWorkObservation,
    [attempt, binding, mounted_frame]: [u64; 3],
) -> bool {
    work.attempt() == attempt && work.binding() == binding && work.mounted_frame() == mounted_frame
}
