//! Native input scheduling and observed settlement; quiet never substitutes for
//! reaching the independently calculated target.
use super::*;
use crate::external_observation::NativeInputDeliveryTiming;
use crate::native_platform::{WindowsCaptureStream, WindowsPreparedWheelInput};
use std::time::Instant;

const QUIET: Duration = Duration::from_millis(150);
const TAIL_DEADLINE: Duration = Duration::from_secs(2);

fn deliver(
    input: &WindowsPreparedWheelInput<'_>,
    direction: i32,
    evidence: &mut ScrollLatencyEvidence,
) -> Result<NativeInputDeliveryTiming, Failure> {
    let issued = input.deliver_notch(direction).map_err(Failure::Native)?;
    evidence
        .input_brackets
        .push(interval(issued.before_qpc_100ns, issued.after_qpc_100ns)?);
    Ok(issued)
}

fn advance_target(offset: &mut f64, direction: i32, notch: f64) {
    *offset = (*offset + f64::from(direction) * notch)
        .clamp(0.0, RecentActivityScrollGeometry.max_offset_points());
}

fn record_change(
    current: &ObservedScrollFrame,
    previous: &ObservedScrollFrame,
    last: &mut Option<i64>,
    evidence: &mut ScrollLatencyEvidence,
) -> Result<bool, Failure> {
    if !visible_change::changed(current, previous)? {
        return Ok(false);
    }
    let timestamp = current.visible.qpc_100ns;
    if let Some(previous) = *last {
        evidence.frame_gaps.push(interval(previous, timestamp)?);
    }
    *last = Some(timestamp);
    evidence.visible_frames.push(current.visible.clone());
    Ok(true)
}

pub(super) fn settled(
    frame: &VisibleScrollFrame,
    expected: f64,
    dpi: u32,
    quiet: Duration,
) -> bool {
    thumb::at_offset(frame, expected, dpi) && quiet >= QUIET
}

pub(super) fn isolated(
    stream: &mut WindowsCaptureStream<'_>,
    input: &WindowsPreparedWheelInput<'_>,
    strip: [u32; 4],
    direction: i32,
    previous: &mut ObservedScrollFrame,
    evidence: &mut ScrollLatencyEvidence,
    expected_offset: &mut f64,
    notch: f64,
) -> Result<(), Failure> {
    let issued = deliver(input, direction, evidence)?;
    advance_target(expected_offset, direction, notch);
    let started = Instant::now();
    let mut last_motion_observed = started;
    let mut first = None;
    let mut last = None;
    while started.elapsed() < TAIL_DEADLINE {
        if let Some(frame) = stream
            .next(Duration::from_millis(2))
            .map_err(Failure::Native)?
        {
            let current = evidence.observe(frame, strip)?;
            if current.visible.qpc_100ns > issued.after_qpc_100ns
                && record_change(&current, previous, &mut last, evidence)?
            {
                first.get_or_insert(interval(
                    issued.before_qpc_100ns,
                    current.visible.qpc_100ns,
                )?);
                last_motion_observed = Instant::now();
            }
            *previous = current;
        }
        if first.is_some()
            && settled(
                &previous.visible,
                *expected_offset,
                evidence.dpi,
                last_motion_observed.elapsed(),
            )
        {
            evidence.first_change.push(first.expect("visible response"));
            evidence.settlement.push(interval(
                issued.before_qpc_100ns,
                last.expect("visible response"),
            )?);
            return Ok(());
        }
    }
    Err(Failure::InputDelivery(
        "isolated notch never reached its expected quiet target before deadline",
    ))
}

pub(super) fn active(
    stream: &mut WindowsCaptureStream<'_>,
    input: &WindowsPreparedWheelInput<'_>,
    strip: [u32; 4],
    previous: &mut ObservedScrollFrame,
    evidence: &mut ScrollLatencyEvidence,
    expected_offset: &mut f64,
    notch: f64,
) -> Result<(), Failure> {
    const SPAN: Duration = Duration::from_secs(10);
    const PERIOD: Duration = Duration::from_millis(80);
    let started = Instant::now();
    let mut last_motion_observed = started;
    let mut inputs = 0_u32;
    let mut first_issue = None;
    let mut last_issue = None;
    let mut last_change = None;
    let mut response: Option<(NativeInputDeliveryTiming, i32)> = None;
    let mut last_direction = 0;
    let mut observed_active_span = None;
    while started.elapsed() < SPAN + TAIL_DEADLINE {
        let elapsed = started.elapsed();
        if elapsed >= SPAN {
            observed_active_span.get_or_insert(elapsed);
        }
        if elapsed < SPAN && elapsed >= PERIOD * inputs {
            if elapsed.saturating_sub(PERIOD * inputs) > Duration::from_millis(25) {
                return Err(Failure::InputDelivery(
                    "timing harness missed its input schedule",
                ));
            }
            if response.is_some() {
                return Err(Failure::InputDelivery(
                    "reversal did not visibly respond before the next input",
                ));
            }
            let direction = if inputs % 4 < 2 { 1 } else { -1 };
            let issued = deliver(input, direction, evidence)?;
            advance_target(expected_offset, direction, notch);
            if direction != last_direction {
                response = Some((issued, direction));
            }
            first_issue.get_or_insert(issued.before_qpc_100ns);
            last_issue = Some(issued);
            last_direction = direction;
            inputs += 1;
        }
        if let Some(frame) = stream
            .next(Duration::from_millis(2))
            .map_err(Failure::Native)?
        {
            let current = evidence.observe(frame, strip)?;
            let first = first_issue.expect("trace issues input before capture");
            if current.visible.qpc_100ns > first
                && record_change(&current, previous, &mut last_change, evidence)?
            {
                last_motion_observed = Instant::now();
                if let Some((issued, direction)) = response {
                    let movement = i64::from(current.visible.thumb_top_px)
                        - i64::from(previous.visible.thumb_top_px);
                    if current.visible.qpc_100ns > issued.after_qpc_100ns
                        && movement.signum() == i64::from(direction)
                    {
                        evidence.first_change.push(interval(
                            issued.before_qpc_100ns,
                            current.visible.qpc_100ns,
                        )?);
                        response = None;
                    }
                }
            }
            *previous = current;
        }
        if let Some(active_span) = observed_active_span {
            if response.is_none()
                && inputs >= 120
                && settled(
                    &previous.visible,
                    *expected_offset,
                    evidence.dpi,
                    last_motion_observed.elapsed(),
                )
            {
                let issued = last_issue.expect("active inputs exist");
                evidence.settlement.push(interval(
                    issued.before_qpc_100ns,
                    last_change.ok_or(Failure::WheelNeverMoved(0))?,
                )?);
                evidence.trace_spans.push(active_span);
                evidence.trace_input_counts.push(inputs as usize);
                return Ok(());
            }
        }
    }
    Err(Failure::InputDelivery(
        "active trace never reached its expected quiet target before deadline",
    ))
}
