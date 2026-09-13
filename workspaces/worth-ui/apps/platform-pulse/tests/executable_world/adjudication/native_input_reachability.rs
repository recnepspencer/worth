use std::fmt;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseIntentInteractionFamily, PlatformPulseIntentRoutingStoppedObservation,
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationEnvelope,
    PlatformPulseNativeInputIngressPosture, PlatformPulseNativeInputReached,
};

use crate::external_observation::{
    NativeClientPixelCapture, NativeClientPixelPoint, NativeInputDeliveryObservation,
    NativeInputProbeKind, ProcessBoundNativeClientAreaObservation,
};

use super::{
    adjudicate_native_color, ExecutableFirstFrameEvidence, ExpectedNativeColor, NativeColorFailure,
    NativeColorVerdict,
};

#[derive(Debug)]
pub(crate) struct ExecutableNativeInputReachabilityEvidence {
    pointer_delivery: NativeInputDeliveryObservation,
    keyboard_delivery: NativeInputDeliveryObservation,
    pointer_reached: PlatformPulseNativeInputReached,
    keyboard_reached: PlatformPulseNativeInputReached,
    pointer_sequence: u64,
    keyboard_routing_stop_sequence: u64,
    keyboard_sequence: u64,
    pixels: NativeClientPixelCapture,
    color: NativeColorVerdict,
}

pub(crate) struct NativeInputFamilyObservation {
    delivery: NativeInputDeliveryObservation,
    envelope: PlatformPulseLifecycleObservationEnvelope,
}

pub(crate) struct NativeInputReachabilityObservationSet {
    pointer: NativeInputFamilyObservation,
    keyboard_routing_stop: PlatformPulseLifecycleObservationEnvelope,
    keyboard: NativeInputFamilyObservation,
    pixels: NativeClientPixelCapture,
}

#[derive(Debug)]
pub(crate) enum ExecutableNativeInputReachabilityFailure {
    DeliveryTargetMismatch,
    DeliveryPointOutsideClient,
    WrongProbeKind,
    PartialDelivery {
        kind: NativeInputProbeKind,
        delivered: u32,
    },
    MissingReachabilityOutcome(NativeInputProbeKind),
    MissingKeyboardRoutingStop,
    InvalidKeyboardRoutingStop,
    ForeignRun,
    UnexpectedSequence {
        kind: NativeInputProbeKind,
        observed: u64,
    },
    UnexpectedPosture {
        kind: NativeInputProbeKind,
        expected: PlatformPulseNativeInputIngressPosture,
        observed: PlatformPulseNativeInputIngressPosture,
    },
    InputFamilyMissing(NativeInputProbeKind),
    BackgroundPoint(NativeColorFailure),
    NativeColor(NativeColorFailure),
}

pub(crate) fn native_input_background_point(
    first_frame: &ExecutableFirstFrameEvidence,
) -> Result<NativeClientPixelPoint, ExecutableNativeInputReachabilityFailure> {
    super::adjudicate_native_background_point(first_frame.pixels(), ExpectedNativeColor::Blue)
        .map_err(ExecutableNativeInputReachabilityFailure::BackgroundPoint)
}

pub(crate) fn adjudicate_native_input_reachability(
    first_frame: &ExecutableFirstFrameEvidence,
    observations: NativeInputReachabilityObservationSet,
) -> Result<ExecutableNativeInputReachabilityEvidence, ExecutableNativeInputReachabilityFailure> {
    let NativeInputReachabilityObservationSet {
        pointer,
        keyboard_routing_stop,
        keyboard,
        pixels,
    } = observations;
    let client = first_frame.client_area();
    require_delivery_target(client, pointer.delivery, NativeInputProbeKind::Pointer)?;
    require_delivery_target(client, keyboard.delivery, NativeInputProbeKind::Keyboard)?;
    let (pointer_reached, pointer_sequence) = require_reached(
        first_frame,
        pointer.envelope,
        NativeInputProbeKind::Pointer,
        5,
        PlatformPulseNativeInputIngressPosture::Retained,
    )?;
    let (keyboard_reached, keyboard_sequence) = require_reached(
        first_frame,
        keyboard.envelope,
        NativeInputProbeKind::Keyboard,
        7,
        PlatformPulseNativeInputIngressPosture::Retained,
    )?;
    let keyboard_routing_stop_sequence =
        require_keyboard_routing_stop(first_frame, keyboard_routing_stop)?;
    let color = adjudicate_native_color(&pixels, ExpectedNativeColor::Blue)
        .map_err(ExecutableNativeInputReachabilityFailure::NativeColor)?;
    Ok(ExecutableNativeInputReachabilityEvidence {
        pointer_delivery: pointer.delivery,
        keyboard_delivery: keyboard.delivery,
        pointer_reached,
        keyboard_reached,
        pointer_sequence,
        keyboard_routing_stop_sequence,
        keyboard_sequence,
        pixels,
        color,
    })
}

impl NativeInputFamilyObservation {
    pub(crate) fn new(
        delivery: NativeInputDeliveryObservation,
        envelope: PlatformPulseLifecycleObservationEnvelope,
    ) -> Self {
        Self { delivery, envelope }
    }
}

impl NativeInputReachabilityObservationSet {
    pub(crate) fn new(
        pointer: NativeInputFamilyObservation,
        keyboard_routing_stop: PlatformPulseLifecycleObservationEnvelope,
        keyboard: NativeInputFamilyObservation,
        pixels: NativeClientPixelCapture,
    ) -> Self {
        Self {
            pointer,
            keyboard_routing_stop,
            keyboard,
            pixels,
        }
    }
}

fn require_keyboard_routing_stop(
    first_frame: &ExecutableFirstFrameEvidence,
    envelope: PlatformPulseLifecycleObservationEnvelope,
) -> Result<u64, ExecutableNativeInputReachabilityFailure> {
    if envelope.run().value() != first_frame.run_identity() {
        return Err(ExecutableNativeInputReachabilityFailure::ForeignRun);
    }
    let PlatformPulseLifecycleObservation::IntentRoutingStopped(
        PlatformPulseIntentRoutingStoppedObservation::Unrouted {
            graph_node,
            interaction,
        },
    ) = envelope.outcome()
    else {
        return Err(ExecutableNativeInputReachabilityFailure::MissingKeyboardRoutingStop);
    };
    if *graph_node == 0 || *interaction != PlatformPulseIntentInteractionFamily::Activate {
        return Err(ExecutableNativeInputReachabilityFailure::InvalidKeyboardRoutingStop);
    }
    let observed = envelope.sequence().value();
    if observed != 6 {
        return Err(
            ExecutableNativeInputReachabilityFailure::UnexpectedSequence {
                kind: NativeInputProbeKind::Keyboard,
                observed,
            },
        );
    }
    Ok(observed)
}

fn require_delivery_target(
    client: ProcessBoundNativeClientAreaObservation,
    delivery: NativeInputDeliveryObservation,
    expected_kind: NativeInputProbeKind,
) -> Result<(), ExecutableNativeInputReachabilityFailure> {
    if delivery.process_id() != client.process_id() || delivery.window() != client.window() {
        return Err(ExecutableNativeInputReachabilityFailure::DeliveryTargetMismatch);
    }
    if delivery.kind() != expected_kind {
        return Err(ExecutableNativeInputReachabilityFailure::WrongProbeKind);
    }
    if delivery.delivered_event_count() != 2 {
        return Err(ExecutableNativeInputReachabilityFailure::PartialDelivery {
            kind: expected_kind,
            delivered: delivery.delivered_event_count(),
        });
    }
    let (x, y) = delivery.qualified_screen_point();
    let bounds = client.bounds();
    if x < bounds.left() || x >= bounds.right() || y < bounds.top() || y >= bounds.bottom() {
        return Err(ExecutableNativeInputReachabilityFailure::DeliveryPointOutsideClient);
    }
    Ok(())
}

fn require_reached(
    first_frame: &ExecutableFirstFrameEvidence,
    envelope: PlatformPulseLifecycleObservationEnvelope,
    kind: NativeInputProbeKind,
    expected_sequence: u64,
    expected_posture: PlatformPulseNativeInputIngressPosture,
) -> Result<(PlatformPulseNativeInputReached, u64), ExecutableNativeInputReachabilityFailure> {
    let reached = match envelope.outcome() {
        PlatformPulseLifecycleObservation::NativeInputReached(reached) => *reached,
        _ => {
            return Err(ExecutableNativeInputReachabilityFailure::MissingReachabilityOutcome(kind))
        }
    };
    if envelope.run().value() != first_frame.run_identity() {
        return Err(ExecutableNativeInputReachabilityFailure::ForeignRun);
    }
    let observed = envelope.sequence().value();
    if observed != expected_sequence {
        return Err(
            ExecutableNativeInputReachabilityFailure::UnexpectedSequence { kind, observed },
        );
    }
    if reached.posture() != expected_posture {
        return Err(
            ExecutableNativeInputReachabilityFailure::UnexpectedPosture {
                kind,
                expected: expected_posture,
                observed: reached.posture(),
            },
        );
    }
    let family_reached = match kind {
        NativeInputProbeKind::Pointer => reached.pointer_button_events() > 0,
        NativeInputProbeKind::Keyboard => reached.keyboard_events() > 0,
    };
    if !family_reached {
        return Err(ExecutableNativeInputReachabilityFailure::InputFamilyMissing(kind));
    }
    Ok((reached, observed))
}

impl ExecutableNativeInputReachabilityEvidence {
    pub(crate) fn delivered_event_count(&self) -> u32 {
        self.pointer_delivery
            .delivered_event_count()
            .saturating_add(self.keyboard_delivery.delivered_event_count())
    }

    pub(crate) fn sequences(&self) -> (u64, u64, u64) {
        (
            self.pointer_sequence,
            self.keyboard_routing_stop_sequence,
            self.keyboard_sequence,
        )
    }

    pub(crate) fn pointer_button_events(&self) -> u64 {
        self.pointer_reached.pointer_button_events()
    }

    pub(crate) fn keyboard_events(&self) -> u64 {
        self.keyboard_reached.keyboard_events()
    }

    pub(crate) fn capture_count(&self) -> u32 {
        self.pixels.capture_count()
    }

    pub(crate) fn matching_blue_samples(&self) -> usize {
        self.color.matching_samples()
    }

    pub(crate) fn sampled_pixels(&self) -> usize {
        self.color.sampled_pixels()
    }
}

impl fmt::Display for ExecutableNativeInputReachabilityFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeliveryTargetMismatch => {
                formatter.write_str("native input did not target the process-bound window")
            }
            Self::DeliveryPointOutsideClient => {
                formatter.write_str("native input point was outside the bound client area")
            }
            Self::WrongProbeKind => {
                formatter.write_str("native input delivery used the wrong probe family")
            }
            Self::PartialDelivery { kind, delivered } => {
                write!(formatter, "{kind:?} input delivered {delivered}/2 events")
            }
            Self::MissingReachabilityOutcome(kind) => {
                write!(formatter, "child did not publish {kind:?} reachability")
            }
            Self::MissingKeyboardRoutingStop => formatter
                .write_str("keyboard probe did not publish the expected unrouted observation"),
            Self::InvalidKeyboardRoutingStop => formatter
                .write_str("keyboard probe published invalid unrouted observation evidence"),
            Self::ForeignRun => formatter.write_str("reachability belongs to a foreign run"),
            Self::UnexpectedSequence { kind, observed } => {
                write!(formatter, "{kind:?} reachability sequence was {observed}")
            }
            Self::UnexpectedPosture {
                kind,
                expected,
                observed,
            } => write!(
                formatter,
                "{kind:?} input posture was {observed:?}, expected {expected:?}"
            ),
            Self::InputFamilyMissing(kind) => {
                write!(formatter, "{kind:?} input did not reach the adapter")
            }
            Self::BackgroundPoint(failure) => {
                write!(formatter, "native input background point: {failure}")
            }
            Self::NativeColor(failure) => {
                write!(
                    formatter,
                    "pre-intent input changed native pixels: {failure}"
                )
            }
        }
    }
}
