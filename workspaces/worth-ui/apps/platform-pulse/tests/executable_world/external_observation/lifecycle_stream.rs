use std::fmt;
use std::io::{BufRead, BufReader};
use std::process::ChildStdout;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationCodecDenial,
    PlatformPulseLifecycleObservationEnvelope,
};

const MAXIMUM_EVENTS: usize = 256;
const MAXIMUM_ENCODED_BYTES: usize = 1_048_576;

pub(crate) struct PlatformPulseLifecycleStream {
    pub(super) receiver: Receiver<LifecycleStreamItem>,
    pub(super) reader: Option<JoinHandle<()>>,
    run: Option<String>,
    accepted_events: usize,
    accepted_bytes: usize,
    accepted_trace: Vec<LifecycleTraceEntry>,
    accepted_envelopes: Vec<PlatformPulseLifecycleObservationEnvelope>,
    terminal_observed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LifecycleStreamMeasurement {
    accepted_events: usize,
    accepted_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct LifecycleTraceEntry {
    run: String,
    sequence: u64,
    outcome: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LifecycleFailureSnapshot {
    measurement: LifecycleStreamMeasurement,
    trace: Vec<LifecycleTraceEntry>,
}

#[derive(Debug)]
pub(crate) enum PlatformPulseLifecycleStreamFailure {
    Read(String),
    Decode(PlatformPulseLifecycleObservationCodecDenial),
    Deadline,
    ReaderDisconnected,
    EventBudgetExceeded,
    ByteBudgetExceeded,
    ForeignRun,
    OutOfOrder { expected: u64, observed: u64 },
    TrailingAfterTerminal,
    MissingEndOfStream,
    ReaderPanicked,
    UnexpectedDuringQuiescence,
}

impl fmt::Display for PlatformPulseLifecycleStreamFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(formatter, "read lifecycle stdout: {error}"),
            Self::Decode(denial) => write!(formatter, "decode lifecycle envelope: {denial:?}"),
            Self::Deadline => formatter.write_str("lifecycle observation deadline elapsed"),
            Self::ReaderDisconnected => {
                formatter.write_str("lifecycle observation reader disconnected")
            }
            Self::EventBudgetExceeded => formatter.write_str("lifecycle event budget exceeded"),
            Self::ByteBudgetExceeded => {
                formatter.write_str("lifecycle encoded-byte budget exceeded")
            }
            Self::ForeignRun => formatter.write_str("lifecycle envelope belongs to a foreign run"),
            Self::OutOfOrder { expected, observed } => write!(
                formatter,
                "lifecycle envelope out of order: expected {expected}, observed {observed}"
            ),
            Self::TrailingAfterTerminal => {
                formatter.write_str("lifecycle event appeared after terminal observation")
            }
            Self::MissingEndOfStream => {
                formatter.write_str("lifecycle stdout ended before a terminal observation")
            }
            Self::ReaderPanicked => formatter.write_str("lifecycle reader thread panicked"),
            Self::UnexpectedDuringQuiescence => {
                formatter.write_str("lifecycle event appeared during the observed idle interval")
            }
        }
    }
}

pub(super) enum LifecycleStreamItem {
    Envelope {
        envelope: PlatformPulseLifecycleObservationEnvelope,
        encoded_bytes: usize,
    },
    Failure(PlatformPulseLifecycleStreamFailure),
    End,
}

impl PlatformPulseLifecycleStream {
    pub(crate) fn read(stdout: ChildStdout) -> Self {
        let (sender, receiver) = mpsc::channel();
        let reader = thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let line = match line {
                    Ok(line) => line,
                    Err(error) => {
                        let _ = sender.send(LifecycleStreamItem::Failure(
                            PlatformPulseLifecycleStreamFailure::Read(error.to_string()),
                        ));
                        return;
                    }
                };
                let encoded_bytes = line.len().saturating_add(1);
                match PlatformPulseLifecycleObservationEnvelope::decode_prefixed_line(&line) {
                    Ok(envelope) => {
                        if sender
                            .send(LifecycleStreamItem::Envelope {
                                envelope,
                                encoded_bytes,
                            })
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(denial) => {
                        let _ = sender.send(LifecycleStreamItem::Failure(
                            PlatformPulseLifecycleStreamFailure::Decode(denial),
                        ));
                        return;
                    }
                }
            }
            let _ = sender.send(LifecycleStreamItem::End);
        });
        Self {
            receiver,
            reader: Some(reader),
            run: None,
            accepted_events: 0,
            accepted_bytes: 0,
            accepted_trace: Vec::new(),
            accepted_envelopes: Vec::new(),
            terminal_observed: false,
        }
    }

    pub(crate) fn next(
        &mut self,
        deadline: Instant,
    ) -> Result<PlatformPulseLifecycleObservationEnvelope, PlatformPulseLifecycleStreamFailure>
    {
        if self.terminal_observed {
            return Err(PlatformPulseLifecycleStreamFailure::TrailingAfterTerminal);
        }
        let item = self.receive(deadline)?;
        let LifecycleStreamItem::Envelope {
            envelope,
            encoded_bytes,
        } = item
        else {
            return match item {
                LifecycleStreamItem::Failure(failure) => Err(failure),
                LifecycleStreamItem::End => {
                    Err(PlatformPulseLifecycleStreamFailure::MissingEndOfStream)
                }
                LifecycleStreamItem::Envelope { .. } => unreachable!(),
            };
        };
        self.accept(&envelope, encoded_bytes)?;
        if matches!(
            envelope.outcome(),
            PlatformPulseLifecycleObservation::ShutdownCompleted(_)
                | PlatformPulseLifecycleObservation::TerminalFailure(_)
        ) {
            self.terminal_observed = true;
        }
        Ok(envelope)
    }

    pub(crate) fn finish(
        &mut self,
        deadline: Instant,
    ) -> Result<(), PlatformPulseLifecycleStreamFailure> {
        match self.receive(deadline)? {
            LifecycleStreamItem::End => self.join_reader(),
            LifecycleStreamItem::Envelope { .. } => {
                Err(PlatformPulseLifecycleStreamFailure::TrailingAfterTerminal)
            }
            LifecycleStreamItem::Failure(failure) => Err(failure),
        }
    }

    pub(crate) fn measurement(&self) -> LifecycleStreamMeasurement {
        LifecycleStreamMeasurement {
            accepted_events: self.accepted_events,
            accepted_bytes: self.accepted_bytes,
        }
    }

    pub(crate) fn failure_snapshot(&self) -> LifecycleFailureSnapshot {
        LifecycleFailureSnapshot {
            measurement: self.measurement(),
            trace: self.accepted_trace.clone(),
        }
    }

    pub(crate) fn observe_quiescent_until(
        &mut self,
        deadline: Instant,
    ) -> Result<LifecycleStreamMeasurement, PlatformPulseLifecycleStreamFailure> {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or(PlatformPulseLifecycleStreamFailure::Deadline)?;
        match self.receiver.recv_timeout(remaining) {
            Err(RecvTimeoutError::Timeout) => Ok(self.measurement()),
            Err(RecvTimeoutError::Disconnected) => {
                Err(PlatformPulseLifecycleStreamFailure::ReaderDisconnected)
            }
            Ok(LifecycleStreamItem::Envelope {
                envelope,
                encoded_bytes,
            }) => {
                self.accept(&envelope, encoded_bytes)?;
                Err(PlatformPulseLifecycleStreamFailure::UnexpectedDuringQuiescence)
            }
            Ok(LifecycleStreamItem::Failure(failure)) => Err(failure),
            Ok(LifecycleStreamItem::End) => {
                Err(PlatformPulseLifecycleStreamFailure::MissingEndOfStream)
            }
        }
    }

    pub(crate) fn accepted_envelopes(&self) -> &[PlatformPulseLifecycleObservationEnvelope] {
        &self.accepted_envelopes
    }

    fn accept(
        &mut self,
        envelope: &PlatformPulseLifecycleObservationEnvelope,
        encoded_bytes: usize,
    ) -> Result<(), PlatformPulseLifecycleStreamFailure> {
        let next_events = self
            .accepted_events
            .checked_add(1)
            .ok_or(PlatformPulseLifecycleStreamFailure::EventBudgetExceeded)?;
        if next_events > MAXIMUM_EVENTS {
            return Err(PlatformPulseLifecycleStreamFailure::EventBudgetExceeded);
        }
        let next_bytes = self
            .accepted_bytes
            .checked_add(encoded_bytes)
            .ok_or(PlatformPulseLifecycleStreamFailure::ByteBudgetExceeded)?;
        if next_bytes > MAXIMUM_ENCODED_BYTES {
            return Err(PlatformPulseLifecycleStreamFailure::ByteBudgetExceeded);
        }
        let observed_run = envelope.run().value();
        if let Some(run) = &self.run {
            if run != observed_run {
                return Err(PlatformPulseLifecycleStreamFailure::ForeignRun);
            }
        } else {
            self.run = Some(observed_run.to_owned());
        }
        let expected = next_events as u64;
        let observed = envelope.sequence().value();
        if observed != expected {
            return Err(PlatformPulseLifecycleStreamFailure::OutOfOrder { expected, observed });
        }
        self.accepted_events = next_events;
        self.accepted_bytes = next_bytes;
        self.accepted_trace
            .push(LifecycleTraceEntry::from_envelope(envelope));
        self.accepted_envelopes.push(envelope.clone());
        Ok(())
    }

    fn receive(
        &self,
        deadline: Instant,
    ) -> Result<LifecycleStreamItem, PlatformPulseLifecycleStreamFailure> {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or(PlatformPulseLifecycleStreamFailure::Deadline)?;
        self.receiver
            .recv_timeout(remaining)
            .map_err(|error| match error {
                RecvTimeoutError::Timeout => PlatformPulseLifecycleStreamFailure::Deadline,
                RecvTimeoutError::Disconnected => {
                    PlatformPulseLifecycleStreamFailure::ReaderDisconnected
                }
            })
    }

    fn join_reader(&mut self) -> Result<(), PlatformPulseLifecycleStreamFailure> {
        let reader = self
            .reader
            .take()
            .ok_or(PlatformPulseLifecycleStreamFailure::ReaderDisconnected)?;
        reader
            .join()
            .map_err(|_| PlatformPulseLifecycleStreamFailure::ReaderPanicked)
    }
}

impl LifecycleStreamMeasurement {
    pub(crate) fn accepted_events(self) -> usize {
        self.accepted_events
    }

    pub(crate) fn accepted_bytes(self) -> usize {
        self.accepted_bytes
    }
}

impl LifecycleFailureSnapshot {
    pub(crate) fn measurement(&self) -> LifecycleStreamMeasurement {
        self.measurement
    }

    pub(crate) fn trace(&self) -> &[LifecycleTraceEntry] {
        &self.trace
    }
}

impl LifecycleTraceEntry {
    fn from_envelope(envelope: &PlatformPulseLifecycleObservationEnvelope) -> Self {
        let outcome = match envelope.outcome() {
            PlatformPulseLifecycleObservation::ThemeSwitchSettled(_) => "theme_switch_settled",
            PlatformPulseLifecycleObservation::ProcessStarted(_) => "process_started",
            PlatformPulseLifecycleObservation::FirstFramePublished(_) => "first_frame_published",
            PlatformPulseLifecycleObservation::NativeInputReached(_) => "native_input_reached",
            PlatformPulseLifecycleObservation::IntentInputAdmitted(_) => "intent_input_admitted",
            PlatformPulseLifecycleObservation::IntentExecutorStarted(_) => {
                "intent_executor_started"
            }
            PlatformPulseLifecycleObservation::IntentPosturePublished(_) => {
                "intent_posture_published"
            }
            PlatformPulseLifecycleObservation::IntentRoutingStopped(_) => "intent_routing_stopped",
            PlatformPulseLifecycleObservation::SemanticFocusPublished(_) => {
                "semantic_focus_published"
            }
            PlatformPulseLifecycleObservation::PortalDismissed(_) => "portal_dismissed",
            PlatformPulseLifecycleObservation::IntentCausalTrace(_) => "intent_causal_trace",
            PlatformPulseLifecycleObservation::QueryAction(_) => "query_action",
            PlatformPulseLifecycleObservation::QueryProjectionIssued(_) => {
                "query_projection_issued"
            }
            PlatformPulseLifecycleObservation::QueryProjectionPublished(_) => {
                "query_projection_published"
            }
            PlatformPulseLifecycleObservation::VisualSnapshotCaptured(_) => {
                "visual_snapshot_captured"
            }
            PlatformPulseLifecycleObservation::VisualPointTrace(_) => "visual_point_trace",
            PlatformPulseLifecycleObservation::VisualOverlayPublished(_) => {
                "visual_overlay_published"
            }
            PlatformPulseLifecycleObservation::VisualOverlayCleared(_) => "visual_overlay_cleared",
            PlatformPulseLifecycleObservation::VisualSnapshotRetired(_) => {
                "visual_snapshot_retired"
            }
            PlatformPulseLifecycleObservation::RebindPublished(_) => "rebind_published",
            PlatformPulseLifecycleObservation::RebindDeniedPreserving(_) => {
                "replacement_denied_preserving"
            }
            PlatformPulseLifecycleObservation::VisualComparison(_) => "visual_comparison",
            PlatformPulseLifecycleObservation::ShutdownCompleted(_) => "shutdown_completed",
            PlatformPulseLifecycleObservation::TerminalFailure(_) => "terminal_failure",
        };
        Self {
            run: envelope.run().value().to_owned(),
            sequence: envelope.sequence().value(),
            outcome: match envelope.outcome() {
                PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                    format!("terminal_failure:{:?}", failure.family())
                }
                _ => outcome.to_owned(),
            },
        }
    }
}
