use worth_ui_host_contract::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationLoss,
    UiHostObservationPayload, UiHostObservationReport, UiHostObservationSequence,
    UiHostObservationSequenceRange, UiHostObservationTimeBasis, UiHostPointerDeviceKind,
};

use super::{
    UiNativeInputObservationDisposition, UiNativeInputObservationState,
    UiNativeInputObservationStop,
};

impl UiNativeInputObservationState {
    pub(in crate::native::input) fn emit_payloads<I>(
        &mut self,
        payloads: I,
    ) -> UiNativeInputObservationDisposition
    where
        I: IntoIterator<Item = UiHostObservationPayload>,
    {
        let event_tick = self.event_tick;
        self.emit_payloads_at(payloads, event_tick)
    }

    fn emit_payloads_at<I>(
        &mut self,
        payloads: I,
        event_tick: u64,
    ) -> UiNativeInputObservationDisposition
    where
        I: IntoIterator<Item = UiHostObservationPayload>,
    {
        if self.terminal_stop.is_some() {
            return UiNativeInputObservationDisposition::Stopped;
        }
        let Some((protocol, host_session, presentation)) = self.completed else {
            self.record_stop(UiNativeInputObservationStop::NoPresentationBasis);
            return UiNativeInputObservationDisposition::Ignored;
        };
        let payloads = payloads.into_iter().collect::<Vec<_>>();
        let Some(first_value) = self.next_sequence else {
            self.record_terminal_stop(UiNativeInputObservationStop::ObservationSequenceExhausted);
            return UiNativeInputObservationDisposition::Stopped;
        };
        let Some(last_value) = first_value.checked_add(payloads.len().saturating_sub(1) as u64)
        else {
            self.record_terminal_stop(UiNativeInputObservationStop::ObservationSequenceExhausted);
            return UiNativeInputObservationDisposition::Stopped;
        };
        let Some(next_sequence) = last_value.checked_add(1) else {
            self.record_terminal_stop(UiNativeInputObservationStop::ObservationSequenceExhausted);
            return UiNativeInputObservationDisposition::Stopped;
        };
        let reports = payloads
            .into_iter()
            .enumerate()
            .map(|(offset, payload)| {
                let report = UiHostObservationReport::new(
                    UiHostObservationSequence::new(first_value + offset as u64),
                    UiHostObservationTimeBasis::HostMonotonicMillis(event_tick),
                    payload,
                );
                let report = if matches!(
                    report.payload(),
                    UiHostObservationPayload::PointerMotion { .. }
                        | UiHostObservationPayload::PointerButton { .. }
                ) {
                    report
                        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
                        .expect("native pointer payloads must accept mouse provenance")
                } else {
                    report
                };
                if report_requires_input_affinity(report.payload()) {
                    let Some(binding) = self.current_input_recipient() else {
                        return report;
                    };
                    report.with_input_affinity(
                        worth_ui_host_contract::UiHostInputRecipientAffinityReceipt::at_event_time(
                            binding,
                            presentation,
                        ),
                    )
                } else {
                    report
                }
            })
            .collect::<Vec<_>>();
        let Some(first) = reports.first().map(UiHostObservationReport::sequence) else {
            return UiNativeInputObservationDisposition::Ignored;
        };
        let last = reports
            .last()
            .expect("non-empty payload collection has a last report")
            .sequence();
        let batch = match UiHostObservationBatch::new(UiHostObservationBatchInput {
            protocol,
            host_session,
            presentation,
            sequences: UiHostObservationSequenceRange::new(first, last),
            loss: UiHostObservationLoss::Complete,
            reports,
        }) {
            Ok(batch) => batch,
            Err(denial) => {
                self.record_terminal_stop(UiNativeInputObservationStop::BatchConstruction(denial));
                return UiNativeInputObservationDisposition::Stopped;
            }
        };
        let evidence_checkpoint = self.evidence.clone();
        self.evidence.record_batch(batch.reports());
        if let Err(denial) = self.retention.retain(batch) {
            self.evidence = evidence_checkpoint;
            self.record_terminal_stop(UiNativeInputObservationStop::Retention(denial));
            return UiNativeInputObservationDisposition::Stopped;
        }
        self.next_sequence = Some(next_sequence);
        UiNativeInputObservationDisposition::Retained
    }

    pub(in crate::native::input) fn emit_profile_transition(
        &mut self,
    ) -> UiNativeInputObservationDisposition {
        let Some(profile) = self.profile else {
            return UiNativeInputObservationDisposition::Ignored;
        };
        if !self.profile_requires_completion {
            return UiNativeInputObservationDisposition::Ignored;
        }
        let transition_tick = self
            .profile_transition_tick
            .take()
            .unwrap_or(self.event_tick);
        self.profile_requires_completion = false;
        let disposition = self.emit_payloads_at(
            [
                UiHostObservationPayload::Viewport {
                    width_subpixels: super::super::profile::logical_subpixels(
                        profile.physical_size[0],
                        profile.scale_factor,
                    ),
                    height_subpixels: super::super::profile::logical_subpixels(
                        profile.physical_size[1],
                        profile.scale_factor,
                    ),
                },
                UiHostObservationPayload::DeviceScale {
                    micros: profile.scale_micros,
                },
            ],
            transition_tick,
        );
        if disposition == UiNativeInputObservationDisposition::Retained {
            self.evidence.record_profile_transition();
        }
        disposition
    }
}

fn report_requires_input_affinity(payload: &UiHostObservationPayload) -> bool {
    matches!(
        payload,
        UiHostObservationPayload::Keyboard { .. }
            | UiHostObservationPayload::TextInput { .. }
            | UiHostObservationPayload::ImeComposition { .. }
    )
}
