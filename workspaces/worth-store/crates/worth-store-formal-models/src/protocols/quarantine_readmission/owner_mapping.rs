use worth_foundational::PhysicalQuarantinePosture;
use worth_store_layout_indexes::integrity::{
    RecoveryLayoutReadmissionAdmissionDenial, RecoveryLayoutReadmissionOutcomeView,
};

use super::QuarantineReadmissionState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineRecordObservation {
    posture: PhysicalQuarantinePosture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuarantineReadmissionOutcomeObservation {
    states: [QuarantineReadmissionState; 2],
}

pub const fn map_quarantine_record(
    posture: PhysicalQuarantinePosture,
) -> QuarantineRecordObservation {
    QuarantineRecordObservation { posture }
}

impl QuarantineRecordObservation {
    pub const fn posture(&self) -> PhysicalQuarantinePosture {
        self.posture
    }

    pub const fn proves_repair(&self) -> bool {
        false
    }

    pub fn states(&self) -> impl Iterator<Item = QuarantineReadmissionState> {
        [
            QuarantineReadmissionState::Proposed,
            QuarantineReadmissionState::Sealed,
        ]
        .into_iter()
    }
}

pub const fn map_quarantine_readmission_outcome(
    outcome: RecoveryLayoutReadmissionOutcomeView<'_>,
) -> QuarantineReadmissionOutcomeObservation {
    let final_state = match outcome {
        RecoveryLayoutReadmissionOutcomeView::Readmitted(_) => {
            QuarantineReadmissionState::Readmitted
        }
        RecoveryLayoutReadmissionOutcomeView::Denied(
            RecoveryLayoutReadmissionAdmissionDenial::NoForegroundAuthority,
        ) => QuarantineReadmissionState::RetainedForAudit,
        RecoveryLayoutReadmissionOutcomeView::Denied(_) => QuarantineReadmissionState::Denied,
    };
    QuarantineReadmissionOutcomeObservation {
        states: [
            QuarantineReadmissionState::RecoveryVerificationPending,
            final_state,
        ],
    }
}

impl QuarantineReadmissionOutcomeObservation {
    pub fn states(&self) -> impl Iterator<Item = QuarantineReadmissionState> + '_ {
        self.states.iter().copied()
    }
}
