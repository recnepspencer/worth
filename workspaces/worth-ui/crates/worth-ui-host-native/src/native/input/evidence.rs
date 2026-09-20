use worth_ui_host_contract::{
    UiHostObservationFamily, UiHostObservationPayload, UiHostObservationReport,
    UiHostObservationTimeBasis,
};

use super::{UiNativePointerButtonObservation, UiNativeScrollDeltaObservation};

#[derive(Clone, Debug, Default)]
pub(super) struct UiNativeInputObservationEvidence {
    retained_batch_count: u64,
    retained_event_count: u64,
    first_retained_sequence: Option<u64>,
    last_retained_sequence: Option<u64>,
    family_counts: [u64; 11],
    last_pointer_button: Option<UiNativePointerButtonObservation>,
    last_vertical_scroll: Option<UiNativeScrollDeltaObservation>,
    last_horizontal_scroll: Option<UiNativeScrollDeltaObservation>,
    profile_transition_count: u64,
}

impl UiNativeInputObservationEvidence {
    pub(super) fn record_batch(&mut self, reports: &[UiHostObservationReport]) {
        self.retained_batch_count = self.retained_batch_count.saturating_add(1);
        self.retained_event_count = self
            .retained_event_count
            .saturating_add(reports.len() as u64);
        for report in reports {
            let sequence = report.sequence().value();
            self.first_retained_sequence.get_or_insert(sequence);
            self.last_retained_sequence = Some(sequence);
            if let Some(index) = family_index(report.family()) {
                self.family_counts[index] = self.family_counts[index].saturating_add(1);
            }
            if let Some(pointer_button) = pointer_button_observation(report) {
                self.last_pointer_button = Some(pointer_button);
            }
            if let Some(scroll) = scroll_delta_observation(report) {
                if scroll.y_subpixels() != 0 {
                    self.last_vertical_scroll = Some(scroll);
                }
                if scroll.x_subpixels() != 0 {
                    self.last_horizontal_scroll = Some(scroll);
                }
            }
        }
    }

    pub(super) fn record_profile_transition(&mut self) {
        self.profile_transition_count = self.profile_transition_count.saturating_add(1);
    }

    pub(super) fn retained_batch_count(&self) -> u64 {
        self.retained_batch_count
    }

    pub(super) fn retained_event_count(&self) -> u64 {
        self.retained_event_count
    }

    pub(super) fn first_retained_sequence(&self) -> Option<u64> {
        self.first_retained_sequence
    }

    pub(super) fn last_retained_sequence(&self) -> Option<u64> {
        self.last_retained_sequence
    }

    pub(super) fn family_counts(&self) -> [u64; 11] {
        self.family_counts
    }

    pub(super) fn last_pointer_button(&self) -> Option<UiNativePointerButtonObservation> {
        self.last_pointer_button
    }

    pub(super) fn last_vertical_scroll(&self) -> Option<UiNativeScrollDeltaObservation> {
        self.last_vertical_scroll
    }

    pub(super) fn last_horizontal_scroll(&self) -> Option<UiNativeScrollDeltaObservation> {
        self.last_horizontal_scroll
    }

    pub(super) fn profile_transition_count(&self) -> u64 {
        self.profile_transition_count
    }
}

fn family_index(family: UiHostObservationFamily) -> Option<usize> {
    match family {
        UiHostObservationFamily::Viewport => Some(0),
        UiHostObservationFamily::DeviceScale => Some(1),
        UiHostObservationFamily::PointerMotion => Some(2),
        UiHostObservationFamily::PointerButton => Some(3),
        UiHostObservationFamily::Keyboard => Some(4),
        UiHostObservationFamily::WindowFocus => Some(5),
        UiHostObservationFamily::ScrollDelta => Some(6),
        UiHostObservationFamily::Clock => Some(7),
        UiHostObservationFamily::Tick => Some(8),
        UiHostObservationFamily::TextComposition => Some(9),
        UiHostObservationFamily::ImeComposition => Some(10),
    }
}

fn pointer_button_observation(
    report: &UiHostObservationReport,
) -> Option<UiNativePointerButtonObservation> {
    let UiHostObservationPayload::PointerButton { position, .. } = report.payload() else {
        return None;
    };
    let UiHostObservationTimeBasis::HostMonotonicMillis(event_tick) = report.time_basis() else {
        return None;
    };
    Some(UiNativePointerButtonObservation::reported(
        report.sequence().value(),
        event_tick,
        *position,
    ))
}

fn scroll_delta_observation(
    report: &UiHostObservationReport,
) -> Option<UiNativeScrollDeltaObservation> {
    let UiHostObservationPayload::ScrollDelta {
        precision,
        x_subpixels,
        y_subpixels,
        ..
    } = report.payload()
    else {
        return None;
    };
    let UiHostObservationTimeBasis::HostMonotonicMillis(event_tick) = report.time_basis() else {
        return None;
    };
    Some(UiNativeScrollDeltaObservation::reported(
        report.sequence().value(),
        event_tick,
        *x_subpixels,
        *y_subpixels,
        *precision,
    ))
}
