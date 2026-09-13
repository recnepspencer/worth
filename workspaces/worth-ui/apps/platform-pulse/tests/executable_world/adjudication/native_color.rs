use std::fmt;

use crate::external_observation::{NativeClientPixelCapture, NativeClientPixelPoint};

use super::visual_contract_manifest::{
    checked_in_adjudication_contract, PlatformPulseVisualAdjudicationContract,
    PlatformPulseVisualContractFailure,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExpectedNativeColor {
    Blue,
    Green,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeColorVerdict {
    expected: ExpectedNativeColor,
    matching_samples: usize,
    sampled_pixels: usize,
}

#[derive(Debug)]
pub(crate) enum NativeColorFailure {
    VisualContract(PlatformPulseVisualContractFailure),
    InsufficientPixelSamples,
    ExpectedColorNotVisible {
        expected: ExpectedNativeColor,
        matching: usize,
        samples: Box<[NativePixelSampleObservation]>,
    },
    ExpectedTargetNotVisible {
        point: [u32; 2],
        rgba: Option<[u8; 4]>,
        capture_extent: [u32; 2],
        matching_target_pixels: usize,
        target_pixel_bounds: Option<([u32; 2], [u32; 2])>,
    },
    BackgroundPointMissing(ExpectedNativeColor),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativePixelSampleObservation {
    x: usize,
    y: usize,
    rgba: [u8; 4],
    matches_expected_color: bool,
}

pub(crate) fn adjudicate_native_color(
    pixels: &NativeClientPixelCapture,
    expected: ExpectedNativeColor,
) -> Result<NativeColorVerdict, NativeColorFailure> {
    let manifest =
        checked_in_adjudication_contract().map_err(NativeColorFailure::VisualContract)?;
    let samples = signal_samples(pixels, expected, &manifest);
    let sampled_pixels = samples.len();
    let matching_samples = samples
        .iter()
        .filter(|sample| sample.matches_expected_color)
        .count();
    if sampled_pixels < 9 {
        return Err(NativeColorFailure::InsufficientPixelSamples);
    }
    if matching_samples * 4 < sampled_pixels * 3 {
        return Err(NativeColorFailure::ExpectedColorNotVisible {
            expected,
            matching: matching_samples,
            samples: samples.into_boxed_slice(),
        });
    }
    let target_point = scaled_point(
        pixels,
        manifest.target_logical_point(),
        manifest.logical_client_extent(),
    );
    let target_rgba = pixel_at(pixels, target_point);
    let target_point_visible = target_rgba.is_some_and(|rgba| {
        rgba[..3]
            .iter()
            .zip(manifest.target_rgba())
            .all(|(&observed, expected)| {
                observed.abs_diff(expected) <= manifest.channel_tolerance()
            })
    });
    if !target_point_visible {
        let target_summary = target_pixel_summary(pixels, &manifest);
        return Err(NativeColorFailure::ExpectedTargetNotVisible {
            point: target_point,
            rgba: target_rgba,
            capture_extent: [pixels.width(), pixels.height()],
            matching_target_pixels: target_summary.matching_pixels,
            target_pixel_bounds: target_summary.bounds,
        });
    }
    Ok(NativeColorVerdict {
        expected,
        matching_samples,
        sampled_pixels,
    })
}

pub(crate) fn adjudicate_native_background_point(
    pixels: &NativeClientPixelCapture,
    expected: ExpectedNativeColor,
) -> Result<NativeClientPixelPoint, NativeColorFailure> {
    let manifest =
        checked_in_adjudication_contract().map_err(NativeColorFailure::VisualContract)?;
    let expected_rgb = expected.rgb(&manifest);
    let mut interior = Vec::new();
    for y in 1..pixels.height().saturating_sub(1) {
        for x in 1..pixels.width().saturating_sub(1) {
            let matches = ((y - 1)..=(y + 1)).all(|sample_y| {
                ((x - 1)..=(x + 1)).all(|sample_x| {
                    pixel_at(pixels, [sample_x, sample_y]).is_some_and(|rgba| {
                        rgba[..3]
                            .iter()
                            .zip(expected_rgb)
                            .all(|(&observed, expected)| {
                                observed.abs_diff(expected) <= manifest.channel_tolerance()
                            })
                    })
                })
            });
            if matches {
                interior.push((x, y));
            }
        }
    }
    let (x, y) = interior
        .get(interior.len() / 2)
        .copied()
        .ok_or(NativeColorFailure::BackgroundPointMissing(expected))?;
    NativeClientPixelPoint::interior(pixels, x, y, 1)
        .ok_or(NativeColorFailure::BackgroundPointMissing(expected))
}

struct TargetPixelSummary {
    matching_pixels: usize,
    bounds: Option<([u32; 2], [u32; 2])>,
}

fn target_pixel_summary(
    pixels: &NativeClientPixelCapture,
    manifest: &PlatformPulseVisualAdjudicationContract,
) -> TargetPixelSummary {
    let mut matching_pixels = 0;
    let mut minimum = [u32::MAX, u32::MAX];
    let mut maximum = [0, 0];
    for (index, rgba) in pixels.rgba().chunks_exact(4).enumerate() {
        let matches = rgba[..3]
            .iter()
            .zip(manifest.target_rgba())
            .all(|(&observed, expected)| {
                observed.abs_diff(expected) <= manifest.channel_tolerance()
            });
        if !matches {
            continue;
        }
        let index = u32::try_from(index).expect("capture index fits its u32 extent");
        let point = [index % pixels.width(), index / pixels.width()];
        matching_pixels += 1;
        minimum = [minimum[0].min(point[0]), minimum[1].min(point[1])];
        maximum = [maximum[0].max(point[0]), maximum[1].max(point[1])];
    }
    TargetPixelSummary {
        matching_pixels,
        bounds: (matching_pixels != 0).then_some((minimum, maximum)),
    }
}

fn signal_samples(
    pixels: &NativeClientPixelCapture,
    expected: ExpectedNativeColor,
    manifest: &PlatformPulseVisualAdjudicationContract,
) -> Vec<NativePixelSampleObservation> {
    let point = scaled_point(
        pixels,
        manifest.source_signal_logical_point(),
        manifest.logical_client_extent(),
    );
    let xs = [
        point[0].saturating_sub(1),
        point[0],
        point[0].saturating_add(1),
    ];
    let ys = [
        point[1].saturating_sub(1),
        point[1],
        point[1].saturating_add(1),
    ];
    let expected_rgb = expected.rgb(manifest);
    let mut samples = Vec::with_capacity(9);
    let width = pixels.width() as usize;
    for y in ys {
        for x in xs {
            let offset = (y as usize * width + x as usize) * 4;
            if let Some(pixel) = pixels.rgba().get(offset..offset + 4) {
                let rgba = [pixel[0], pixel[1], pixel[2], pixel[3]];
                let matches_expected_color =
                    rgba[..3]
                        .iter()
                        .zip(expected_rgb)
                        .all(|(&observed, expected)| {
                            observed.abs_diff(expected) <= manifest.channel_tolerance()
                        });
                samples.push(NativePixelSampleObservation {
                    x: x as usize,
                    y: y as usize,
                    rgba,
                    matches_expected_color,
                });
            }
        }
    }
    samples
}

fn scaled_point(
    pixels: &NativeClientPixelCapture,
    logical: [u32; 2],
    logical_extent: [u32; 2],
) -> [u32; 2] {
    [
        ((u64::from(logical[0]) * u64::from(pixels.width()) + u64::from(logical_extent[0] / 2))
            / u64::from(logical_extent[0])) as u32,
        ((u64::from(logical[1]) * u64::from(pixels.height()) + u64::from(logical_extent[1] / 2))
            / u64::from(logical_extent[1])) as u32,
    ]
}

fn pixel_at(pixels: &NativeClientPixelCapture, point: [u32; 2]) -> Option<[u8; 4]> {
    let width = usize::try_from(pixels.width()).ok()?;
    let x = usize::try_from(point[0]).ok()?;
    let y = usize::try_from(point[1]).ok()?;
    let offset = y.checked_mul(width)?.checked_add(x)?.checked_mul(4)?;
    let pixel = pixels.rgba().get(offset..offset + 4)?;
    Some([pixel[0], pixel[1], pixel[2], pixel[3]])
}

impl NativeColorVerdict {
    pub(crate) fn expected(self) -> ExpectedNativeColor {
        self.expected
    }

    pub(crate) fn matching_samples(self) -> usize {
        self.matching_samples
    }

    pub(crate) fn sampled_pixels(self) -> usize {
        self.sampled_pixels
    }
}

impl ExpectedNativeColor {
    fn rgb(self, manifest: &PlatformPulseVisualAdjudicationContract) -> [u8; 3] {
        match self {
            Self::Blue => manifest.blue_rgba()[..3].try_into().expect("RGB prefix"),
            Self::Green => manifest.green_rgba()[..3].try_into().expect("RGB prefix"),
        }
    }
}

impl fmt::Display for NativeColorFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VisualContract(failure) => {
                write!(formatter, "Platform Pulse visual contract: {failure:?}")
            }
            Self::InsufficientPixelSamples => {
                formatter.write_str("native client capture yielded fewer than nine samples")
            }
            Self::ExpectedColorNotVisible {
                expected,
                matching,
                samples,
            } => {
                write!(
                    formatter,
                    "expected {expected:?} was visible in only {matching}/{} samples",
                    samples.len()
                )?;
                for sample in samples {
                    write!(
                        formatter,
                        "; ({}, {}) rgba={:?} match={}",
                        sample.x, sample.y, sample.rgba, sample.matches_expected_color
                    )?;
                }
                Ok(())
            }
            Self::ExpectedTargetNotVisible {
                point,
                rgba,
                capture_extent,
                matching_target_pixels,
                target_pixel_bounds,
            } => {
                write!(
                    formatter,
                    "canonical target point ({}, {}) did not show target color; rgba={rgba:?}; \
                     capture={}x{}; target-colored pixels={matching_target_pixels}; \
                     target bounds={target_pixel_bounds:?}",
                    point[0], point[1], capture_extent[0], capture_extent[1]
                )
            }
            Self::BackgroundPointMissing(expected) => write!(
                formatter,
                "native client capture had no interior {expected:?} background point"
            ),
        }
    }
}
