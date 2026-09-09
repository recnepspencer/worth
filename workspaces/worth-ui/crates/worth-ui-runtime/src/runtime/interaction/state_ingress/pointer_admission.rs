use worth_ui_host_contract::UiHostObservationReport;

use crate::runtime::interaction::gesture::UiPointerGestureStopReason;
use crate::runtime::interaction::{UiPointerPresenceAdmissionDenial, UiPrimaryPointerKind};

pub(super) struct UiPointerAdmission {
    kind: Option<UiPrimaryPointerKind>,
    denials: Vec<UiPointerPresenceAdmissionDenial>,
    stop: Option<(
        worth_ui_host_contract::UiHostPointerIdentity,
        UiPointerGestureStopReason,
    )>,
}

impl UiPointerAdmission {
    pub(super) fn from_report(report: &UiHostObservationReport) -> Self {
        match crate::runtime::interaction::pointer_admission::admit(report) {
            Ok(kind) => Self {
                kind,
                denials: Vec::new(),
                stop: None,
            },
            Err(denial) => {
                let mut admission = Self {
                    kind: None,
                    denials: Vec::new(),
                    stop: None,
                };
                admission.deny(denial);
                admission
            }
        }
    }

    pub(super) fn kind(&self) -> Option<UiPrimaryPointerKind> {
        self.kind
    }

    pub(super) fn denied(&self) -> bool {
        !self.denials.is_empty()
    }

    pub(super) fn deny(&mut self, denial: UiPointerPresenceAdmissionDenial) {
        if let Some(reason) = pointer_stop_reason(&denial) {
            self.stop = Some((denial.pointer(), reason));
        }
        self.denials.push(denial);
    }

    pub(super) fn take_stop(
        &mut self,
    ) -> Option<(
        worth_ui_host_contract::UiHostPointerIdentity,
        UiPointerGestureStopReason,
    )> {
        self.stop.take()
    }

    pub(super) fn into_denials(self) -> Vec<UiPointerPresenceAdmissionDenial> {
        self.denials
    }
}

fn pointer_stop_reason(
    denial: &UiPointerPresenceAdmissionDenial,
) -> Option<UiPointerGestureStopReason> {
    match denial {
        UiPointerPresenceAdmissionDenial::Targeting { denial, .. } => {
            Some(UiPointerGestureStopReason::Targeting(*denial))
        }
        UiPointerPresenceAdmissionDenial::MissingDeviceKind { .. } => {
            Some(UiPointerGestureStopReason::MissingPointerDeviceKind)
        }
        UiPointerPresenceAdmissionDenial::CapacityExceeded { .. } => None,
        UiPointerPresenceAdmissionDenial::PointerKindChanged {
            prior, observed, ..
        } => Some(UiPointerGestureStopReason::PointerDeviceKindChanged {
            expected: *prior,
            observed: *observed,
        }),
    }
}
