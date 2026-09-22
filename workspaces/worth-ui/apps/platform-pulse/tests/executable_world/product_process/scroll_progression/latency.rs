//! Warm native timing: OS compositor timestamps, bounded capture, raw intervals.
use super::{NativeBoundExecutableWorld, PlatformPulseScrollJourneyFailure as Failure};
use crate::adjudication::{physical_px, RecentActivityScrollGeometry};
use crate::external_observation::{
    NativeClientPixelCapture, NativeClientPixelPoint, NativeTimedClientPixelCapture,
};
use crate::native_platform::NativePlatformContract;
use std::time::Duration;

mod accepted_samples;
#[cfg(test)]
mod tests;
pub(super) mod thumb;
mod trace;
mod visible_change;

#[derive(Clone, Debug)]
pub(crate) struct VisibleScrollFrame {
    pub(super) qpc_100ns: i64,
    pub(super) thumb_top_px: u32,
    pub(super) thumb_length_px: u32,
}

/// Only the current predecessor retains pixels; history keeps bounded pose/time
/// observations rather than accumulating a movie.
pub(super) struct ObservedScrollFrame {
    visible: VisibleScrollFrame,
    pixels: NativeClientPixelCapture,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ScrollLatencyEvidence {
    pub(super) input_brackets: Vec<Duration>,
    pub(super) copy_cost: Vec<Duration>,
    pub(super) acquisition_lag: Vec<Duration>,
    pub(super) first_change: Vec<Duration>,
    pub(super) frame_gaps: Vec<Duration>,
    pub(super) settlement: Vec<Duration>,
    pub(super) trace_input_counts: Vec<usize>,
    pub(super) trace_spans: Vec<Duration>,
    pub(super) visible_frames: Vec<VisibleScrollFrame>,
    pub(super) dpi: u32,
}

impl ScrollLatencyEvidence {
    pub(crate) fn first_change(&self) -> &[Duration] {
        &self.first_change
    }
    pub(crate) fn frame_gaps(&self) -> &[Duration] {
        &self.frame_gaps
    }
    pub(crate) fn settlement(&self) -> &[Duration] {
        &self.settlement
    }
    pub(crate) fn delivery_uncertainty(&self) -> Duration {
        percentile(&self.input_brackets, 1000)
    }
    pub(crate) fn first_change_percentile(&self, p: usize) -> Duration {
        percentile(&self.first_change, p)
    }
    pub(crate) fn frame_gap_percentile(&self, p: usize) -> Duration {
        percentile(&self.frame_gaps, p)
    }
    pub(crate) fn trace_spans(&self) -> &[Duration] {
        &self.trace_spans
    }

    pub(crate) fn report(&self) -> String {
        format!(
            "DXGI desktop-presentation QPC timing at {} DPI; active traces {:?}, inputs {:?}\ninput-to-first-change p95={:?}; visible gap p95={:?}, p99={:?}, max={:?}; settlement max={:?}\nSendInput brackets max={:?}; capture copy max={:?}; acquisition lag max={:?}\nraw first-change intervals={:?}\nraw visible-frame gaps={:?}\nraw settlements={:?}\nraw input uncertainty={:?}\nraw capture copy={:?}\nraw acquisition lag={:?}",
            self.dpi, self.trace_spans, self.trace_input_counts,
            self.first_change_percentile(950), self.frame_gap_percentile(950),
            self.frame_gap_percentile(990), percentile(&self.frame_gaps, 1000),
            percentile(&self.settlement, 1000), self.delivery_uncertainty(),
            percentile(&self.copy_cost, 1000), percentile(&self.acquisition_lag, 1000),
            self.first_change, self.frame_gaps, self.settlement,
            self.input_brackets, self.copy_cost, self.acquisition_lag,
        )
    }

    pub(super) fn observe(
        &mut self,
        frame: NativeTimedClientPixelCapture,
        strip: [u32; 4],
    ) -> Result<ObservedScrollFrame, Failure> {
        if self.copy_cost.len() >= 4096 {
            return Err(Failure::InputDelivery("capture trace capacity exceeded"));
        }
        self.copy_cost
            .push(interval(frame.acquired_qpc_100ns, frame.copied_qpc_100ns)?);
        self.acquisition_lag.push(interval(
            frame.captured_qpc_100ns,
            frame.acquired_qpc_100ns,
        )?);
        let (top, length) = thumb::observed(&frame.pixels, self.dpi, strip)?;
        Ok(ObservedScrollFrame {
            visible: VisibleScrollFrame {
                qpc_100ns: frame.captured_qpc_100ns,
                thumb_top_px: top,
                thumb_length_px: length,
            },
            pixels: frame.pixels,
        })
    }
}

pub(super) fn measure_wheel_latency(
    world: &mut NativeBoundExecutableWorld,
    dpi: u32,
    interior: NativeClientPixelPoint,
    mut expected_offset: f64,
    notch_points: f64,
) -> Result<ScrollLatencyEvidence, Failure> {
    let input = world
        .platform
        .prepare_wheel_input(&world.native_client, interior)
        .map_err(Failure::Native)?;
    let exposed = world
        .platform
        .expose_client_area(&world.native_client)
        .map_err(Failure::Native)?;
    let region = RecentActivityScrollGeometry.viewport_points();
    // Trailing column ink and the complete thumb, not a full-window movie.
    let strip = [region[0] + region[2] - 128.0, region[1], 128.0, region[3]]
        .map(|v| physical_px(v, dpi) as u32);
    let mut stream = world
        .platform
        .start_capture_stream(&exposed, strip)
        .map_err(Failure::Native)?;
    let mut evidence = ScrollLatencyEvidence {
        dpi,
        ..Default::default()
    };
    let first = stream
        .next(Duration::from_secs(3))
        .map_err(Failure::Native)?
        .ok_or(Failure::InputDelivery("compositor capture did not start"))?;
    let mut previous = evidence.observe(first, strip)?;
    if !thumb::at_offset(&previous.visible, expected_offset, dpi) {
        return Err(Failure::InputDelivery(
            "timing baseline does not match adjudicated drag offset",
        ));
    }
    // Isolated warm notches establish causal input-to-first-change latency.
    for notch in 0..12 {
        trace::isolated(
            &mut stream,
            &input,
            strip,
            if notch % 2 == 0 { 1 } else { -1 },
            &mut previous,
            &mut evidence,
            &mut expected_offset,
            notch_points,
        )?;
    }
    for _ in 0..3 {
        trace::active(
            &mut stream,
            &input,
            strip,
            &mut previous,
            &mut evidence,
            &mut expected_offset,
            notch_points,
        )?;
    }
    stream.finish().map_err(Failure::Native)?;
    Ok(evidence)
}

pub(super) fn interval(start: i64, end: i64) -> Result<Duration, Failure> {
    let ticks = end
        .checked_sub(start)
        .and_then(|v| u64::try_from(v).ok())
        .ok_or(Failure::InputDelivery(
            "nonmonotonic native timing interval",
        ))?;
    ticks
        .checked_mul(100)
        .map(Duration::from_nanos)
        .ok_or(Failure::InputDelivery("native timing interval overflow"))
}

fn percentile(samples: &[Duration], permille: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (permille * sorted.len()).div_ceil(1000).saturating_sub(1);
    sorted.get(rank).copied().unwrap_or_default()
}
